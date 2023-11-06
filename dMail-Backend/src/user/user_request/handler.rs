/*!
用户请求的处理函数
*/

use crate::{
    chat::{send_admin_message_to_group_chat, ChatMembers},
    config::datatype::{ChatID, UserID},
    database::{self, check_user_is_admin, get_chat_user_list},
    user::GetUserInfoResponse,
    user::{
        send_msg_to_online_handlers,
        user_session::{protocol::ServerToClientMessage, send_request},
        MemberChangeData, MemberChangeType, UserRequestHandler, UserSendRequestData,
        UserSendRequestState,
    },
};

use super::{UserRequestInfo, UserRequsetContent};

/** `on_request_send` 请求发送时的处理函数
*/
pub async fn on_request_send(info: &UserRequestInfo) -> Result<(), ()> {
    let sender_id = info.sender_id;
    match info.content {
        UserRequsetContent::MakeFriend { receiver_id } => {
            on_make_friend_send(sender_id, receiver_id).await
        }
        UserRequsetContent::JoinGroup { chat_id } => on_join_group_send(sender_id, chat_id).await,
        UserRequsetContent::GroupInvitation {
            receiver_id,
            chat_id,
        } => on_group_invitation_send(sender_id, receiver_id, chat_id).await,
        UserRequsetContent::InvitedJoinGroup {
            inviter_id: _,
            chat_id: _,
        } => Ok(()),
    }
}

/** `on_request_approved` 请求同意时的处理函数
*/
pub async fn on_request_approved(info: &UserRequestInfo) -> Result<(), ()> {
    let sender_id = info.sender_id;
    match info.content {
        UserRequsetContent::MakeFriend { receiver_id } => {
            on_make_friend_approved(sender_id, receiver_id).await
        }

        UserRequsetContent::JoinGroup { chat_id } => {
            on_join_group_approved(chat_id, sender_id).await
        }
        UserRequsetContent::GroupInvitation {
            chat_id,
            receiver_id,
        } => on_group_invitation_approved(sender_id, receiver_id, chat_id).await,
        UserRequsetContent::InvitedJoinGroup {
            inviter_id,
            chat_id,
        } => on_invited_join_group_approved(inviter_id, sender_id, chat_id).await,
    }
}

/** `on_request_refused` 请求拒绝时的处理函数
*/
pub async fn on_request_refused(info: &UserRequestInfo) -> Result<(), ()> {
    let sender_id = info.sender_id;
    match info.content {
        UserRequsetContent::MakeFriend { receiver_id } => {
            on_make_friend_refused(sender_id, receiver_id).await
        }
        UserRequsetContent::JoinGroup { chat_id } => {
            on_join_group_refused(sender_id, chat_id).await
        }
        UserRequsetContent::GroupInvitation {
            receiver_id,
            chat_id,
        } => on_group_invitation_refused(sender_id, receiver_id, chat_id).await,
        UserRequsetContent::InvitedJoinGroup {
            inviter_id: _,
            chat_id: _,
        } => Ok(()),
    }
}

async fn on_make_friend_approved(sender_id: UserID, receiver_id: UserID) -> Result<(), ()> {
    let chat_id = database::make_two_users_be_friends(sender_id, receiver_id).await?;

    // let arc = Arc::new(ServerToClientMessage::Chat(info));
    // user_sessions.send_message_to_online(vec![sender_id, receiver_id], arc);
    send_admin_message_to_group_chat(chat_id, "你们成为好友力，开始聊天吧".to_string()).await;

    return Ok(());
}

async fn on_make_friend_refused(sender_id: UserID, receiver_id: UserID) -> Result<(), ()> {
    return database::delete_friend_request_send(sender_id, receiver_id).await;
}

async fn on_join_group_refused(sender_id: UserID, chat_id: ChatID) -> Result<(), ()> {
    return database::delete_join_group_request_send(sender_id, chat_id).await;
}

async fn on_group_invitation_refused(
    inviter_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), ()> {
    return database::delete_invite_request_send(inviter_id, receiver_id, chat_id).await;
}

async fn on_make_friend_send(sender_id: UserID, receiver_id: UserID) -> Result<(), ()> {
    return database::write_friend_request_send(sender_id, receiver_id).await;
}

async fn on_join_group_send(user_id: UserID, chat_id: ChatID) -> Result<(), ()> {
    return database::write_join_group_request_send(user_id, chat_id).await;
}

async fn on_group_invitation_send(
    inviter_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), ()> {
    return database::write_invite_request_send(inviter_id, receiver_id, chat_id).await;
}

