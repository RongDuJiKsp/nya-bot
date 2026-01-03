use crate::config::EmojiAttackConfig;
use kovi::PluginBuilder as plugin;
use kovi::bot::runtimebot::RuntimeBot;

use crate::data::EmojiAttackData;
use crate::handle::{handle_group_msg, register_cmd};
use std::sync::Arc;

pub async fn init() {
    let bot: Arc<RuntimeBot> = plugin::get_runtime_bot();
    EmojiAttackConfig::init(&bot).unwrap();
    EmojiAttackData::init(&bot).unwrap();
    plugin::on_group_msg(move |e| handle_group_msg(e, bot.clone()));
    register_cmd().await
}
