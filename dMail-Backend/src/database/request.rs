use crate::config::datatype::*;

use crate::user::*;

use super::redis;

/// 获得用户请求
pub async fn get_user_requests(
    user_id: UserID,
    start_req_id: UserReqId,
) -> Result<Vec<SerializedRequest>, ()> {
    return redis::get_user_requests(user_id, start_req_id).await;
}

/// 写入用户请求并分配ID
pub async fn write_user_request(
    sender_id: UserID,
    data: UserSendRequestData,
    handler: &UserRequestHandler,
) -> Result<(SerializedRequest, UserRequestInfo), ()> {
    return redis::write_user_request(sender_id, data, &handler).await;
}

/// 存入UserReqID
pub async fn store_user_request(user_id: UserID, req_id: UserReqId) -> Result<(), ()> {
    return redis::store_user_request(user_id, req_id).await;
}

/// 通过UserReqID获取UserRequest
pub async fn get_user_request(req_id: UserReqId) -> Result<Option<UserRequset>, ()> {
    return redis::get_user_request(req_id).await;
}

/// 设置用户请求状态，只有Unsolved的请求能够被设置状态
pub async fn set_user_request_state(
    req_id: UserReqId,
    state: UserRequestState,
) -> Result<(), UserSolveRequestResponse> {
    return redis::set_user_request_state(req_id, state)
        .await
        .map_err(|state| UserSolveRequestResponse {
            state: state,
            req_id,
        });
}

/// 获取请求的处理者
pub async fn get_handlers_of_request(req: &UserRequsetContent) -> Result<UserRequestHandler, ()> {
    match req {
        UserRequsetContent::MakeFriend { receiver_id } => {
            return Ok(UserRequestHandler::One(*receiver_id));
        }
        UserRequsetContent::JoinGroup { chat_id } => {
            return redis::get_chat_admins_list(*chat_id).await;
        }
        UserRequsetContent::GroupInvitation {
            chat_id: _,
            receiver_id,
        } => return Ok(UserRequestHandler::One(*receiver_id)),
        UserRequsetContent::InvitedJoinGroup {
            inviter_id: _,
            chat_id,
        } => {
            return redis::get_chat_admins_list(*chat_id).await;
        }
    }
}

/// 在好友申请发送时为两个用户建立id=0的Chat
pub async fn write_friend_request_send(user_one_id: UserID, user_two_id: UserID) -> Result<(), ()> {
    return redis::write_friend_request_send(user_one_id, user_two_id).await;
}

/// 在好友申请被处理时删除两个用户间id=0的Chat
pub async fn delete_friend_request_send(
    user_one_id: UserID,
    user_two_id: UserID,
) -> Result<(), ()> {
    return redis::delete_friend_request_send(user_one_id, user_two_id).await;
}

/// 在申请加群时将群聊id加入用户的pre_join
pub async fn write_join_group_request_send(user_id: UserID, chat_id: ChatID) -> Result<(), ()> {
    return redis::write_join_group_request_send(user_id, chat_id).await;
}

/// 在申请加群时将群聊id移出用户的pre_join
pub async fn delete_join_group_request_send(user_id: UserID, chat_id: ChatID) -> Result<(), ()> {
    return redis::delete_join_group_request_send(user_id, chat_id).await;
}

/// 在加群邀请发送时增添Invitations哈希表键
pub async fn write_invite_request_send(
    inviter_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), ()> {
    return redis::write_invite_request_send(inviter_id, receiver_id, chat_id).await;
}

/// 在加群邀请被处理时删除Invitations的对应键
pub async fn delete_invite_request_send(
    inviter_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), ()> {
    return redis::delete_invite_request_send(inviter_id, receiver_id, chat_id).await;
}
