use kovi::{RuntimeBot, serde_json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct InfoDwd;
impl InfoDwd {
    pub async fn get_member_info(
        bot: Arc<RuntimeBot>,
        group_id: i64,
        user_id: i64,
    ) -> anyhow::Result<GroupMemberInfo> {
        serde_json::from_value::<GroupMemberInfo>(
            bot.get_group_member_info(group_id, user_id, false)
                .await
                .map_err(|e| anyhow::anyhow!("Fail to get group member:{:?}", e))?
                .data,
        )
        .map_err(|e| anyhow::anyhow!("Fail to de_serde group member:{:?}", e))
    }
    pub async fn get_user_info(bot: Arc<RuntimeBot>, user_id: i64) -> anyhow::Result<UserInfo> {
        serde_json::from_value::<UserInfo>(
            bot.get_stranger_info(user_id, false)
                .await
                .map_err(|e| anyhow::anyhow!("Fail to get user member:{:?}", e))?
                .data,
        )
        .map_err(|e| anyhow::anyhow!("Fail to de_serde user member:{:?}", e))
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupRequestEvent {
    pub time: i64,    // 事件发生时间戳
    pub self_id: i64, // 接收事件的机器人 QQ 号

    pub post_type: String,    // 固定为 "request"
    pub request_type: String, // 固定为 "group"
    pub sub_type: String,     // "add" 或 "invite"

    pub group_id: i64, // 群号
    pub user_id: i64,  // 发送请求的 QQ 号

    pub comment: String, // 验证信息
    pub flag: String,    // 请求 flag，处理请求时需传入
}
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupMemberInfo {
    pub group_id: i64,
    pub user_id: i64,
    pub nickname: String,
    pub card: String,
    pub sex: String, // male / female / unknown
    pub age: i32,
    pub area: String,
    pub join_time: i32,
    pub last_sent_time: i32,
    pub level: String,
    pub role: String, // owner / admin / member
    pub unfriendly: bool,
    pub title: String,
    pub title_expire_time: i32,
    pub card_changeable: bool,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    /// 年龄
    pub age: i64,
    /// 头像
    pub avatar: String,
    #[serde(rename = "Business")]
    pub business: Vec<Business>,
    /// 等级
    pub level: i64,
    /// 昵称
    pub nickname: String,
    /// QID
    pub q_id: Option<String>,
    /// 注册时间
    #[serde(rename = "RegisterTime")]
    pub register_time: String,
    /// 性别
    pub sex: String,
    /// 个性签名
    pub sign: String,
    /// 当前状态信息
    pub status: StatusClass,
    /// 用户 Uin
    pub user_id: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Business {
    pub icon: Option<String>,
    pub ispro: i64,
    pub isyear: i64,
    pub level: i64,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub business_type: i64,
}

/// 当前状态信息
#[derive(Serialize, Deserialize, Debug)]
pub struct StatusClass {
    /// 表情 ID
    pub face_id: Option<i64>,
    /// 信息
    pub message: Option<String>,
    /// 状态 ID
    pub status_id: i64,
}
