use kovi_plugin_dev_utils::config;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::OnceLock;

static QQ_TOOLS_CONFIG: OnceLock<QQToolsConfig> = OnceLock::new();
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct QQToolsConfig {
    pub allow_exec_groups: HashSet<i64>, //允许执行tools的群组
}
config!(QQToolsConfig, QQ_TOOLS_CONFIG);
