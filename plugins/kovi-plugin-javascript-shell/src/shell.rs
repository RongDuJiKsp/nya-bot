use boa_engine::Source;
use boa_engine::error::JsErasedError;
use kovi::chrono::{DateTime, Utc};
use kovi::log::warn;
use kovi::tokio::sync::{Mutex, RwLock, mpsc};
use kovi::{RuntimeBot, serde_json};
use kovi_plugin_command_exec::app::{BotCommand, BotCommandBuilder};
use kovi_plugin_dev_utils::infoev::MemberInfo;
use std::collections::HashMap;
use std::sync::atomic::{AtomicI64, AtomicUsize, Ordering};
use std::sync::{Arc, OnceLock};
use std::thread::spawn;

pub async fn register_shell_cmd(rt_bot: Arc<RuntimeBot>) {
    let shell_bot = rt_bot.clone();
    BotCommandBuilder::on_super_command("$shell", move |e| exec_shell_cmd(e, shell_bot.clone()))
        .await;
}
async fn exec_shell_cmd(e: BotCommand, bot: Arc<RuntimeBot>) {
    let mut arg = e.args.iter();
    if let Some(sub_cmd) = arg.next() {
        match sub_cmd.as_str() {
            "help" => {
                e.event.reply_and_quote("$shell 指令目前支持Js Shell的喵。使用$shell new 创建新shell，$shell use 切换绑定的shell,$shell lock 锁定个人上下文shell $shell unlock 解锁个人上下文 $shell list shell和所有者列表");
                return;
            }
            "lock" => {
                if let Some(lock_n) = arg.next().and_then(|e| e.parse::<usize>().ok()) {
                    match ShellMemory::get().lock_shell(e.event.user_id, lock_n).await {
                        Ok(_) => {
                            e.event.reply_and_quote(format!(
                                "shell绑定成功喵。现在shell{lock_n}是你的了喵"
                            ));
                        }
                        Err(_) => {
                            e.event
                                .reply_and_quote("shell不存在或者shell已经被使用了喵");
                        }
                    }
                } else {
                    e.event
                        .reply_and_quote("参数必须是shell编号喵,请检查参数是否正确喵");
                }
                return;
            }
            "unlock" => {
                if let Some(lock_n) = arg.next().and_then(|e| e.parse::<usize>().ok()) {
                    match ShellMemory::get()
                        .unlock_shell(e.event.user_id, lock_n)
                        .await
                    {
                        Ok(_) => {
                            e.event.reply_and_quote("shell解绑成功喵");
                        }
                        Err(_) => {
                            e.event.reply_and_quote("shell不存在或者shell不是你的喵");
                        }
                    }
                } else {
                    e.event
                        .reply_and_quote("参数必须是shell编号喵,请检查参数是否正确喵");
                }
                return;
            }
            "use" => {
                if let Some(lock_n) = arg.next().and_then(|e| e.parse::<usize>().ok()) {
                    match ShellMemory::get().use_shell(e.event.user_id, lock_n).await {
                        Ok(_) => {
                            e.event.reply_and_quote("shell上下文切换成功喵");
                        }
                        Err(_) => {
                            e.event.reply_and_quote("shell不存在或者shell不是你的喵");
                        }
                    }
                } else {
                    e.event
                        .reply_and_quote("参数必须是shell编号喵,请检查参数是否正确喵");
                }
            }
            "list" => {
                let mut lines = vec![String::from("Shel列表如下喵：")];
                for (sid, sh) in ShellMemory::get().instance_gid.read().await.iter() {
                    let owner = sh.owner.load(Ordering::Relaxed);
                    let owner_nick = if let Some(g) = e.event.group_id {
                        bot.get_group_member_info(g, owner, false)
                            .await
                            .ok()
                            .and_then(|a| serde_json::from_value::<MemberInfo>(a.data).ok())
                            .map(|n| n.nickname.clone())
                            .unwrap_or(String::from("Unknown"))
                    } else {
                        String::from("No Group Member")
                    };
                    lines.push(format!("编号{sid}的shell属于{owner}({owner_nick})喵"));
                }
                e.event.reply_and_quote(lines.join("\n"));
                return;
            }
            "new" => {
                let (id, _) = ShellMemory::get().new_shell(e.event.sender.user_id).await;
                e.event
                    .reply_and_quote(format!("shell创建成功喵！现在{id}是你的了喵"));
            }
            cmd => {
                if let Some(sh) = ShellMemory::get().shell(e.event.sender.user_id).await {
                    let mut full_cmd = vec![cmd];
                    full_cmd.append(&mut arg.map(|x| x.as_str()).collect());
                    e.event.reply_and_quote("异步任务创建成功喵！");
                    if let Err(_) = sh
                        .js_eval_queue
                        .send((Utc::now().timestamp(), full_cmd.join(" ")))
                        .await
                    {
                        e.event.reply_and_quote(format!(
                            "Js引擎已经停止运行了喵。原因：{}",
                            sh.exit_error
                                .read()
                                .await
                                .as_ref()
                                .map(|x| x.as_str())
                                .unwrap_or("none")
                        ))
                    }
                    match sh.res_queue.lock().await.recv().await {
                        None => {
                            e.event.reply_and_quote(format!(
                                "Js引擎已经停止运行了喵!原因：{}",
                                sh.exit_error
                                    .read()
                                    .await
                                    .as_ref()
                                    .map(|x| x.as_str())
                                    .unwrap_or("none")
                            ));
                        }
                        Some((time, res)) => e.event.reply_and_quote(format!(
                            "任务{}的执行结果如下：{}",
                            DateTime::from_timestamp(time, 0)
                                .unwrap()
                                .format("%Y-%m-%d %H:%M:%S"),
                            res.unwrap_or_else(|e| e.to_string())
                        )),
                    }
                } else {
                    e.event.reply_and_quote("你没有锁定自己的shell喵");
                }
            }
        }
    } else {
        e.event.reply_and_quote("请指定命令喵");
    }
}

