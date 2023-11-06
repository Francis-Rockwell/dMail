/*! 数据类型定义 */
use aes_gcm::Aes128Gcm;

/** `ClientID` 客户端消息ID
*/
pub type ClientID = u32;

// Chat
/** `ChatID` 聊天ID
*/
pub type ChatID = u64;

// User
/** `UserID` 用户ID
*/
pub type UserID = u32;

// Message
/** `MessageID` 消息ID
*/
pub type MessageID = u64;

/** `MessageIDInChat` 消息在其聊天中的ID
*/
pub type MessageIdInChat = u32;

// UserRequest
/** `UserReqID` 用户请求ID
*/
pub type UserReqId = u32;

/** `SymCipher` 对称秘钥
*/
pub type SymCipher = Aes128Gcm;

/** `EmailCodeValue` 邮箱验证码
*/
pub type EmailCodeValue = u32;

/** `Timestamp` 时间戳
*/
pub type Timestamp = u64;

/** `SerializedChatMessage` 序列化的聊天消息信息
*/
pub type SerializedChatMessage = String;

/** `SerializedChatInfo` 序列化的聊天信息
*/
pub type SerializedChatInfo = String;

/** `SerializedRequest` 序列化的请求信息
*/
pub type SerializedRequest = String;

/** `SerializedFilePubUrl` 序列化的PresignUrl
*/
pub type SerializedFilePubUrl = String;

/** `SerializedUserNotice` 序列化的用户通知信息
*/
pub type SerializedUserNotice = String;

/** `UploadId` 用户上传文件ID
*/
pub type UploadId = u64;

/** `SerializedGroupNotice` 序列化的群公告信息
*/
pub type SerializedGroupNotice = String;

/** `NoticeID` 群公告ID
*/
pub type NoticeID = u32;
