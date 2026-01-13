use kovi_plugin_dev_utils::config;
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::VecSkipError;
use serde_with::{DisplayFromStr, serde_as};
use std::cmp;
use std::collections::HashSet;
use std::sync::OnceLock;
use std::time::Duration;

static BAN_CONFIG: OnceLock<BanConfig> = OnceLock::new();
#[serde_as]
#[derive(Default, Deserialize, Serialize)]
pub struct BanConfig {
    pub enable_group: HashSet<i64>, //启用上下文群组
    pub enable_chat_shut_up: Option<i32>, //触发达到次数自动禁言,建议小于自动ban
    pub chat_shut_up_time: Option<u64>, //禁言时长,默认最大值26d23h59m59s,即2,332,799s
    pub enable_chat_kick: Option<i32>, //触发发言ban达到次数时自动ban
    pub enable_invite_ban: Option<InviteBanConfig>, //群内邀请ban处理配置
    pub enable_invite_kick: Option<i32>, //触发邀请ban达到次数时自动ban
    pub kick_can_request: Option<bool>, //能不能再次加群
    // 打击相关
    // regex打击
    #[serde_as(as = "VecSkipError<DisplayFromStr>")]
    pub chat_regex_list: Vec<Regex>, //触发发言匹配的正则表达式列表
    // llm打击
    pub llm_hit_prompt: Option<String>, //提示词
    pub hit_min_len: Option<u64>,// 短于这个长度的不使用llm打击
    pub hit_max_len: Option<u64>,// 长于这个长度的不使用llm打击
    pub open_api_key: Option<String>,
    pub open_api_endpoint: Option<String>,
}
config!(BanConfig, BAN_CONFIG, "ban_config.json");
impl BanConfig {
    pub fn chat_shut_up_duration(&self) -> Duration {
        Duration::from_secs(cmp::min(
            2332799,
            self.chat_shut_up_time.unwrap_or(u64::MAX),
        ))
    }
    pub fn kick_can_request_or_default(&self) -> bool {
        self.kick_can_request.unwrap_or(true) //人体工程学 默认true防止没机会申述
    }
}
#[derive(Default, Deserialize, Serialize)]
pub struct InviteBanConfig {
    pub min_level: Option<i32>,    //当邀请人等级小于这个数时触发ban
    pub min_activate: Option<i32>, //当邀请人群活跃等级小于这个数时ban
}