#[derive(Debug, Default)]
struct ShellMemory {
    instance_gid: RwLock<HashMap<usize, Arc<JavaScriptEngine>>>,
    id_gen: AtomicUsize,
    user_shell: RwLock<HashMap<i64, usize>>,
}
static SHELL_MEMORY: OnceLock<ShellMemory> = OnceLock::new();
impl ShellMemory {
    fn get() -> &'static ShellMemory {
        SHELL_MEMORY.get_or_init(|| ShellMemory::default())
    }
    async fn new_shell(&self, owner: i64) -> (usize, Arc<JavaScriptEngine>) {
        let shell_id = self.id_gen.fetch_add(1, Ordering::Relaxed);
        self.user_shell.write().await.insert(owner, shell_id);
        (
            shell_id,
            self.instance_gid
                .write()
                .await
                .entry(shell_id)
                .or_insert(Arc::new(JavaScriptEngine::new(owner)))
                .clone(),
        )
    }
    async fn shell(&self, uid: i64) -> Option<Arc<JavaScriptEngine>> {
        if let Some(shell_id) = self.user_shell.read().await.get(&uid).copied() {
            if let Some(sh) = self.instance_gid.read().await.get(&shell_id) {
                Some(sh.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
    async fn use_shell(&self, uid: i64, sid: usize) -> Result<(), ()> {
        if self
            .instance_gid
            .read()
            .await
            .get(&sid)
            .and_then(|s| {
                if s.owner.load(Ordering::Relaxed) == uid {
                    Some(0)
                } else {
                    None
                }
            })
            .is_some()
        {
            self.user_shell.write().await.insert(uid, sid);
            return Ok(());
        }
        Err(())
    }
    async fn lock_shell(&self, uid: i64, sid: usize) -> Result<(), ()> {
        if ShellMemory::get()
            .instance_gid
            .read()
            .await
            .get(&sid)
            .and_then(|sh| {
                sh.owner
                    .compare_exchange(-1, uid, Ordering::Relaxed, Ordering::Relaxed)
                    .ok()
            })
            .is_some()
        {
            //先加锁再插入
            self.user_shell.write().await.insert(uid, sid);
            Ok(())
        } else {
            Err(())
        }
    }
    async fn unlock_shell(&self, uid: i64, sid: usize) -> Result<(), ()> {
        //尝试删一下 删不了也没关系
        let mut lock = self.user_shell.write().await;
        lock.get(&uid)
            .and_then(|x| if *x == sid { Some(*x) } else { None })
            .and_then(|_x| lock.remove(&uid));
        ShellMemory::get()
            .instance_gid
            .read()
            .await
            .get(&sid)
            .and_then(|sh| {
                sh.owner
                    .compare_exchange(uid, -1, Ordering::Relaxed, Ordering::Relaxed)
                    .ok()
            })
            .map(|_x| ())
            .ok_or(())
    }
}
const JS_ENGINE_QUEUE_BUF_SIZE: usize = 50;
#[derive(Debug, Clone)]
struct JavaScriptEngine {
    owner: Arc<AtomicI64>,
    js_eval_queue: mpsc::Sender<(i64, String)>,
    res_queue: Arc<Mutex<mpsc::Receiver<(i64, Result<String, JsErasedError>)>>>,
    exit_error: Arc<RwLock<Option<String>>>,
}
impl JavaScriptEngine {
    fn new(owner: i64) -> JavaScriptEngine {
        let (code_tx, mut code_rx) = mpsc::channel::<(i64, String)>(JS_ENGINE_QUEUE_BUF_SIZE);
        let (resp_tx, resp_rs) =
            mpsc::channel::<(i64, Result<String, JsErasedError>)>(JS_ENGINE_QUEUE_BUF_SIZE);
        let err_output = Arc::new(RwLock::new(None));
        let err_input = err_output.clone();
        spawn(move || {
            let mut ctx = boa_engine::Context::default();
            let mut last_output = String::new();
            while let Some((submit_time, code)) = code_rx.blocking_recv() {
                let js_out = ctx.eval(Source::from_bytes(code.as_bytes()));
                last_output = format!("{:?}", &js_out);
                let rs_out: Result<String, JsErasedError> = match js_out {
                    Ok(val) => match val.to_string(&mut ctx) {
                        Ok(o) => Ok(o.to_std_string_escaped()),
                        Err(e) => Err(e.into_erased(&mut ctx)),
                    },
                    Err(e) => Err(e.into_erased(&mut ctx)),
                };
                if let Err(_) = resp_tx.blocking_send((submit_time, rs_out)) {
                    *err_input.blocking_write() = Some(String::from("异常退出：输出写端已关闭"));
                    break;
                }
            }
            warn!(
                "JS引擎已退出:{}\n最后一次输出:{}",
                err_input
                    .blocking_read()
                    .as_ref()
                    .map(|x| x.as_str())
                    .unwrap_or("输入写端已退出"),
                last_output
            );
        });
        JavaScriptEngine {
            res_queue: Arc::new(Mutex::new(resp_rs)),
            js_eval_queue: code_tx,
            exit_error: err_output,
            owner: Arc::new(AtomicI64::new(owner)),
        }
    }
}
