use std::ops::DerefMut;

use mobc_redis::redis;
use mobc_redis::redis::AsyncCommands;

use super::common::*;
use super::index;
use super::path;
use crate::chat::ChatInfo;
use crate::chat::ChatMembers;
use crate::chat::ChatType;
use crate::config::datatype::ChatID;
use crate::config::datatype::ClientID;
use crate::config::datatype::MessageID;
use crate::config::datatype::NoticeID;
use crate::config::datatype::SerializedChatInfo;
use crate::config::datatype::SerializedChatMessage;
use crate::config::datatype::SerializedGroupNotice;
use crate::config::datatype::Timestamp;
use crate::database::check_user_exist;
use crate::{config::datatype::UserID, user::*};

pub async fn create_group_chat(
    creator: UserID,
    data: UserCreateGroupChatData,
) -> Result<ChatID, ()> {
    let mut con = get_con().await?;

    let chat_id: ChatID = con.incr(path::LAST_CHAT_ID, 1).await.map_err(|_| ())?;
    let serialized_chat_info = format!(
        r#"{{"id":{},"name":{},"avaterHash":{}}}"#,
        chat_id,
        serde_json::to_string::<String>(&data.name).unwrap(),
        serde_json::to_string::<String>(&data.avater_hash).unwrap(),
    );

    // Pika 不支持Multi开始事务，因此这里也不使用
    redis::pipe()
        .set(
            index::get_chat_info_index(chat_id).as_str(),
            serialized_chat_info,
        )
        .ignore()
        .set(index::get_chat_owner_index(chat_id).as_str(), creator)
        .ignore()
        .sadd(index::get_chat_admins_index(chat_id).as_str(), creator)
        .ignore()
        .sadd(index::get_chat_users_index(chat_id).as_str(), creator)
        .ignore()
        .hset(index::get_user_chats_index(creator).as_str(), chat_id, 0)
        .query_async::<_, ()>(con.deref_mut())
        .await
        .map_err(|_| ())?;

    return Ok(chat_id);
}

pub async fn add_user_to_group_chat(chat_id: ChatID, user_id: UserID) -> Result<(), ()> {
    let mut con = get_con().await?;

    redis::pipe()
        .sadd(index::get_chat_users_index(chat_id).as_str(), user_id)
        .ignore()
        .hset(index::get_user_chats_index(user_id).as_str(), chat_id, 0)
        .ignore()
        .query_async(con.deref_mut())
        .await
        .map_err(|_| ())?;

    return Ok(());
}

pub async fn write_message_to_chat(
    r#type: &str,
    serialized_content: String,
    chat_id: ChatID,
    sender_id: UserID,
) -> Result<(SerializedChatMessage, MessageID, Timestamp), ()> {
    let mut con = get_con().await?;

    let chat_index = index::get_chat_msgs_index(chat_id);
    let timestamp = chrono::Utc::now().timestamp_millis() as Timestamp;

    let in_chat_id: MessageID = con
        .incr(index::get_chat_last_id_index(chat_id).as_str(), 1)
        .await
        .map_err(|_| ())?;

    let serialized_msg = format!(
        r#"{{"type":{}, "inChatId":{}, "chatId":{}, "senderId":{}, "serializedContent":{}, "timestamp":{}}}"#,
        r#type, in_chat_id, chat_id, sender_id, serialized_content, timestamp
    );

    // TODO : 实现消息列表分块存储
    con.zadd(chat_index.as_str(), &serialized_msg, in_chat_id)
        .await
        .map_err(|_| ())?;

    return Ok((serialized_msg, in_chat_id, timestamp));
}

pub async fn get_chat_user_list(chat_id: ChatID) -> Result<ChatMembers, ()> {
    let mut con = get_con().await?;

    let is_group: bool = con
        .exists(index::get_chat_owner_index(chat_id).as_str())
        .await
        .map_err(|_| ())?;

    if is_group {
        let users: Vec<UserID> = con
            .smembers(index::get_chat_users_index(chat_id).as_str())
            .await
            .map_err(|_| ())?;
        return Ok(ChatMembers::Group(users));
    } else {
        // "{id1}:{id2}"
        let pair: (UserID, UserID) = redis::pipe()
            .get(index::get_chat_user_index(chat_id, 0).as_str())
            .get(index::get_chat_user_index(chat_id, 1).as_str())
            .query_async(con.deref_mut())
            .await
            .map_err(|_| ())?;

        return Ok(ChatMembers::Private(pair));
    }
}

