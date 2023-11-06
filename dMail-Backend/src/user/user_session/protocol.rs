/*! 前后端交互协议 */

use actix::{Actor, AsyncContext};
use actix_web_actors::ws::WebsocketContext;
use aes_gcm::Aes128Gcm;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use super::client_message_data::*;
use crate::{
    config::datatype::{
        ChatID, EmailCodeValue, MessageID, SerializedChatInfo, SerializedChatMessage,
        SerializedRequest, SerializedUserNotice, UploadId, UserID,
    },
    utils::aes::AesGcmHelper,
};

// TODO : Use Box to Reduce Message Size
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "command", content = "data")]
/** `ClientToServerMessage` 客户端给服务端所发的消息
 */
pub enum ClientToServerMessage {
    Ping,
    Pong,
    Close,
    SetConnectionPubKey(String),
    Register(UserRegisterData),
    UpdateUserInfo(UserUpdateData),
    UpdateGroupInfo(UserUpdateGroupData),
    ApplyForToken,
    Login(UserLoginData),
    // 增量更新，传入客户端知道的最后一个ChatID, MessageID
    Pull(UserPullData),
    SendMessage(UserSendMessageData),
    SendRequest(UserSendRequestData),
    GetUserInfo(UserID),
    GetChatInfo(ChatID),
    GetGroupUsers(ChatID),
    GetFileUrl(String),
    SolveRequest(UserSolveRequestData),
    RevokeMessage(UserRevokeMessageData),
    GetMessages(UserGetMessagesData),
    CreateGroupChat(UserCreateGroupChatData),
    Unfriend(UserID),
    QuitGroupChat(ChatID),
    SetUserSetting(String),
    SetAlreadyRead(UserSetAlreadyReadData),
    UploadFileRequest(UserUploadFileRequestData),
    FileUploaded(UploadId),
    SetGroupAdmin(UserSetGroupAdminData),
    GroupOwnerTransfer(UserGroupOwnerTransferData),
    SendGroupNotice(UserSendGroupNoticeData),
    PullGroupNotice(UserPullGroupNoticeData),
    RemoveGroupMember(UserRemoveGroupMemberData),
    UnsetGroupAdmin(UserUnsetGroupAdminData),
    GetGroupOwner(ChatID),
    GetGroupAdmin(ChatID),
    MediaCall(UserMediaCallData),
    MediaCallAnswer(UserMediaCallAnswer),
    MediaIceCandidate(UserMediaIceCandidate),
    MediaCallStop(UserMediaCallStop),
    GetUserID(String),
    GetUserReadInGroup(UserGetUserReadInGroupData),
    GetUserReadInPrivate(ChatID),
    LogOff(EmailCodeValue),
}

