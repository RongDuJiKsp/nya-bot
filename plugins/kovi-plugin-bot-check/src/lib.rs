mod config;
mod data;
mod handle;

use crate::config::BanConfig;
use crate::data::BanData;
use kovi::event::GroupMsgEvent;
use kovi::log::{error, info};
use kovi::{PluginBuilder as plugin, RequestEvent, RuntimeBot};
use kovi_plugin_command_exec::app::{BotCommand, BotCommandBuilder};
use kovi_plugin_dev_utils::msg::get_at_targets;
use std::sync::Arc;

#[kovi::plugin]
async fn main() {
    let bot = plugin::get_runtime_bot();
    BanConfig::init(&bot).expect("Failed to initialize BanDynConfig");
    BanData::init(&bot).expect("Failed to initialize BanData");
    let bot1 = bot.clone();
    let bot2 = bot.clone();
    plugin::on_group_msg(move |e| on_msg(e, bot1.clone()));
    plugin::on_request(move |e| on_request(e, bot2.clone()));
    BotCommandBuilder::on_super_command("$unban", |e| on_unban(e)).await;
}
async fn on_msg(e: Arc<GroupMsgEvent>, bot: Arc<RuntimeBot>) {
    if let Err(error) = handle::on_chat(e, bot).await {
        error!("{:?}", error);
    }
}
async fn on_request(e: Arc<RequestEvent>, bot: Arc<RuntimeBot>) {
    info!("接收到群邀请请求：{:?}", e);
    if let Err(error) = handle::on_request(e, bot).await {
        error!("{:?}", error);
    }
}
async fn on_unban(c: BotCommand) {
    if !c.event.is_group() {
        return;
    }
    let targets = {
        let mut targets = get_at_targets(&c.event);
        targets.append(
            &mut c
                .args
                .iter()
                .filter_map(|x| x.parse::<i64>().ok())
                .collect::<Vec<_>>(),
        );
        targets
    };
    let lock = BanData::get();
    let mut w = lock.write().await;
    for target in targets {
        *w.invite_action_times
            .entry(c.event.group_id.unwrap())
            .or_default()
            .entry(target)
            .or_default() = 0;
        *w.chat_action_times
            .entry(c.event.group_id.unwrap())
            .or_default()
            .entry(target)
            .or_default() = 0;
        c.bot.set_group_ban(c.event.group_id.unwrap(), target, 0);
    }
}