pub async fn get_private_chat_user_list(chat_id: ChatID) -> Result<Option<(UserID, UserID)>, ()> {
    let mut con = get_con().await?;

    let is_group: bool = con
        .exists(index::get_chat_owner_index(chat_id).as_str())
        .await
        .map_err(|_| ())?;

    if is_group {
        return Ok(None);
    } else {
        // "{id1}:{id2}"
        let pair: (UserID, UserID) = redis::pipe()
            .get(index::get_chat_user_index(chat_id, 0).as_str())
            .get(index::get_chat_user_index(chat_id, 1).as_str())
            .query_async(con.deref_mut())
            .await
            .map_err(|_| ())?;

        return Ok(Some(pair));
    }
}

pub async fn check_user_can_send_in_chat(
    user_id: UserID,
    chat_id: ChatID,
) -> Result<ChatType, UserSendMessageResponseState> {
    let mut con = get_con()
        .await
        .map_err(|_| UserSendMessageResponseState::DatabaseError)?;

    let chat_existed: bool = con
        .exists(index::get_chat_info_index(chat_id).as_str())
        .await
        .map_err(|_| UserSendMessageResponseState::DatabaseError)?;

    if chat_existed == false {
        return Err(UserSendMessageResponseState::ChatNotFound);
    }

    let is_group: bool = con
        .exists(index::get_chat_owner_index(chat_id).as_str())
        .await
        .map_err(|_| UserSendMessageResponseState::DatabaseError)?;

    if is_group {
        let chat_index = index::get_chat_users_index(chat_id);
        let (user_in_chat, group_user_num): (bool, UserID) = redis::pipe()
            .sismember(chat_index.as_str(), user_id)
            .scard(chat_index.as_str())
            .query_async(con.deref_mut())
            .await
            .map_err(|_| UserSendMessageResponseState::DatabaseError)?;

        if user_in_chat == false && user_id != 0 {
            return Err(UserSendMessageResponseState::UserNotInChat);
        }
        return Ok(ChatType::Group(group_user_num));
    } else {
        let pair: (Option<UserID>, Option<UserID>) = redis::pipe()
            .get(index::get_chat_user_index(chat_id, 0).as_str())
            .get(index::get_chat_user_index(chat_id, 1).as_str())
            .query_async(con.deref_mut())
            .await
            .map_err(|_| UserSendMessageResponseState::DatabaseError)?;
        if pair.0.is_none() || pair.1.is_none() {
            return Err(UserSendMessageResponseState::UserNotInChat);
        }
        if user_id == pair.0.unwrap() || user_id == pair.1.unwrap() || user_id == 0 {
            return Ok(ChatType::Private((pair.0.unwrap(), pair.1.unwrap())));
        } else {
            return Err(UserSendMessageResponseState::UserNotInChat);
        }
    }
}

pub async fn get_chats_last_messages(
    chats_id: &Vec<(ChatID, MessageID)>,
    max_messages_per_chat: u8,
) -> Result<Vec<SerializedChatMessage>, ()> {
    let mut con = get_con().await?;

    let mut ret: Vec<SerializedChatMessage> = Vec::new();

    for (chat_id, _) in chats_id {
        let index = index::get_chat_msgs_index(*chat_id);

        let end_msg_id: isize = con.zcard(index.as_str()).await.map_err(|_| ())?;

        let mut msgs: Vec<SerializedChatMessage> = con
            .zrange(
                index.as_str(),
                std::cmp::max(1, end_msg_id - max_messages_per_chat as isize) - 1,
                end_msg_id - 1,
            )
            .await
            .map_err(|_| ())?;

        ret.append(&mut msgs);
    }

    return Ok(ret);
}

pub async fn get_messages_in_chat(
    chat_id: ChatID,
    start_msg_id: MessageID,
    end_msg_id_opt: Option<MessageID>,
) -> Result<Vec<SerializedChatMessage>, ()> {
    let mut con = get_con().await?;

    let index = index::get_chat_msgs_index(chat_id);

    let end_msg_id = match end_msg_id_opt {
        Some(id) => id,
        None => con.zcard(index.as_str()).await.map_err(|_| ())?,
    };

    let msgs: Vec<SerializedChatMessage> = con
        .zrange(
            index.as_str(),
            start_msg_id as isize - 1,
            end_msg_id as isize - 1,
        )
        .await
        .map_err(|_| ())?;

    return Ok(msgs);
}

