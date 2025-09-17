mod commands;
mod config;

use crate::commands::register_tool_cmd;
use crate::config::QQToolsConfig;
use kovi::PluginBuilder as plugin;

#[kovi::plugin]
async fn main() {
    let bot = plugin::get_runtime_bot();
    QQToolsConfig::init(&bot).unwrap();
    register_tool_cmd().await;
}
