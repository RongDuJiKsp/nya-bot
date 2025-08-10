use kovi_plugin_dev_utils::config;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

static CHAT_CONFIG: OnceLock<ChatConfig> = OnceLock::new();
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ChatModelCallConfig {
    //openai Api 参数 照着填即可
    pub key: String,
    pub endpoint: String,
    //最大token限制
    pub max_tokens: u16,
    //角色扮演机器人相关
    pub role_model: String,
    pub role_prompt: String,
    pub role_context_expiration_time_second: u64, //角色扮演机器人的对话记忆过期时间
    pub role_max_message: usize,                  //角色扮演机器人的对话窗口大小
    //角色扮演机器人对话拆分相关
    pub dot_wait_tag: String,              //机器人大段话变成对话的分隔符
    pub dot_wait_time_ms: u64,             //机器人发这大段话的时间
    pub dot_wait_pre_char_ms: Option<u64>, //若配置了这个则机器人会将每段对话的长度与这个综合计算得到停顿的时间
    //聪明机器人相关
    pub smart_model: String,
    pub smart_prompt: String,
}
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ChatConfig {
    pub allow_groups: HashSet<i64>,
    pub model: ChatModelCallConfig,
}
config!(ChatConfig, CHAT_CONFIG, "chat_config.json");

pub struct SyncControl;
static LIVE: AtomicBool = AtomicBool::new(true);
impl SyncControl {
    pub fn set_bot_run(run: bool) {
        LIVE.store(run, Ordering::Relaxed);
    }
    pub fn running() -> bool {
        LIVE.load(Ordering::Relaxed)
    }
}
