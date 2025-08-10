use crate::config::ChatConfig;
use kovi::MsgEvent;
use kovi::tokio::time::sleep;
use std::sync::Arc;
use std::time::Duration;

pub fn reply_as_im(ev: Arc<MsgEvent>, reply: &str) {
    let quotes = reply
        .split(&ChatConfig::get().model.dot_wait_tag)
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    kovi::spawn(async move {
        for reply in quotes.into_iter().filter(|x| x.len() > 0) {
            sleep(Duration::from_millis(
                ChatConfig::get()
                    .model
                    .dot_wait_pre_char_ms
                    .map(|c| c * reply.len() as u64)
                    .unwrap_or_else(|| ChatConfig::get().model.dot_wait_time_ms),
            ))
            .await;
            ev.reply(reply);
        }
    });
}