pub async fn get_chat_info(chat_id: ChatID) -> Result<Option<SerializedChatInfo>, ()> {
    let mut con = get_con().await?;

    let chat_info: Option<String> = con
        .get(index::get_chat_info_index(chat_id).as_str())
        .await
        .map_err(|_| ())?;

    return Ok(chat_info);
}

pub async fn revoke_message(
    chat_id: ChatID,
    in_chat_id: MessageID,
    sender_id: UserID,
    timestamp: Timestamp,
) -> Result<(), ()> {
    let mut con = get_con().await?;

    let chat_index = index::get_chat_msgs_index(chat_id);
    let serialized_msg = format!(
        r#"{{"type":{}, "inChatId":{}, "chatId":{}, "senderId":{}, "serializedContent":"\"\"", "timestamp":{}}}"#,
        ChatMessageType::Revoked.get_str(),
        in_chat_id,
        chat_id,
        sender_id,
        timestamp,
    );

    redis::pipe()
        .zrembyscore(chat_index.as_str(), in_chat_id, in_chat_id)
        .ignore()
        .zadd(chat_index.as_str(), &serialized_msg, in_chat_id)
        .ignore()
        .query_async(con.deref_mut())
        .await
        .map_err(|_| ())?;

    return Ok(());
}

pub async fn check_group_invitation_error(
    sender_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), GroupInvitationError> {
    if let Ok(mut con) = get_con().await {
        let last_user_id: UserID = con
            .get(path::LAST_USER_ID)
            .await
            .map_err(|_| GroupInvitationError::DatabaseError)?;
        if sender_id > last_user_id || receiver_id > last_user_id {
            return Err(GroupInvitationError::UserNotFound);
        }

        if check_user_exist(sender_id).await.is_err()
            || check_user_exist(receiver_id).await.is_err()
        {
            return Err(GroupInvitationError::UserNotFound);
        }

        con.hexists(index::get_user_chats_index(sender_id).as_str(), chat_id)
            .await
            .map_err(|_| GroupInvitationError::UserNotInChat)?;

        let check_result = check_is_group_chat(chat_id).await;
        if check_result.is_err() || !check_result.unwrap() {
            return Err(GroupInvitationError::NotGroupChat);
        }
        let (id1, id2) = if sender_id < receiver_id {
            (sender_id, receiver_id)
        } else {
            (receiver_id, sender_id)
        };
        con.hexists(
            path::FRIEND_CHAT_MAP,
            index::get_friend_pair_index(id1, id2).as_str(),
        )
        .await
        .map_err(|_| GroupInvitationError::NotFriend)?;

        let already_in_group: bool = con
            .hexists(index::get_user_chats_index(receiver_id).as_str(), chat_id)
            .await
            .map_err(|_| GroupInvitationError::DatabaseError)?;
        let req_exist: bool = con
            .hexists(
                path::INVITAION_MAP,
                format!("{}:{}:{}", sender_id, receiver_id, chat_id,),
            )
            .await
            .map_err(|_| GroupInvitationError::DatabaseError)?;
        if already_in_group {
            return Err(GroupInvitationError::AlreadyInGroup);
        } else if req_exist {
            return Err(GroupInvitationError::RequestExist);
        } else {
            return Ok(());
        }
    } else {
        return Err(GroupInvitationError::DatabaseError);
    }
}

pub async fn check_invited_join_group_error(
    sender_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), InvitedJoinGroupError> {
    if let Ok(mut con) = get_con().await {
        let last_user_id: UserID = con
            .get(path::LAST_USER_ID)
            .await
            .map_err(|_| InvitedJoinGroupError::DatabaseError)?;
        if sender_id > last_user_id || receiver_id > last_user_id {
            return Err(InvitedJoinGroupError::UserNotFound);
        }

        if check_user_exist(sender_id).await.is_err()
            || check_user_exist(receiver_id).await.is_err()
        {
            return Err(InvitedJoinGroupError::UserNotFound);
        }

        con.hexists(index::get_user_chats_index(sender_id).as_str(), chat_id)
            .await
            .map_err(|_| InvitedJoinGroupError::UserNotInChat)?;

        let already_in_group: bool = con
            .hexists(index::get_user_chats_index(receiver_id).as_str(), chat_id)
            .await
            .map_err(|_| InvitedJoinGroupError::AlreadyInGroup)?;
        let check_result = check_is_group_chat(chat_id).await;
        if check_result.is_err() || !check_result.unwrap() {
            return Err(InvitedJoinGroupError::NotGroupChat);
        }
        if already_in_group {
            return Err(InvitedJoinGroupError::AlreadyInGroup);
        } else {
            return Ok(());
        }
    } else {
        return Err(InvitedJoinGroupError::DatabaseError);
    }
}

