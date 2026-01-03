use crate::config::CommandExecConfig;
use kovi::log::{error, info};
use kovi::tokio::sync::{RwLock, broadcast};
use kovi::{MsgEvent, RuntimeBot};
use std::collections::HashSet;
use std::sync::{Arc, OnceLock};

#[derive(Debug)]
pub struct BotCommandBuilder {
    event_bus: broadcast::Sender<BotCommand>,
    super_command: HashSet<&'static str>,
    common_command: HashSet<&'static str>,
}
static COMMAND_BUILDER: OnceLock<RwLock<BotCommandBuilder>> = OnceLock::new();
pub static GLOBAL_BOT: OnceLock<Arc<RuntimeBot>> = OnceLock::new();
impl BotCommandBuilder {
    fn instance_lock() -> &'static RwLock<BotCommandBuilder> {
        COMMAND_BUILDER.get_or_init(|| {
            let (tx, _) = broadcast::channel(100);
            let b = BotCommandBuilder {
                event_bus: tx,
                super_command: HashSet::new(),
                common_command: HashSet::new(),
            };
            RwLock::new(b)
        })
    }
    pub async fn on_common_command<F, Fut>(cmd: &'static str, hd: F)
    where
        F: Fn(BotCommand) -> Fut + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Send,
    {
        let mut f = BotCommandBuilder::instance_lock().write().await;
        f.subscribe(cmd, hd);
        f.common_command.insert(cmd);
        info!("Common 命令 {} 注册成功", cmd)
    }
    pub async fn on_super_command<F, Fut>(cmd: &'static str, hd: F)
    where
        F: Fn(BotCommand) -> Fut + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Send,
    {
        let mut f = BotCommandBuilder::instance_lock().write().await;
        f.subscribe(cmd, hd);
        f.super_command.insert(cmd);
        info!("Super 命令 {} 注册成功", cmd)
    }
    fn subscribe<F, Fut>(&self, cmd: &'static str, hd: F)
    where
        F: Fn(BotCommand) -> Fut + Send + Sync + 'static,
        Fut: Future + Send,
        Fut::Output: Send,
    {
        let mut rec = self.event_bus.subscribe();
        kovi::spawn(async move {
            while let Ok(event) = rec.recv().await {
                if event.cmd.as_str() != cmd {
                    continue;
                }
                info!(
                    "命令执行器 {} 执行命令使用参数{:?}",
                    event.cmd.as_str(),
                    &*event.args
                );
                hd(event).await;
            }
        });
    }
}
#[derive(Clone)]
pub struct BotCommand {
    pub cmd: Arc<String>,
    pub args: Arc<Vec<String>>,
    pub event: Arc<MsgEvent>,
    pub bot: Arc<RuntimeBot>,
}
impl BotCommand {
    pub fn from_str(s: &str, e: Arc<MsgEvent>) -> BotCommand {
        let mut args = s.split_whitespace();
        BotCommand {
            event: e,
            cmd: Arc::new(args.next().expect("怎么可能为空捏").to_string()),
            args: Arc::new(args.map(|x| x.to_string()).collect()),
            bot: GLOBAL_BOT.get().expect("Need GLOBAL_BOT Init").clone(),
        }
    }
    pub async fn invoke_command(&self) {
        let f = BotCommandBuilder::instance_lock().read().await;
        if !CommandExecConfig::get().in_context(self.event.as_ref()) {
            self.event.reply_and_quote("不认识的环境喵，害怕喵");
            return;
        }
        if f.super_command.contains(self.cmd.as_str()) {
            if CommandExecConfig::get()
                .in_super_user(self.event.as_ref(), self.bot.clone())
                .await
            {
                self.exec_command(&f.event_bus).await;
            } else {
                self.event.reply_and_quote("你是谁喵！我不认识你喵！哒咩！");
            }
        } else if f.common_command.contains(self.cmd.as_str()) {
            self.exec_command(&f.event_bus).await;
        } else {
            self.event.reply_and_quote("不认识的命令喵！每日疑惑1/1")
        }
    }
    async fn exec_command(&self, sender: &broadcast::Sender<BotCommand>) {
        if let Err(_) = sender.send(self.clone()) {
            error!("分发命令失败!")
        }
    }
}
