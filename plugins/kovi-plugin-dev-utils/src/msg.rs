use kovi::MsgEvent;

pub fn get_at_targets(e: &MsgEvent) -> Vec<i64> {
    e.message
        .get("at")
        .iter()
        .filter_map(|s| s.data.get("qq"))
        .filter_map(|v| v.as_str())
        .filter_map(|s| s.parse::<i64>().ok())
        .collect::<Vec<_>>()
}