pub async fn quit_group_chat(user_id: UserID, chat_id: ChatID) -> UserQuitGroupChatResponse {
    if let Ok(mut con) = get_con().await {
        let chat_remove_user: Result<bool, ()> = con
            .srem(index::get_chat_users_index(chat_id).as_str(), user_id)
            .await
            .map_err(|_| ());
        if chat_remove_user.is_err() {
            return UserQuitGroupChatResponse::DatabaseError;
        } else if !chat_remove_user.unwrap() {
            return UserQuitGroupChatResponse::UserNotInChat;
        }

        let user_remove_chat: Result<bool, ()> = con
            .hdel(index::get_user_chats_index(user_id).as_str(), chat_id)
            .await
            .map_err(|_| ());
        if user_remove_chat.is_err() {
            return UserQuitGroupChatResponse::DatabaseError;
        } else if !user_remove_chat.unwrap() {
            return UserQuitGroupChatResponse::UserNotInChat;
        }

        if let Ok(check) = check_user_is_admin(user_id, chat_id).await {
            if check {
                let chat_remove_admin: Result<bool, ()> = con
                    .srem(index::get_chat_admins_index(chat_id).as_str(), user_id)
                    .await
                    .map_err(|_| ());
                if chat_remove_admin.is_err() || !chat_remove_admin.unwrap() {
                    return UserQuitGroupChatResponse::DatabaseError;
                }
            }
        } else {
            return UserQuitGroupChatResponse::DatabaseError;
        }

        return UserQuitGroupChatResponse::Success { chat_id };
    } else {
        return UserQuitGroupChatResponse::ServerError;
    }
}

pub async fn set_as_admin(user_id: UserID, chat_id: ChatID) -> UserSetGroupAdminResponse {
    if let Ok(mut con) = get_con().await {
        let add_result: Result<bool, ()> = con
            .sadd(index::get_chat_admins_index(chat_id).as_str(), user_id)
            .await
            .map_err(|_| ());
        if add_result.is_ok() && add_result.unwrap() {
            return UserSetGroupAdminResponse::Success { chat_id, user_id };
        } else {
            return UserSetGroupAdminResponse::DatabaseError;
        }
    } else {
        return UserSetGroupAdminResponse::ServerError;
    }
}

pub async fn check_join_group_error(
    user_id: UserID,
    chat_id: ChatID,
) -> Result<(), JoinGroupError> {
    if let Ok(mut con) = get_con().await {
        if check_user_exist(user_id).await.is_err() {
            return Err(JoinGroupError::UserNotFound);
        }
        let check_result = check_is_group_chat(chat_id).await;
        if check_result.is_err() || !check_result.unwrap() {
            return Err(JoinGroupError::NotGroupChat);
        }
        let already_in_group: bool = con
            .hexists(index::get_user_chats_index(user_id).as_str(), chat_id)
            .await
            .map_err(|_| JoinGroupError::DatabaseError)?;
        let req_exist: bool = con
            .sismember(index::get_user_pre_join_index(user_id).as_str(), chat_id)
            .await
            .map_err(|_| JoinGroupError::DatabaseError)?;
        if already_in_group {
            return Err(JoinGroupError::AlreadyInGroup);
        } else if req_exist {
            return Err(JoinGroupError::RequestExisted);
        } else {
            return Ok(());
        }
    } else {
        return Err(JoinGroupError::DatabaseError);
    }
}

pub async fn get_chat_admins_list(chat_id: ChatID) -> Result<UserRequestHandler, ()> {
    let mut con = get_con().await?;

    let is_group: bool = con
        .exists(index::get_chat_owner_index(chat_id).as_str())
        .await
        .map_err(|_| ())?;

    if is_group {
        let admins: Vec<UserID> = con
            .smembers(index::get_chat_admins_index(chat_id).as_str())
            .await
            .map_err(|_| ())?;
        return Ok(UserRequestHandler::Group(admins));
    } else {
        return Err(());
    }
}

pub async fn get_chat_owner(chat_id: ChatID) -> Result<UserID, ()> {
    let mut con = get_con().await?;

    let is_group: bool = con
        .exists(index::get_chat_owner_index(chat_id).as_str())
        .await
        .map_err(|_| ())?;

    if is_group {
        return con
            .get(index::get_chat_owner_index(chat_id).as_str())
            .await
            .map_err(|_| ());
    } else {
        return Err(());
    }
}

