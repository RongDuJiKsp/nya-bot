use crate::config::BanConfig;
use crate::data::BanData;
use kovi::event::GroupMsgEvent;
use kovi::log::info;
use kovi::{RequestEvent, RuntimeBot, serde_json};
use kovi_plugin_dev_utils::infodwd::{GroupRequestEvent, InfoDwd};
use std::sync::Arc;

async fn do_ban(e: Arc<GroupMsgEvent>, bot: Arc<RuntimeBot>) {
    let ban_data = BanData::get();
    let mut ban_lock = ban_data.write().await;
    let cnt = ban_lock.chat_action_times.entry(e.group_id).or_default();
    let times = cnt.entry(e.user_id).or_default();
    *times += 1;
    if BanConfig::get()
        .enable_chat_kick
        .map(|val| *times >= val)
        .unwrap_or(false)
    {
        bot.set_group_kick(
            e.group_id,
            e.user_id,
            BanConfig::get().kick_can_request_or_default(),
        );
        e.reply_and_quote(format!(
            "用户{} 因为触发违禁词达到次数{} 已被踢出！！如需申诉请联系管理员 (哈气)",
            e.user_id, *times
        ));
        //身死债消
        cnt.remove(&e.user_id);
    } else if BanConfig::get()
        .enable_chat_shut_up
        .map(|val| *times >= val)
        .unwrap_or(false)
    {
        bot.set_group_ban(
            e.group_id,
            e.user_id,
            BanConfig::get().chat_shut_up_duration().as_secs() as usize,
        );
        e.reply_and_quote(format!(
            "用户{} 因为触发违禁词达到次数{} 已被封禁{}s！如需申诉请联系管理员 (哈气)",
            e.user_id,
            *times,
            BanConfig::get().chat_shut_up_duration().as_secs()
        ));
    } else if BanConfig::get().enable_chat_kick.is_some()
        || BanConfig::get().enable_chat_shut_up.is_some()
    {
        e.reply_and_quote("爆了")
    }
}
fn hit_by_regex(e: Arc<GroupMsgEvent>) -> bool {
    BanConfig::get()
        .chat_regex_list
        .iter()
        .map(|reg| reg.is_match(&e.human_text))
        .fold(false, |p, c| p || c)
}
pub async fn on_chat(e: Arc<GroupMsgEvent>, bot: Arc<RuntimeBot>) -> Result<(), anyhow::Error> {
    //不是我喜欢的群，直接屏蔽
    if !BanConfig::get().enable_group.contains(&e.group_id) {
        return Ok(());
    }

    if hit_by_regex(e.clone()) {
        do_ban(e.clone(), bot.clone()).await;
    }

    Ok(())
}
pub async fn on_request(e: Arc<RequestEvent>, bot: Arc<RuntimeBot>) -> Result<(), anyhow::Error> {
    //这里只处理群内邀请请求，即request_type=group,sub_type=invite
    if e.request_type != "group" {
        return Ok(());
    }
    let group_request = serde_json::from_value::<GroupRequestEvent>(e.original_json.clone())
        .map_err(|e| anyhow::anyhow!("Fail to serde json on GroupRequestEvent:{:?}", e))?;
    if group_request.sub_type != "invite" {
        return Ok(());
    }
    //不是我喜欢的群，直接屏蔽
    if !BanConfig::get()
        .enable_group
        .contains(&group_request.group_id)
    {
        return Ok(());
    }
    //先查member data 防止退群了
    let member_data =
        InfoDwd::get_member_info(bot.clone(), group_request.group_id, group_request.user_id)
            .await?;
    info!("{:?}", member_data);
    if BanConfig::get()
        .enable_invite_ban
        .as_ref()
        .and_then(|c| c.min_activate)
        .and_then(|e| {
            member_data
                .level
                .parse::<i32>()
                .ok()
                .map(|level| level >= e)
        })
        .unwrap_or(true)
    {
        return Ok(());
    }
    let user_data = InfoDwd::get_user_info(bot.clone(), group_request.user_id).await?;
    info!("{:?}", user_data);
    if BanConfig::get()
        .enable_invite_ban
        .as_ref()
        .and_then(|c| c.min_level)
        .map(|e| user_data.level >= e as i64)
        .unwrap_or(true)
    {
        return Ok(());
    }
    let ban_data = BanData::get();
    let mut ban_lock = ban_data.write().await;
    let cnt = ban_lock
        .invite_action_times
        .entry(group_request.group_id)
        .or_default();
    let times = cnt.entry(group_request.user_id).or_default();
    *times += 1;
    if BanConfig::get()
        .enable_invite_kick
        .map(|e| *times >= e)
        .unwrap_or(false)
    {
        bot.set_group_kick(
            group_request.group_id,
            group_request.user_id,
            BanConfig::get().kick_can_request_or_default(),
        );
        bot.send_group_msg(
            group_request.group_id,
            format!(
                "用户{} 因为邀请群成员达到次数{} 已被踢出！如需申诉请联系管理员",
                group_request.user_id, *times,
            ),
        );
    } else if BanConfig::get().enable_invite_kick.is_some() {
        bot.send_group_msg(
            group_request.group_id,
            format!("用户{} 邀请群成员达到次数{}", group_request.user_id, *times,),
        );
    }
    Ok(())
}
