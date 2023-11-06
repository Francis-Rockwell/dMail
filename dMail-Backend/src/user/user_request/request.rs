/*!
 用户请求有关的数据类型
*/

use serde::{Deserialize, Serialize};

use crate::config::datatype::{ChatID, UserID, UserReqId};

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum UserRequsetContent {
    #[serde(rename_all = "camelCase")]
    MakeFriend { receiver_id: UserID },
    #[serde(rename_all = "camelCase")]
    JoinGroup { chat_id: ChatID },
    #[serde(rename_all = "camelCase")]
    GroupInvitation {
        receiver_id: UserID,
        chat_id: ChatID,
    },
    #[serde(rename_all = "camelCase")]
    InvitedJoinGroup { inviter_id: UserID, chat_id: ChatID },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UserRequestInfo {
    pub req_id: UserReqId,
    pub sender_id: UserID,
    pub message: String,
    pub content: UserRequsetContent,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "errorType")]
pub enum MakeFriendError {
    AlreadyBeFrineds,
    RequestExisted,
    SameUser,
    DatabaseError,
    UserNotFound,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "errorType")]
pub enum JoinGroupError {
    AlreadyInGroup,
    DatabaseError,
    RequestExisted,
    UserNotFound,
    NotGroupChat,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "errorType")]
pub enum GroupInvitationError {
    UserNotInChat,
    AlreadyInGroup,
    NotFriend,
    SameUser,
    DatabaseError,
    UserNotFound,
    RequestExist,
    NotGroupChat,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "errorType")]
pub enum InvitedJoinGroupError {
    UserNotInChat,
    AlreadyInGroup,
    SameUser,
    DatabaseError,
    UserNotFound,
    NotGroupChat,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum UserRequestError {
    MakeFriend(MakeFriendError),
    JoinGroup(JoinGroupError),
    GroupInvation(GroupInvitationError),
    InvitedJoinGroup(InvitedJoinGroupError),
    DatabaseError,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
pub enum UserRequestState {
    Unsolved,
    Refused,
    Approved,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum UserRequestHandler {
    One(UserID),
    Group(Vec<UserID>),
}

impl UserRequestHandler {
    pub fn is_handler(&self, user_id: UserID) -> bool {
        match self {
            UserRequestHandler::One(id) => return *id == user_id,
            UserRequestHandler::Group(ids) => {
                for id in ids {
                    if *id == user_id {
                        return true;
                    }
                }
                return false;
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserRequset {
    pub info: UserRequestInfo,
    pub state: UserRequestState,
}