pub async fn check_user_is_owner(user_id: UserID, chat_id: ChatID) -> Result<bool, ()> {
    let owner = get_chat_owner(chat_id).await?;
    return Ok(owner == user_id);
}

pub async fn check_user_is_admin(user_id: UserID, chat_id: ChatID) -> Result<bool, ()> {
    let mut con = get_con().await?;
    return con
        .sismember(index::get_chat_admins_index(chat_id).as_str(), user_id)
        .await
        .map_err(|_| ());
}

pub async fn owner_transfer(user_id: UserID, chat_id: ChatID) -> UserGroupOwnerTransferResponse {
    if let Ok(mut con) = get_con().await {
        let set_result: Result<bool, ()> = con
            .set(index::get_chat_owner_index(chat_id).as_str(), user_id)
            .await
            .map_err(|_| ());
        if set_result.is_ok() && set_result.unwrap() {
            if let Ok(check) = check_user_is_admin(user_id, chat_id).await {
                if !check {
                    if let UserSetGroupAdminResponse::Success { .. } =
                        set_as_admin(user_id, chat_id).await
                    {
                    } else {
                        return UserGroupOwnerTransferResponse::DatabaseError;
                    }
                }
            } else {
                return UserGroupOwnerTransferResponse::DatabaseError;
            }
            return UserGroupOwnerTransferResponse::Success { chat_id, user_id };
        } else {
            return UserGroupOwnerTransferResponse::ServerError;
        }
    } else {
        return UserGroupOwnerTransferResponse::ServerError;
    }
}

pub async fn add_group_notice(
    user_id: UserID,
    chat_id: ChatID,
    client_id: ClientID,
    notice: String,
) -> UserSendGroupNoticeResponse {
    if let Ok(mut con) = get_con().await {
        let timestamp = chrono::Utc::now().timestamp_millis() as Timestamp;
        let notice_id: NoticeID = match con
            .incr(index::get_chat_last_notice_id_index(chat_id).as_str(), 1)
            .await
            .map_err(|_| ())
        {
            Ok(notice_id) => notice_id,
            Err(_) => return UserSendGroupNoticeResponse::DatabaseError,
        };
        let serialized_notice = format!(
            r#"{{"chatId":{}, "noticeId":{}, "senderId":{}, "content":{}, "timestamp":{}}}"#,
            chat_id,
            notice_id,
            user_id,
            serde_json::to_string::<String>(&notice).unwrap(),
            timestamp
        );

        let add_result: Result<bool, ()> = con
            .zadd(
                index::get_chat_notice_index(chat_id).as_str(),
                serialized_notice,
                notice_id,
            )
            .await
            .map_err(|_| ());
        if add_result.is_err() || !add_result.unwrap() {
            return UserSendGroupNoticeResponse::DatabaseError;
        }
        return UserSendGroupNoticeResponse::Success {
            chat_id,
            client_id,
            notice_id,
            timestamp,
        };
    } else {
        return UserSendGroupNoticeResponse::ServerError;
    }
}

pub async fn pull_group_notice(
    chat_id: ChatID,
    last_notice_id: NoticeID,
) -> UserPullGroupNoticeResponse {
    if let Ok(mut con) = get_con().await {
        let index = index::get_chat_notice_index(chat_id);

        let end_id: NoticeID = match con.zcard(index.as_str()).await {
            Ok(id) => id,
            Err(_) => return UserPullGroupNoticeResponse::DatabaseError,
        };

        let group_notice: Vec<SerializedGroupNotice> = match con
            .zrange(index.as_str(), last_notice_id as isize, end_id as isize - 1)
            .await
        {
            Ok(notices) => notices,
            Err(_) => {
                return UserPullGroupNoticeResponse::DatabaseError;
            }
        };

        return UserPullGroupNoticeResponse::Success {
            group_notice,
            chat_id,
        };
    } else {
        return UserPullGroupNoticeResponse::ServerError;
    }
}

