use serde::{Deserialize, Serialize};

use crate::config::datatype::{Timestamp, UserID};
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
/** `UserInfo` 用户信息的数据类型
*/
pub struct UserInfo {
    pub user_id: UserID,
    pub user_name: String,
    pub avater_hash: String,
}

/** `Token` 与客户端连接时令牌的数据类型
*/
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub token: String,
    pub timestamp: Timestamp,
}