// TODO : Use Box to Reduce Message Size
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "command", content = "data")]
pub enum ServerToClientMessage {
    Ping,
    Pong,
    Close,
    SetConnectionSymKey(String),
    SetConnectionPubKeyResponse(UserSetPubKeyResponse),
    ApplyForTokenResponse(UserApplyForTokenResponse),
    LoginResponse(UserLoginResponse),
    RegisterResponse(UserRegisterResponse),
    UpdateUserInfoResponse(UserUpdateResponse),
    UpdateGroupInfoResponse(UserUpdateGroupInfoResponse),
    SendMessageResponse(UserSendMessageResponse),
    SendRequestResponse(UserSendRequestResponse),
    GetUserInfoResponse(GetUserInfoResponse),
    GetGroupUsersResponse(UserGetGroupUsersResponse),
    GetFileUrlResponse(UserGetFileUrlResponse),
    SolveRequestResponse(UserSolveRequestResponse),
    CreateGroupChatResponse(UserCreateGroupChatResponse),
    UploadFileRequestResponse(UserUploadFileRequestResponse),
    RevokeMessageResponse(UserRevokeMessageResponse),
    FileUploadedResponse(UserFileUploadedResponse),
    RequestStateUpdate(UserRequsetStateUpdated),
    PullResponse(UserPullResponse),
    Notice(SerializedUserNotice),
    Notices(Vec<SerializedUserNotice>),
    Chat(SerializedChatInfo),
    Chats(Vec<SerializedChatInfo>),
    ReadCursors(Vec<(ChatID, MessageID)>),
    Messages(Vec<SerializedChatMessage>),
    Message(SerializedChatMessage),
    Request(SerializedRequest),
    Requests(Vec<SerializedRequest>),
    UnfriendResponse(UserUnfriendResponse),
    QuitGroupChatResponse(UserQuitGroupChatResponse),
    DeleteChat(ChatID),
    SetUserSettingResponse(SetSettingResponse),
    UserSetting(String),
    DatabaseError,
    NotFound,
    SetAlreadyReadResponse(SetAlreadyReadResponse),
    SetGroupAdminResponse(UserSetGroupAdminResponse),
    GroupOwnerTransferResponse(UserGroupOwnerTransferResponse),
    GroupNoticeResponse(UserSendGroupNoticeResponse),
    PullGroupNoticeResponse(UserPullGroupNoticeResponse),
    RemoveGroupMemberResponse(UserRemoveGroupMemberResponse),
    UnsetGroupAdminResponse(UserUnsetGroupAdminResponse),
    GetGroupOwnerResponse(UserGetGroupOwnerResponse),
    GetGroupAdminResponse(UserGetGroupAdminResponse),
    MediaCallResponse(UserMediaCallResponse),
    MediaCallOffer(UserMediaCallData),
    MediaCallAnswer(UserMediaCallAnswer),
    MediaIceCandidate(UserMediaIceCandidate),
    MediaCallStop(UserMediaCallStop),
    GetUserIDResponse(UserGetUserIDResponse),
    GetUserReadInGroupResponse(UserGetUserReadInGroupResponse),
    GetUserReadInPrivateResponse(UserGetUserReadInPrivateResponse),
    SetOppositeReadCursor(UserSetOppositeReadCursorData),
    LogOffResponse(UserLogOffResponse),
    GroupMemberChange(MemberChangeData),
    RequestMessage(RequestMessageResponse),
}

static SET_PUB_KEY_EQ: Lazy<ClientToServerMessage> =
    Lazy::new(|| ClientToServerMessage::SetConnectionPubKey("".to_string()));

static REGISTER_EQ: Lazy<ClientToServerMessage> = Lazy::new(|| {
    ClientToServerMessage::Register(UserRegisterData {
        user_name: "".to_string(),
        password: "".to_string(),
        email: "".to_string(),
        email_code: 0,
    })
});

static LOGIN_EQ: Lazy<ClientToServerMessage> = Lazy::new(|| {
    ClientToServerMessage::Login(UserLoginData {
        email: "".to_string(),
        password: None,
        email_code: None,
        address: None,
        token: None,
    })
});

// TODO : 使用宏简化代码
impl ClientToServerMessage {
    pub fn is_set_pub_key(&self) -> bool {
        return *self == *SET_PUB_KEY_EQ;
    }

    pub fn is_register(&self) -> bool {
        return *self == *REGISTER_EQ;
    }

    pub fn is_login(&self) -> bool {
        return *self == *LOGIN_EQ;
    }
}

impl PartialEq for ClientToServerMessage {
    fn eq(&self, other: &Self) -> bool {
        return std::mem::discriminant(self) == std::mem::discriminant(other);
    }
}

pub trait ServerMessageSender<A>
where
    A: Actor<Context = Self>,
    Self: AsyncContext<A>,
{
    // TODO : 使用宏简化代码
    fn send_server_message(&mut self, msg: &ServerToClientMessage, cipher: Option<&Aes128Gcm>);
}

impl<A> ServerMessageSender<A> for WebsocketContext<A>
where
    A: Actor<Context = WebsocketContext<A>>,
{
    fn send_server_message(&mut self, msg: &ServerToClientMessage, cipher: Option<&Aes128Gcm>) {
        match msg {
            ServerToClientMessage::Ping => self.ping(&[]),
            ServerToClientMessage::Pong => self.pong(&[]),
            ServerToClientMessage::Close => self.close(None),
            _ => {
                let json = serde_json::to_string::<ServerToClientMessage>(msg).unwrap();
                if let Some(cipher) = cipher {
                    self.text(cipher.encrypt_with_default_nouce_to_base64(&json).unwrap());
                } else {
                    self.text(json);
                }
            }
        }
    }
}

pub trait DataChecker<ErrorType> {
    fn check_data(&self) -> Result<(), ErrorType>;
}
