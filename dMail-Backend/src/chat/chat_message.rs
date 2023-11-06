use serde::{Deserialize, Serialize};

use crate::{
    config::datatype::{ChatID, MessageID, Timestamp, UserID},
    user::ChatMessageType,
};

/** `ChatMessage` 聊天消息数据类型
*/
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub r#type: ChatMessageType,
    pub in_chat_id: MessageID,
    pub chat_id: ChatID,
    pub sender_id: UserID,
    pub serialized_content: String,
    pub timestamp: Timestamp,
}