async fn on_join_group_approved(chat_id: ChatID, user_id: UserID) -> Result<(), ()> {
    database::delete_join_group_request_send(user_id, chat_id).await?;
    match database::add_user_to_chat(chat_id, user_id).await {
        Ok(()) => {
            let user_name = match database::get_user_info(user_id).await {
                GetUserInfoResponse::Success(user_info) => user_info.user_name,
                _ => return Err(()),
            };
            send_admin_message_to_group_chat(chat_id, format!("{}加入群聊", user_name)).await;
            if let Ok(users) = get_chat_user_list(chat_id).await {
                match users {
                    ChatMembers::Group(group_users) => {
                        let handlers = UserRequestHandler::Group(group_users);
                        let msg = ServerToClientMessage::GroupMemberChange(MemberChangeData {
                            chat_id,
                            user_id,
                            r#type: MemberChangeType::AddMember,
                        });
                        send_msg_to_online_handlers(msg, handlers).await;
                    }
                    ChatMembers::Private(_) => {
                        return Err(());
                    }
                }
            } else {
                return Err(());
            }
            return Ok(());
        }
        Err(()) => return Err(()),
    }
}

async fn on_group_invitation_approved(
    inviter_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), ()> {
    if let Ok(check) = check_user_is_admin(inviter_id, chat_id).await {
        database::delete_invite_request_send(inviter_id, receiver_id, chat_id).await?;
        if check {
            return on_invited_join_group_approved(inviter_id, receiver_id, chat_id).await;
        } else {
            let inviter_name = match database::get_user_info(inviter_id).await {
                GetUserInfoResponse::Success(user_info) => user_info.user_name,
                _ => return Err(()),
            };
            let receiver_name = match database::get_user_info(receiver_id).await {
                GetUserInfoResponse::Success(user_info) => user_info.user_name,
                _ => return Err(()),
            };

            let send_response = send_request(
                receiver_id,
                UserSendRequestData {
                    message: format!(
                        "群成员{}邀请用户{}加入群聊{}",
                        inviter_name, receiver_name, chat_id
                    ),
                    content: UserRequsetContent::InvitedJoinGroup {
                        inviter_id,
                        chat_id,
                    },
                    client_id: 0,
                },
            )
            .await;

            let req_id;
            if let ServerToClientMessage::SendRequestResponse(response) = send_response {
                match response.state {
                    UserSendRequestState::Success => {
                        if response.req_id.is_some() {
                            req_id = response.req_id.unwrap()
                        } else {
                            return Err(());
                        }
                    }
                    _ => return Err(()),
                }
            } else {
                return Err(());
            }
            let store_result = database::store_user_request(receiver_id, req_id).await;
            if store_result.is_err() {
                return Err(());
            }
            let handler = UserRequestHandler::One(receiver_id);
            let serialized_req;
            if let Ok(req) = database::get_user_request(req_id).await {
                if req.is_some() {
                    serialized_req = serde_json::to_string(&req.unwrap()).unwrap();
                } else {
                    return Err(());
                }
            } else {
                return Err(());
            }
            send_msg_to_online_handlers(ServerToClientMessage::Request(serialized_req), handler)
                .await;
        }
    } else {
        return Err(());
    }

    return Ok(());
}

async fn on_invited_join_group_approved(
    inviter_id: UserID,
    user_id: UserID,
    chat_id: ChatID,
) -> Result<(), ()> {
    match database::add_user_to_chat(chat_id, user_id).await {
        Ok(()) => {
            let user_name = match database::get_user_info(user_id).await {
                GetUserInfoResponse::Success(user_info) => user_info.user_name,
                _ => return Err(()),
            };
            let inviter_name = match database::get_user_info(inviter_id).await {
                GetUserInfoResponse::Success(user_info) => user_info.user_name,
                _ => return Err(()),
            };
            send_admin_message_to_group_chat(
                chat_id,
                format!("群成员{}邀请用户{}加入群聊", inviter_name, user_name),
            )
            .await;
            if let Ok(users) = get_chat_user_list(chat_id).await {
                match users {
                    ChatMembers::Group(group_users) => {
                        let handlers = UserRequestHandler::Group(group_users);
                        let msg = ServerToClientMessage::GroupMemberChange(MemberChangeData {
                            chat_id,
                            user_id,
                            r#type: MemberChangeType::AddMember,
                        });
                        send_msg_to_online_handlers(msg, handlers).await;
                    }
                    ChatMembers::Private(_) => {
                        return Err(());
                    }
                }
            } else {
                return Err(());
            }
            return Ok(());
        }
        Err(()) => return Err(()),
    }
}
