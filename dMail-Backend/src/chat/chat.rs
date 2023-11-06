use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::{
    config::datatype::{ChatID, UserID},
    user::{user_session::send_message, ChatMessageType, UserSendMessageData},
};

/** `ChatMember` 聊天成员数据类型
*/
#[derive(Clone)]
pub enum ChatMembers {
    Private((UserID, UserID)),
    Group(Vec<UserID>),
}

/** `ChatType` 聊天类型数据类型
*/
pub enum ChatType {
    Private((UserID, UserID)),
    /// 包含群聊人数
    Group(UserID),
}

/** `Chat` 聊天的数据类型
*/
#[allow(dead_code)]
pub struct Chat {
    pub id: ChatID,
    pub members: ChatMembers,
}

/** `GroupChatData` 群聊数据类型
*/
pub struct GroupChatData {
    pub name: String,
    pub chat_id: ChatID,
    pub owner_user_id: UserID,
}

/** `ChatInfo` 聊天基本信息数据类型
*/
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatInfo {
    pub id: ChatID,
    pub name: String,
    pub avater_hash: String,
}

/** `send_admin_message_to_group_chat` 向群聊中发送系统消息
*/
pub async fn send_admin_message_to_group_chat(chat_id: ChatID, text: String) {
    send_message(
        0,
        UserSendMessageData {
            r#type: ChatMessageType::Text,
            client_id: 0,
            chat_id,
            serialized_content: serde_json::to_string(&text).unwrap(),
            timestamp: Utc::now().timestamp_millis() as u64,
        },
    )
    .await;
}
