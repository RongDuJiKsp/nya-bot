use kovi_plugin_dev_utils::config;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Duration;

static EMOJI_ATTACK_CONFIG: OnceLock<EmojiAttackConfig> = OnceLock::new();
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EmojiAttackConfig {
    pub allow_monkey_groups: HashSet<i64>, //允许对标记的用户贴emoji的群组上下文
    pub emoji: Vec<String>,                //贴的emoji id
    pub wait_ms: Option<u64>,              //贴间隔时间 默认300ms
}
config!(EmojiAttackConfig, EMOJI_ATTACK_CONFIG);
impl EmojiAttackConfig {
    pub fn wait_duration(&self) -> Duration {
        Duration::from_millis(self.wait_ms.unwrap_or(300))
    }
}
