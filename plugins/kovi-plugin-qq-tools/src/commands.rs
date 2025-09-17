use crate::config::QQToolsConfig;
use kovi::serde_json::json;
use kovi::{Message, serde_json};
use kovi_plugin_command_exec::app::{BotCommand, BotCommandBuilder};

pub async fn register_tool_cmd() {
    BotCommandBuilder::on_common_command("$raw_msg", move |e| print_raw(e)).await;
}
async fn print_raw(e: BotCommand) {
    let group_id = match e.event.group_id {
        Some(group_id) => group_id,
        None => return,
    };
    if !QQToolsConfig::get().allow_exec_groups.contains(&group_id) {
        return;
    }
    let target_msg = e
        .event
        .message
        .get("reply")
        .iter()
        .filter_map(|s| s.data.get("id"))
        .filter_map(|v| v.as_str())
        .filter_map(|s| s.parse::<i32>().ok())
        .next();
    match target_msg {
        Some(msg_id) => match e.bot.get_msg(msg_id).await {
            Ok(msg) => {
                serde_json::to_string_pretty(&msg.data)
                    .ok()
                    .and_then(|s| {
                        Some(json!([
                            {
                                "type": "text",
                                "data": {
                                    "text": s
                                }
                            },
                            {
                                "type": "reply",
                                "data": {
                                    "id": format!("{}", msg_id),
                                }
                            }
                        ]))
                    })
                    .and_then(|val| Message::from_value(val).ok())
                    .and_then(|m| {
                        e.bot.send_group_msg(group_id, m);
                        Some(())
                    });
            }
            Err(err) => {
                e.event.reply(format!("坏掉了喵: {}", err.echo));
            }
        },
        None => e.event.reply("未找到需要打印的消息喵！"),
    }
}
