use crate::config::{ChatConfig, SyncControl};
use crate::handlers::tool::reply_as_im;
use crate::ml;
use anyhow::anyhow;
use async_openai::types::{
    ChatCompletionRequestAssistantMessage, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage, ChatCompletionRequestUserMessage,
};
use kovi::log::{error, info};
use kovi::tokio::sync::RwLock;
use kovi::{MsgEvent, RuntimeBot};
use kovi_plugin_dev_utils::infoev::InfoEv;
use kovi_plugin_dev_utils::msg::get_at_targets;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, OnceLock};
use std::time::SystemTime;

pub async fn handle_group_chat(
    bot: Arc<RuntimeBot>,
    event: Arc<MsgEvent>,
) -> Result<(), anyhow::Error> {
    //只考虑已经监听的群
    if !ChatConfig::get()
        .allow_groups
        .contains(&event.group_id.ok_or(anyhow!("找不到群id"))?)
    {
        return Ok(());
    }
    //有人@猫娘
    if get_at_targets(&event)
        .into_iter()
        .any(|e| e == event.self_id)
    {
        at_me(event.clone()).await;
        return Ok(());
    }

    let bot_info = InfoEv::self_bot_info(&bot, &event).await.ok();
    //若有bot info
    if let Some(bot_if) = &bot_info {
        //判断消息是否有猫娘的名字
        if event
            .text
            .as_ref()
            .and_then(|e| {
                if e.contains(&bot_if.nickname) {
                    Some(())
                } else {
                    None
                }
            })
            .is_some()
        {
            call_me_msg(event.clone()).await;
            return Ok(());
        }
    }
    //判断消息是否有猫娘两个字
    if event
        .text
        .as_ref()
        .and_then(|e| if e.contains("猫娘") { Some(()) } else { None })
        .is_some()
    {
        method_me(event.clone()).await;
        return Ok(());
    }

    Ok(())
}
async fn call_me_msg(e: Arc<MsgEvent>) {
    at_me(e).await;
}

async fn method_me(e: Arc<MsgEvent>) {
    if !SyncControl::running() {
        return;
    }
    e.reply("是不是有人叫我喵");
}
type UnixTime = u64;
#[derive(Debug, Default)]
pub struct NyaCatMemory {
    //user chat time and message
    user_memory: HashMap<i64, VecDeque<(UnixTime, ChatCompletionRequestMessage)>>,
}
static CAT_MEMORY: OnceLock<RwLock<NyaCatMemory>> = OnceLock::new();
impl NyaCatMemory {
    pub fn load() -> &'static RwLock<NyaCatMemory> {
        CAT_MEMORY.get_or_init(|| RwLock::new(NyaCatMemory::default()))
    }
    fn system_msg() -> ChatCompletionRequestMessage {
        ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage::from(
            ChatConfig::get().model.role_prompt.as_str(),
        ))
    }
    pub fn clean(&mut self) {
        self.user_memory.clear();
    }
    fn load_mem(&mut self, user_id: i64, new_msg: &str) -> Vec<ChatCompletionRequestMessage> {
        info!("群聊或用户{user_id}发出提问:{new_msg}");
        let now_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System Time Error!!!!!!")
            .as_secs();
        let arr = self.user_memory.entry(user_id).or_default();
        arr.push_back((
            now_time,
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage::from(new_msg)),
        ));
        while let Some((chat_time, msg)) = arr.pop_front() {
            if arr.len() < ChatConfig::get().model.role_max_message
                && now_time - chat_time
                    < ChatConfig::get().model.role_context_expiration_time_second
            {
                arr.push_front((chat_time, msg));
                break;
            } else {
                info!("模型忘记了{:?}", msg);
            }
        }
        let mut v = vec![Self::system_msg()];
        v.append(&mut arr.iter().cloned().map(|x| x.1).collect());
        v
    }
    fn save_mem(&mut self, user_id: i64, new_chat_msg: &str) {
        info!("模型对群聊或用户{user_id}回答:{new_chat_msg}");
        let now_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("System Time Error!!!!!!")
            .as_secs();
        let ctx = self.user_memory.entry(user_id).or_default();
        ctx.push_back((
            now_time,
            ChatCompletionRequestMessage::Assistant(ChatCompletionRequestAssistantMessage::from(
                new_chat_msg,
            )),
        ));
        info!("模型最终记忆：{:?}", ctx);
    }
}
async fn at_me(e: Arc<MsgEvent>) {
    //如果是指令则忽略问话
    if e.text.as_ref().map(|e| e.starts_with("$")).unwrap_or(true) {
        return;
    }

    //否则当成问话
    if !SyncControl::running() {
        //如果关闭了则不响应问话
        return;
    }
    if let Some(question) = e
        .text
        .as_ref()
        .and_then(|s| if s.len() > 0 { Some(s) } else { None })
    {
        let ctx_id = e.group_id.unwrap_or(e.sender.user_id);
        let chat = NyaCatMemory::load()
            .write()
            .await
            .load_mem(ctx_id, question);
        info!("模型思考上下文：{:?}", chat);
        match ml::get_reply_as_nya_cat(chat).await {
            Ok(out) => {
                NyaCatMemory::load().write().await.save_mem(ctx_id, &out);
                reply_as_im(e.clone(), &out)
            }
            Err(err) => {
                e.reply_and_quote("不想理你喵");
                error!("模型在回复时发生错误：{}", err);
            }
        }
    } else {
        e.reply_and_quote("叫我什么事喵？");
    }
}