pub async fn update_group_info(
    chat_id: ChatID,
    data: UserUpdateGroupContent,
) -> UserUpdateGroupInfoResponse {
    if let Ok(mut con) = get_con().await {
        let old_info = match get_chat_info(chat_id).await {
            Ok(info) => match info {
                Some(info) => info,
                None => {
                    return UserUpdateGroupInfoResponse::DatabaseError;
                }
            },
            Err(_) => {
                return UserUpdateGroupInfoResponse::DatabaseError;
            }
        };
        let old_info = serde_json::from_str::<ChatInfo>(&old_info).unwrap();

        let new_info = match data {
            UserUpdateGroupContent::GroupName { new_name } => ChatInfo {
                name: new_name,
                ..old_info
            },
            UserUpdateGroupContent::Avater { new_avater } => ChatInfo {
                avater_hash: new_avater,
                ..old_info
            },
        };

        let new_info = serde_json::to_string::<ChatInfo>(&new_info).unwrap();

        let set_result: Result<bool, ()> = con
            .set(index::get_chat_info_index(chat_id).as_str(), new_info)
            .await
            .map_err(|_| ());
        if set_result.is_err() || !set_result.unwrap() {
            return UserUpdateGroupInfoResponse::DatabaseError;
        }
        return UserUpdateGroupInfoResponse::Success;
    } else {
        return UserUpdateGroupInfoResponse::ServerError;
    }
}

pub async fn unset_admin(user_id: UserID, chat_id: ChatID) -> UserUnsetGroupAdminResponse {
    if let Ok(mut con) = get_con().await {
        let rem_result: Result<bool, ()> = con
            .srem(index::get_chat_admins_index(chat_id).as_str(), user_id)
            .await
            .map_err(|_| ());
        if rem_result.is_ok() && rem_result.unwrap() {
            return UserUnsetGroupAdminResponse::Success { chat_id, user_id };
        } else {
            return UserUnsetGroupAdminResponse::DatabaseError;
        }
    } else {
        return UserUnsetGroupAdminResponse::ServerError;
    }
}

pub async fn check_is_group_chat(chat_id: ChatID) -> Result<bool, ()> {
    let mut con = get_con().await?;
    con.exists(index::get_chat_owner_index(chat_id).as_str())
        .await
        .map_err(|_| ())
}

pub async fn get_user_read_in_group(
    chat_id: ChatID,
    in_chat_id: MessageID,
) -> UserGetUserReadInGroupResponse {
    if let Ok(mut con) = get_con().await {
        if let Ok(users) = get_chat_user_list(chat_id).await {
            match users {
                ChatMembers::Group(user_ids) => {
                    let mut reads = vec![];
                    for user_id in user_ids {
                        let get_result: Result<MessageID, ()> = con
                            .hget(index::get_user_chats_index(user_id).as_str(), chat_id)
                            .await
                            .map_err(|_| ());
                        match get_result {
                            Ok(msg_id) => {
                                if msg_id >= in_chat_id {
                                    reads.push(user_id);
                                }
                            }
                            Err(_) => {
                                return UserGetUserReadInGroupResponse::DatabaseError;
                            }
                        }
                    }
                    return UserGetUserReadInGroupResponse::Success {
                        user_ids: reads,
                        chat_id,
                        in_chat_id,
                    };
                }
                ChatMembers::Private(_) => return UserGetUserReadInGroupResponse::NotGroupChat,
            }
        } else {
            return UserGetUserReadInGroupResponse::DatabaseError;
        }
    } else {
        return UserGetUserReadInGroupResponse::ServerError;
    }
}

pub async fn get_user_read_in_private(
    user_id: UserID,
    chat_id: ChatID,
) -> UserGetUserReadInPrivateResponse {
    if let Ok(mut con) = get_con().await {
        let users;
        match get_private_chat_user_list(chat_id).await {
            Ok(users_opt) => {
                if users_opt.is_none() {
                    return UserGetUserReadInPrivateResponse::DatabaseError;
                };
                users = users_opt.unwrap();
            }
            Err(()) => return UserGetUserReadInPrivateResponse::DatabaseError,
        }

        let friend_id;
        if user_id == users.0 {
            friend_id = users.1;
        } else if user_id == users.1 {
            friend_id = users.0;
        } else {
            return UserGetUserReadInPrivateResponse::UserNotInChat;
        }

        let get_result: Result<MessageID, ()> = con
            .hget(index::get_user_chats_index(friend_id).as_str(), chat_id)
            .await
            .map_err(|_| ());
        match get_result {
            Ok(msg_id) => {
                return UserGetUserReadInPrivateResponse::Success {
                    chat_id,
                    in_chat_id: msg_id,
                }
            }
            Err(_) => {
                return UserGetUserReadInPrivateResponse::DatabaseError;
            }
        }
    } else {
        return UserGetUserReadInPrivateResponse::ServerError;
    }
}
