use crate::config::EmojiAttackConfig;
use crate::data::EmojiAttackData;
use kovi::RuntimeBot;
use kovi::event::GroupMsgEvent;
use kovi::log::error;
use kovi::tokio::time::sleep;
use kovi_plugin_command_exec::app::{BotCommand, BotCommandBuilder};
use kovi_plugin_dev_utils::msg::get_at_targets;
use kovi_plugin_expand_napcat::NapCatApi;
use std::collections::HashSet;
use std::sync::Arc;

static NULL_STR: String = String::new();
pub async fn handle_group_msg(e: Arc<GroupMsgEvent>, bot: Arc<RuntimeBot>) {
    if !EmojiAttackConfig::get()
        .allow_monkey_groups
        .contains(&e.group_id)
    {
        return;
    }
    if !EmojiAttackData::get()
        .read()
        .await
        .group_users
        .get(&e.group_id)
        .map(|s| s.contains(&e.user_id))
        .unwrap_or(false)
    {
        return;
    }
    let c = EmojiAttackConfig::get();
    for ji in &c.emoji {
        if let Err(e) = bot
            .set_msg_emoji_like(e.message_id as i64, ji.as_str())
            .await
        {
            error!("Failed to set message emoji literally: {}", e);
        }
        sleep(c.wait_duration()).await;
    }
}
fn get_targets(e: &BotCommand) -> Vec<i64> {
    let mut targets = get_at_targets(&e.event)
        .into_iter()
        .filter(|x| *x != e.event.self_id)
        .collect::<Vec<_>>();
    if e.args.len() > 1 {
        targets.append(
            &mut e.args[1..]
                .iter()
                .filter_map(|x| x.parse::<i64>().ok())
                .collect::<Vec<_>>(),
        );
    }
    targets
}
async fn handle_cmd(e: BotCommand) {
    let group_id = match e.event.group_id {
        Some(x) => x,
        None => return,
    };
    let command = e.args.get(0).unwrap_or(&NULL_STR).as_str();
    if HashSet::from(["add", "del", "clean"]).contains(command) {
        handle_auto_cmd(&e, command, group_id).await;
        return;
    }
    if HashSet::from(["atk", "once"]).contains(command) {
        handle_attack_cmd(&e, command).await;
        return;
    }
}
async fn handle_auto_cmd(e: &BotCommand, cmd: &str, group_id: i64) {
    let targets = get_targets(e);
    let data = EmojiAttackData::get();
    let mut lock = data.write().await;

    let result = match cmd {
        "add" => targets
            .iter()
            .map(|target| {
                lock.group_users
                    .entry(group_id)
                    .or_default()
                    .insert(*target)
            })
            .any(|x| x),
        "del" => targets
            .iter()
            .map(|target| lock.group_users.entry(group_id).or_default().remove(target))
            .any(|x| x),
        "clean" => {
            lock.group_users.entry(group_id).or_default().clear();
            true
        }
        _ => {
            error!("存在处理器处理不了的命令");
            false
        }
    };
    e.event
        .reply(format!("操作{}喵！", if result { "成功" } else { "失败" }));
}
async fn handle_attack_cmd(e: &BotCommand, cmd: &str) {
    let c = EmojiAttackConfig::get();
    let target_msg = e
        .event
        .message
        .get("reply")
        .iter()
        .filter_map(|s| s.data.get("id"))
        .filter_map(|v| v.as_str())
        .filter_map(|s| s.parse::<i64>().ok())
        .collect::<Vec<_>>();
    match cmd {
        "atk" => {
            let emoji_list = {
                //过滤无效表情
                let emoji_blacklist = HashSet::from([17]);
                (1..=21)
                    .filter(|x| !emoji_blacklist.contains(x))
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
            };
            for target in &target_msg {
                for ji in &emoji_list {
                    if let Err(e) = e.bot.set_msg_emoji_like(*target, ji.as_str()).await {
                        error!("Failed to set message emoji literally: {}", e);
                    }
                    sleep(c.wait_duration()).await;
                }
            }
        }
        "once" => {
            for target in &target_msg {
                for ji in &c.emoji {
                    if let Err(e) = e.bot.set_msg_emoji_like(*target, ji.as_str()).await {
                        error!("Failed to set message emoji literally: {}", e);
                    }
                    sleep(c.wait_duration()).await;
                }
            }
        }
        _ => {
            error!("存在处理器处理不了的命令");
        }
    }
}
pub async fn register_cmd() {
    BotCommandBuilder::on_super_command("$monkey", move |e| handle_cmd(e)).await;
}
