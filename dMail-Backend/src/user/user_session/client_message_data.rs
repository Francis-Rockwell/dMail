/*!
服务端与客户端通讯的直接处理函数
*/

use lettre::Address;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::{
    config::datatype::{ChatID, ClientID, EmailCodeValue, MessageID, Timestamp, UserID, UserReqId},
    config::{
        config::PWD_PATTERN,
        datatype::{NoticeID, SerializedGroupNotice, UploadId},
        Config,
    },
    user::{UserInfo, UserRequestError, UserRequestState, UserRequsetContent},
};

use super::protocol::DataChecker;

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserSetPubKeyResponse {
    NeedSetPubKey,
    PubKeyError,
    HasApproved,
}

// Register
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserRegisterData {
    pub user_name: String,
    pub password: String,
    pub email_code: EmailCodeValue,
    pub email: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserRegisterResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        user_id: UserID,
    },
    UserNameFormatError,
    PasswordFormatError,
    EmailRegistered,
    EmailInvalid,
    EmailCodeError,
    ServerError,
}

impl DataChecker<UserRegisterResponse> for UserRegisterData {
    fn check_data(&self) -> Result<(), UserRegisterResponse> {
        if self.user_name.len() > Config::get().user.max_user_name_length as usize {
            return Err(UserRegisterResponse::UserNameFormatError);
        }

        if let Err(_) = &self.email.parse::<Address>() {
            return Err(UserRegisterResponse::EmailInvalid);
        }

        if !PWD_PATTERN.is_match(&self.password) {
            return Err(UserRegisterResponse::PasswordFormatError);
        }

        return Ok(());
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum UserUpdateData {
    #[serde(rename_all = "camelCase")]
    UserName { new_name: String },
    #[serde(rename_all = "camelCase")]
    Password {
        new_password: String,
        email_code: EmailCodeValue,
    },
    #[serde(rename_all = "camelCase")]
    AvaterHash { new_hash: String },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserUpdateResponse {
    Success,
    UserNameFormatError,
    PasswordFormatError,
    AvaterHashFormatError,
    EmailCodeError,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateGroupData {
    pub chat_id: ChatID,
    pub content: UserUpdateGroupContent,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum UserUpdateGroupContent {
    #[serde(rename_all = "camelCase")]
    GroupName { new_name: String },
    #[serde(rename_all = "camelCase")]
    Avater { new_avater: String },
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserUpdateGroupInfoResponse {
    Success,
    GroupNameFormatError,
    AvaterFormatError,
    NoPermission,
    DatabaseError,
    ServerError,
}
// Login
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserLoginData {
    pub email: String,
    pub password: Option<String>,
    pub email_code: Option<EmailCodeValue>,
    pub address: Option<SocketAddr>,
    pub token: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserLoginResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        user_id: UserID,
    },
    Unapproved,
    UserNotFound,
    EmailInvalid,
    EmailCodeError,
    UserLogged,
    PasswordError,
    ServerError,
    NeedLogin,
    TokenError,
    TokenExpired,
}

impl DataChecker<UserLoginResponse> for UserLoginData {
    fn check_data(&self) -> Result<(), UserLoginResponse> {
        if let Err(_) = &self.email.parse::<Address>() {
            return Err(UserLoginResponse::EmailInvalid);
        }
        return Ok(());
    }
}

// SendMessage
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSendMessageData {
    pub r#type: ChatMessageType,
    pub client_id: MessageID,
    pub chat_id: ChatID,
    pub timestamp: Timestamp,
    pub serialized_content: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ChatMessageType {
    Text,
    Image,
    File,
    Voice,
    Transfer,
    Revoked,
    ReplyText,
    MentionText,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MentionTextType {
    pub user_ids: Vec<UserID>,
    pub text: String,
}

impl ChatMessageType {
    pub fn get_str(&self) -> &'static str {
        match *self {
            ChatMessageType::Text => "\"Text\"",
            ChatMessageType::Image => "\"Image\"",
            ChatMessageType::File => "\"File\"",
            ChatMessageType::Voice => "\"Voice\"",
            ChatMessageType::Transfer => "\"Transfer\"",
            ChatMessageType::Revoked => "\"Revoked\"",
            ChatMessageType::ReplyText => "\"ReplyText\"",
            ChatMessageType::MentionText { .. } => "\"MentionText\"",
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileMetaData {
    pub file_name: String,
    pub file_size: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
    pub hash: String,
    pub file_name: String,
    pub file_size: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSendMessageResponse {
    pub state: UserSendMessageResponseState,
    pub client_id: MessageID,
    pub chat_id: ChatID,
    pub in_chat_id: Option<MessageID>,
    pub timestamp: Option<Timestamp>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserSendMessageResponseState {
    ServerError,
    DatabaseError,
    LenthLimitExceeded,
    // 对私聊表示不是好友关系
    UserNotInChat,
    UserNotLoggedIn,
    UserBannedInChat,
    ChatNotFound,
    Success,
    ImageNotFound,
    FileNotFound,
    FileMetaDataFormatError,
    ContentError,
    SendNoticeError,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSendFriendRequest {
    pub user_id: UserID,
    pub text: String,
}

// Pull
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserPullData {
    pub last_request_id: UserReqId,
    pub notice_timestamp: Timestamp,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserPullResponse {
    DatabaseError,
    Success,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserGetMessagesData {
    pub chat_id: ChatID,
    pub start_id: MessageID,
    pub end_id: Option<MessageID>,
}

// SendRequest
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSendRequestData {
    pub message: String,
    pub content: UserRequsetContent,
    pub client_id: ClientID,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserRequsetStateUpdated {
    pub req_id: UserReqId,
    pub state: UserRequestState,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserSendRequestState {
    Success,
    DatabaseError,
    RequestError(UserRequestError),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSendRequestResponse {
    pub req_id: Option<UserReqId>,
    pub client_id: ClientID,
    pub state: UserSendRequestState,
}

// SolveRequest
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSolveRequestData {
    pub req_id: UserReqId,
    pub answer: UserRequestState,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserSolveRequestState {
    Success,
    DatabaseError,
    NotHandler,
    AnswerUnsolved,
    RequestNotFound,
    AlreadySolved,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSolveRequestResponse {
    pub state: UserSolveRequestState,
    pub req_id: UserID,
}

// CreateGroupChat
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserCreateGroupChatData {
    pub name: String,
    pub avater_hash: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserCreateGroupChatResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
    },
    ChatNameFormatError,
    DatabaseError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum GetUserInfoResponse {
    Success(UserInfo),
    UserNotFound,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserUnfriendResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
    },
    NotFriend,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum SetSettingResponse {
    Success,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSetAlreadyReadData {
    pub chat_id: ChatID,
    pub in_chat_id: MessageID,
    pub private: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum SetAlreadyReadResponse {
    Success,
    ServerError,
    DatabaseError,
    NotPrivate,
    NotInChat,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserGetGroupUsersResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        user_ids: Vec<UserID>,
    },
    UserNotInChat,
    NotGroupChat,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserQuitGroupChatResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
    },
    // 群主不能退群
    NoPermission,
    UserNotInChat,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSetGroupAdminData {
    pub chat_id: ChatID,
    pub user_id: UserID,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserSetGroupAdminResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        user_id: UserID,
    },
    NotOwner,
    UserNotInChat,
    AlreadyAdmin,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserUnsetGroupAdminData {
    pub chat_id: ChatID,
    pub user_id: UserID,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserUnsetGroupAdminResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        user_id: UserID,
    },
    SameUser,
    NotOwner,
    NotAdmin,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserGroupOwnerTransferData {
    pub chat_id: ChatID,
    pub user_id: UserID,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserGroupOwnerTransferResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        user_id: UserID,
    },
    DatabaseError,
    NotOwner,
    UserNotInChat,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSendGroupNoticeData {
    pub client_id: ClientID,
    pub chat_id: ChatID,
    pub notice: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserPullGroupNoticeData {
    pub chat_id: ChatID,
    pub last_notice_id: NoticeID,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserSendGroupNoticeResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        client_id: ClientID,
        notice_id: NoticeID,
        timestamp: Timestamp,
    },
    NoPermission,
    #[serde(rename_all = "camelCase")]
    LenthLimitExceeded {
        client_id: ClientID,
        chat_id: ChatID,
    },
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserPullGroupNoticeResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        group_notice: Vec<SerializedGroupNotice>,
    },
    UserNotInChat,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserRemoveGroupMemberData {
    pub chat_id: ChatID,
    pub user_id: UserID,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserRemoveGroupMemberResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        user_id: UserID,
    },
    SameUser,
    NoPermission,
    UserNotInChat,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserUploadFileRequestData {
    pub suffix: String,
    pub user_hash: String,
    pub size: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserUploadFileRequestResponse {
    pub user_hash: String,
    pub state: UserUploadFileRequestResponseState,
    pub url: Option<String>,
    pub upload_id: Option<UploadId>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserUploadFileRequestResponseState {
    Approve,
    Existed,
    OSSError,
    DatabaseError,
    FileTooLarge,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserFileUploadedResponse {
    pub upload_id: UploadId,
    pub state: UserFileUploadedState,
    pub url: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserFileUploadedState {
    Success,
    FileHashError,
    FileSizeError,
    NotUploader,
    RequestNotFound,
    DatabaseError,
    ObjectNotFound,
    OSSError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserGetFileUrlResponse {
    pub hash: String,
    pub state: UserGetFilePubUrlState,
    pub url: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]

pub enum UserGetFilePubUrlState {
    Success,
    FileNotExisted,
    OSSError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserRevokeMethod {
    Sender,
    GroupOwner,
    GroupAdmin,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserRevokeMessageData {
    pub chat_id: ChatID,
    pub in_chat_id: MessageID,
    pub method: UserRevokeMethod,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserRevokeMessageResponseState {
    Success,
    TimeLimitExceeded,
    PermissionsDenied,
    DatabaseError,
    MessageNotExisted,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UserRevokeMessageResponse {
    pub chat_id: ChatID,
    pub in_chat_id: MessageID,
    pub state: UserRevokeMessageResponseState,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserGetGroupOwnerResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        user_id: UserID,
    },
    UserNotInChat,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserGetGroupAdminResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        user_ids: Vec<UserID>,
    },
    UserNotInChat,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserMediaCallType {
    Video,
    Voice,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserMediaCallData {
    pub friend_id: UserID,
    pub call_type: UserMediaCallType,
    pub serialized_offer: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserMediaCallResponse {
    Success,
    NotFriend,
    DatabaseError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserMediaCallAnswer {
    pub friend_id: UserID,
    pub accept: bool,
    pub serialized_answer: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum UserMediaCallStopReason {
    Network,
    User,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserMediaCallStop {
    pub friend_id: UserID,
    pub reason: UserMediaCallStopReason,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserMediaIceCandidate {
    pub friend_id: UserID,
    pub serialized_candidate: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserGetUserIDResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        user_ids: Vec<UserID>,
    },
    NotFound,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserGetUserReadInGroupData {
    pub chat_id: ChatID,
    pub in_chat_id: MessageID,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserGetUserReadInGroupResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        in_chat_id: MessageID,
        user_ids: Vec<UserID>,
    },
    NotGroupChat,
    UserNotInChat,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserGetUserReadInPrivateResponse {
    #[serde(rename_all = "camelCase")]
    Success {
        chat_id: ChatID,
        in_chat_id: MessageID,
    },
    NotPrivateChat,
    UserNotInChat,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserSetOppositeReadCursorData {
    pub chat_id: ChatID,
    pub in_chat_id: MessageID,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserLogOffResponse {
    Success,
    NoPermission,
    EmailCodeError,
    UserNotFound,
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MemberChangeData {
    pub r#type: MemberChangeType,
    pub chat_id: ChatID,
    pub user_id: UserID,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum MemberChangeType {
    AddMember,
    DeleteMember,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserApplyForTokenResponse {
    Success { token: String, timestamp: Timestamp },
    DatabaseError,
    ServerError,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RequestMessageResponse {
    pub req_id: UserID,
    pub r#type: RequstMessageType,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum RequstMessageType {
    UserLogOff,
    UserAlreadyInChat,
}
