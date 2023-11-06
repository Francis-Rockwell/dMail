use std::sync::Arc;

use actix::Recipient;

use crate::{
    chat,
    config::{
        datatype::{ChatID, MessageID, SerializedChatMessage, SerializedRequest, UserID},
        Config,
    },
    database::{self},
    server::server_state::{user_sessions, UserSessionGetter},
};

use super::{
    user_session::{protocol::ServerToClientMessage, UserSessionActorMessage},
    UserPullData, UserPullResponse, UserRequestHandler,
};

/** `send_msg_to_online_user_in_private_chat` 向私聊中的两个用户发送消息
*/
pub fn send_msg_to_online_user_in_private_chat(
    sender_id: UserID,
    chat_msg: SerializedChatMessage,
    members: (UserID, UserID),
) {
    let msg = ServerToClientMessage::Message(chat_msg);

    if sender_id == 0 {
        let arc_msg = Arc::new(msg);
        user_sessions.do_send_message_to(
            members.0,
            UserSessionActorMessage::SendServerMessageArc(arc_msg.clone()),
        );
        user_sessions.do_send_message_to(
            members.1,
            UserSessionActorMessage::SendServerMessageArc(arc_msg),
        );
    } else {
        let actor_msg = UserSessionActorMessage::SendServerMessage(msg);
        let receiver = if members.0 == sender_id {
            members.1
        } else {
            members.0
        };
        user_sessions.do_send_message_to(receiver, actor_msg);
    }
}

/** `send_msg_to_online_users_in_chat` 封装向私聊和群聊用户发送消息的两个函数
*/
pub async fn send_msg_to_online_users_in_chat(
    chat_msg: SerializedChatMessage,
    sender_id: UserID,
    chat_id: ChatID,
) {
    let members = match database::get_chat_user_list(chat_id).await {
        Ok(mem) => mem,
        Err(_) => {
            return;
        }
    };

    // TODO : 针对session的特定协议类型进行优化
    // 具体来说，针对使用WebSocket的用户来讲，数据只需要序列化一次，不需要发送过去每次再序列化
    // 同时，使用相对底层的API，可以实现发送数据时不需要clone，每个Actor都发送一块内存上的数据
    // 这里由于协议类型无法确定，这里要实现的话需要修改泛型，手动传入Serialzier

    match members {
        chat::ChatMembers::Private(ids) => {
            send_msg_to_online_user_in_private_chat(sender_id, chat_msg, ids);
        }
        chat::ChatMembers::Group(users_vec) => {
            let arc = Arc::new(ServerToClientMessage::Message(chat_msg));

            user_sessions.send_message_to_online_with_exclusion(users_vec, arc, sender_id);
        }
    }
}

/** `send_msg_to_online_handlers` 向请求的处理者发送消息
*/
pub async fn send_msg_to_online_handlers(msg: ServerToClientMessage, handlers: UserRequestHandler) {
    match handlers {
        UserRequestHandler::One(handler) => {
            user_sessions
                .do_send_message_to(handler, UserSessionActorMessage::SendServerMessage(msg));
        }
        UserRequestHandler::Group(handlers) => {
            let arc = Arc::new(msg);
            user_sessions.send_message_to_online(&handlers, arc);
        }
    }
}

/** `send_delete_chat_msg` 向用户发送消息删除某个聊天
*/
pub async fn send_delete_chat_msg(user_id: UserID, chat_id: ChatID) {
    user_sessions.do_send_message_to(
        user_id,
        UserSessionActorMessage::SendServerMessage(ServerToClientMessage::DeleteChat(chat_id)),
    );
}

/** `user_pull`用户登录时向服务器拉取基本信息
*/
pub async fn user_pull(
    pull_data: UserPullData,
    user_id: UserID,
    receiver: Recipient<UserSessionActorMessage>,
) {
    // 这个操作或许需要一些针对数据库类型的优化

    let chats: Vec<(ChatID, MessageID)> = match database::get_user_chat_list(user_id).await {
        Ok(chats) => chats,
        Err(_) => {
            receiver.do_send(UserSessionActorMessage::SendServerMessage(
                ServerToClientMessage::PullResponse(UserPullResponse::DatabaseError),
            ));
            return;
        }
    };

    let messages: Vec<SerializedChatMessage> = match database::get_chats_last_messages(
        &chats,
        Config::get()
            .protocol
            .max_messages_num_in_one_chat_when_pulling,
    )
    .await
    {
        Ok(msgs) => msgs,
        Err(_) => {
            receiver.do_send(UserSessionActorMessage::SendServerMessage(
                ServerToClientMessage::PullResponse(UserPullResponse::DatabaseError),
            ));
            return;
        }
    };

    receiver.do_send(UserSessionActorMessage::SendServerMessage(
        ServerToClientMessage::ReadCursors(chats),
    ));

    receiver.do_send(UserSessionActorMessage::SendServerMessage(
        ServerToClientMessage::Messages(messages),
    ));

    let requests: Vec<SerializedRequest> =
        match database::get_user_requests(user_id, pull_data.last_request_id).await {
            Ok(reqs) => reqs,
            Err(_) => {
                receiver.do_send(UserSessionActorMessage::SendServerMessage(
                    ServerToClientMessage::PullResponse(UserPullResponse::DatabaseError),
                ));
                return;
            }
        };

    receiver.do_send(UserSessionActorMessage::SendServerMessage(
        ServerToClientMessage::Requests(requests),
    ));

    match database::get_user_setting(user_id).await {
        Ok(setting) => {
            if let Some(setting) = setting {
                receiver.do_send(UserSessionActorMessage::SendServerMessage(
                    ServerToClientMessage::UserSetting(setting),
                ));
            }
        }
        Err(_) => {
            receiver.do_send(UserSessionActorMessage::SendServerMessage(
                ServerToClientMessage::PullResponse(UserPullResponse::DatabaseError),
            ));
            return;
        }
    };

    match database::get_user_notice(user_id, pull_data.notice_timestamp).await {
        Ok(notices) => receiver.do_send(UserSessionActorMessage::SendServerMessage(
            ServerToClientMessage::Notices(notices),
        )),
        Err(_) => {
            receiver.do_send(UserSessionActorMessage::SendServerMessage(
                ServerToClientMessage::PullResponse(UserPullResponse::DatabaseError),
            ));
            return;
        }
    }

    receiver.do_send(UserSessionActorMessage::SendServerMessage(
        ServerToClientMessage::PullResponse(UserPullResponse::Success),
    ))
}
