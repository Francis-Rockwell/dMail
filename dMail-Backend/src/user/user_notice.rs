use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{
    config::datatype::{ChatID, MessageID, SerializedUserNotice, Timestamp, UserID},
    database,
    server::server_state::{user_sessions, UserSessionGetter},
};

use super::user_session::{protocol::ServerToClientMessage, UserSessionActorMessage};

/** `UserNotice` 在有消息撤回和被提及的时候发送notice
*/
#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(tag = "state")]
pub enum UserNotice {
    #[serde(rename_all = "camelCase")]
    Revoked {
        chat_id: ChatID,
        in_chat_id: MessageID,
        timestamp: Timestamp,
    },
    #[serde(rename_all = "camelCase")]
    Mentioned {
        chat_id: ChatID,
        in_chat_id: MessageID,
        timestamp: Timestamp,
    },
}
/** `send_notice_to_user_in_chat` 向聊天中的所有用户发送notice
*/
pub async fn send_notice_to_user_in_chat(
    chat_id: ChatID,
    serialized: SerializedUserNotice,
    timestamp: Timestamp,
) -> Result<(), ()> {
    let members = match database::get_chat_user_list(chat_id).await {
        Ok(mem) => mem,
        Err(_) => {
            return Err(());
        }
    };

    let arc_msg = Arc::new(ServerToClientMessage::Notice(serialized.clone()));

    match members {
        crate::chat::ChatMembers::Private((id1, id2)) => {
            user_sessions.do_send_message_to(
                id1,
                UserSessionActorMessage::SendServerMessageArc(arc_msg.clone()),
            );
            user_sessions
                .do_send_message_to(id2, UserSessionActorMessage::SendServerMessageArc(arc_msg));

            database::write_user_notice(id1, timestamp, &serialized).await?;
            database::write_user_notice(id2, timestamp, &serialized).await?;
        }
        crate::chat::ChatMembers::Group(users) => {
            user_sessions.send_message_to_online(&users, arc_msg);

            for id in users {
                database::write_user_notice(id, timestamp, &serialized).await?;
            }
        }
    };

    return Ok(());
}

/** `send_notice` 向指定的一系列用户发送notice
*/
pub async fn send_notice(
    users: Vec<UserID>,
    serialized: SerializedUserNotice,
    timestamp: Timestamp,
) -> Result<(), ()> {
    let arc_msg = Arc::new(ServerToClientMessage::Notice(serialized.clone()));
    user_sessions.send_message_to_online(&users, arc_msg);
    for id in users {
        database::write_user_notice(id, timestamp, &serialized).await?;
    }
    return Ok(());
}
