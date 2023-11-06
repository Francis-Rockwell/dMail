use std::ops::DerefMut;

use chrono::Utc;
use log::debug;
use mobc_redis::redis;
use mobc_redis::redis::AsyncCommands;
use uuid::Uuid;

use super::check_user_is_owner;
use super::common::*;
use super::get_chat_user_list;
use super::index;
use super::path;
use super::quit_group_chat;

use crate::chat::ChatMembers;
use crate::config::datatype::ChatID;
use crate::config::datatype::MessageID;
use crate::config::datatype::SerializedUserNotice;
use crate::config::datatype::Timestamp;
use crate::config::Config;

use crate::{config::datatype::UserID, user::*};

pub async fn user_register(data: UserRegisterData) -> Result<UserRegisterResponse, ()> {
    let mut con = get_con().await?;

    let email_existed: bool = con
        .hexists(path::USER_EMAIL_MAP, &data.email)
        .await
        .map_err(|_| ())?;

    if email_existed {
        return Ok(UserRegisterResponse::EmailRegistered);
    }

    let user_id: UserID = con.incr(path::LAST_USER_ID, 1).await.map_err(|_| ())?;

    let info_serialized = format!(
        r#"{{"userId":{},"userName":"{}","avaterHash":""}}"#,
        user_id, data.user_name
    );

    redis::pipe()
        .set(
            index::get_user_info_index(user_id).as_str(),
            info_serialized,
        )
        .ignore()
        .set(
            index::get_user_password_index(user_id).as_str(),
            data.password,
        )
        .ignore()
        .set(index::get_user_exist_index(user_id).as_str(), 1)
        .ignore()
        .set(index::get_user_email_index(user_id).as_str(), &data.email)
        .ignore()
        .hset(path::USER_EMAIL_MAP, data.email, user_id)
        .query_async::<_, ()>(con.deref_mut())
        .await
        .map_err(|_| ())?;

    add_user_name_id(&data.user_name, user_id).await?;
    return Ok(UserRegisterResponse::Success { user_id });
}

pub async fn user_login_with_password(data: &UserLoginData) -> Result<UserLoginResponse, ()> {
    let mut con = get_con().await?;

    let user_id_opt: Option<UserID> = con
        .hget(path::USER_EMAIL_MAP, &data.email)
        .await
        .map_err(|_| ())?;

    let user_id = match user_id_opt {
        Some(id) => id,
        None => return Ok(UserLoginResponse::UserNotFound),
    };

    let password: String = con
        .get(index::get_user_password_index(user_id).as_str())
        .await
        .map_err(|_| ())?;

    if &password != data.password.as_ref().unwrap() {
        return Ok(UserLoginResponse::PasswordError);
    }

    return Ok(UserLoginResponse::Success { user_id });
}

pub async fn get_user_id_by_email(email: &String) -> Result<Option<UserID>, ()> {
    let mut con = get_con().await?;

    Ok(con
        .hget(path::USER_EMAIL_MAP, &email)
        .await
        .map_err(|_| ())?)
}

pub async fn get_user_chat_list(user_id: UserID) -> Result<Vec<(ChatID, MessageID)>, ()> {
    let mut con = get_con().await?;

    Ok(con
        .hgetall(index::get_user_chats_index(user_id).as_str())
        .await
        .map_err(|_| ())?)
}

pub async fn get_user_info(user_id: UserID) -> GetUserInfoResponse {
    if let Ok(mut con) = get_con().await {
        let user_info: Result<String, ()> = con
            .get(index::get_user_info_index(user_id).as_str())
            .await
            .map_err(|_| ());
        if let Ok(user_info) = user_info {
            debug!("get_user_info: {}", user_info);
            return GetUserInfoResponse::Success(
                serde_json::from_str::<UserInfo>(&user_info).unwrap(),
            );
        } else {
            return GetUserInfoResponse::UserNotFound;
        }
    } else {
        return GetUserInfoResponse::ServerError;
    }
}

pub async fn get_user_email(user_id: UserID) -> Result<String, ()> {
    if let Ok(mut con) = get_con().await {
        let user_email: Result<String, ()> = con
            .get(index::get_user_email_index(user_id).as_str())
            .await
            .map_err(|_| ());
        if let Ok(user_email) = user_email {
            debug!("get_user_email: {}", user_email);
            return Ok(user_email);
        } else {
            return Err(());
        }
    } else {
        return Err(());
    }
}

pub async fn make_two_users_be_friends(
    user_one_id: UserID,
    user_two_id: UserID,
) -> Result<ChatID, ()> {
    let mut con = get_con().await?;

    let (id1, id2) = if user_one_id < user_two_id {
        (user_one_id, user_two_id)
    } else {
        (user_two_id, user_one_id)
    };

    let chat_id: ChatID = con.incr(path::LAST_CHAT_ID, 1).await.map_err(|_| ())?;
    let serialized_chat_info = format!(r#"{{"id":{}, "users":[{},{}]}}"#, chat_id, id1, id2);

    redis::pipe()
        .hset(
            path::FRIEND_CHAT_MAP,
            index::get_friend_pair_index(id1, id2).as_str(),
            chat_id,
        )
        .ignore()
        .set(
            index::get_chat_info_index(chat_id).as_str(),
            &serialized_chat_info,
        )
        .ignore()
        .set(index::get_chat_user_index(chat_id, 0).as_str(), id1)
        .ignore()
        .set(index::get_chat_user_index(chat_id, 1).as_str(), id2)
        .ignore()
        .hset(index::get_user_chats_index(id1).as_str(), chat_id, 0)
        .ignore()
        .hset(index::get_user_chats_index(id2).as_str(), chat_id, 0)
        .ignore()
        .query_async(con.deref_mut())
        .await
        .map_err(|_| ())?;

    return Ok(chat_id);
}

pub async fn check_make_friend_error(
    user_one_id: UserID,
    user_two_id: UserID,
) -> Result<(), MakeFriendError> {
    let mut con = get_con()
        .await
        .map_err(|_| MakeFriendError::DatabaseError)?;

    let (id1, id2) = if user_one_id < user_two_id {
        (user_one_id, user_two_id)
    } else {
        (user_two_id, user_one_id)
    };

    let last_user_id: UserID = con
        .get(path::LAST_USER_ID)
        .await
        .map_err(|_| MakeFriendError::DatabaseError)?;

    let opt: Option<ChatID> = con
        .hget(
            path::FRIEND_CHAT_MAP,
            index::get_friend_pair_index(id1, id2).as_str(),
        )
        .await
        .map_err(|_| MakeFriendError::DatabaseError)?;

    if id1 > last_user_id || id2 > last_user_id {
        return Err(MakeFriendError::UserNotFound);
    }

    if check_user_exist(id1).await.is_err() || check_user_exist(id2).await.is_err() {
        return Err(MakeFriendError::UserNotFound);
    }

    if let Some(chat_id) = opt {
        return Err(if chat_id == 0 {
            MakeFriendError::RequestExisted
        } else {
            MakeFriendError::AlreadyBeFrineds
        });
    } else {
        return Ok(());
    }
}

pub async fn set_user_setting(user_id: UserID, user_setting: String) -> SetSettingResponse {
    if let Ok(mut con) = get_con().await {
        let set_result: Result<bool, ()> = con
            .set(
                index::get_user_setting_index(user_id).as_str(),
                user_setting,
            )
            .await
            .map_err(|_| ());
        match set_result {
            Ok(_) => return SetSettingResponse::Success,
            Err(_) => return SetSettingResponse::DatabaseError,
        }
    } else {
        return SetSettingResponse::ServerError;
    }
}

pub async fn get_user_setting(user_id: UserID) -> Result<Option<String>, ()> {
    if let Ok(mut con) = get_con().await {
        let user_setting: Result<String, ()> = con
            .get(index::get_user_setting_index(user_id).as_str())
            .await
            .map_err(|_| ());
        if let Ok(user_setting) = user_setting {
            debug!("get_user_setting: {}", user_setting);
            return Ok(Some(user_setting));
        } else {
            return Ok(None);
        }
    } else {
        return Err(());
    }
}

pub async fn update_user_name(user_id: UserID, new_name: String) -> UserUpdateResponse {
    if let Ok(mut con) = get_con().await {
        let old_user_info: GetUserInfoResponse = get_user_info(user_id).await;
        match old_user_info {
            GetUserInfoResponse::Success(old_user_info) => {
                let new_user_info: UserInfo = UserInfo {
                    user_name: new_name.clone(),
                    ..old_user_info
                };
                let set_result: Result<bool, ()> = con
                    .set(
                        index::get_user_info_index(user_id).as_str(),
                        serde_json::to_string::<UserInfo>(&new_user_info).unwrap(),
                    )
                    .await
                    .map_err(|_| ());
                let add_result = add_user_name_id(&new_name, user_id).await;
                let del_result = del_user_name_id(&old_user_info.user_name, user_id).await;
                if set_result.is_err() || add_result.is_err() || del_result.is_err() {
                    return UserUpdateResponse::DatabaseError;
                }
                return UserUpdateResponse::Success;
            }
            _ => return UserUpdateResponse::ServerError,
        }
    } else {
        return UserUpdateResponse::ServerError;
    }
}

pub async fn update_user_avater(user_id: UserID, new_hash: String) -> UserUpdateResponse {
    if let Ok(mut con) = get_con().await {
        let old_user_info: GetUserInfoResponse = get_user_info(user_id).await;
        match old_user_info {
            GetUserInfoResponse::Success(old_user_info) => {
                let new_user_info: UserInfo = UserInfo {
                    avater_hash: new_hash,
                    ..old_user_info
                };
                let set_result: Result<bool, ()> = con
                    .set(
                        index::get_user_info_index(user_id).as_str(),
                        serde_json::to_string::<UserInfo>(&new_user_info).unwrap(),
                    )
                    .await
                    .map_err(|_| ());
                match set_result {
                    Ok(_) => UserUpdateResponse::Success,
                    Err(_) => UserUpdateResponse::ServerError,
                }
            }
            _ => return UserUpdateResponse::ServerError,
        }
    } else {
        return UserUpdateResponse::ServerError;
    }
}

pub async fn update_user_password(user_id: UserID, new_password: String) -> UserUpdateResponse {
    if let Ok(mut con) = get_con().await {
        let set_result: Result<bool, ()> = con
            .set(
                index::get_user_password_index(user_id).as_str(),
                new_password,
            )
            .await
            .map_err(|_| ());
        match set_result {
            Ok(_) => UserUpdateResponse::Success,
            Err(_) => UserUpdateResponse::ServerError,
        }
    } else {
        return UserUpdateResponse::ServerError;
    }
}

pub async fn get_chat_id_by_friends(
    user_id1: UserID,
    user_id2: UserID,
) -> Result<Option<ChatID>, ()> {
    let (id1, id2) = if user_id1 < user_id2 {
        (user_id1, user_id2)
    } else {
        (user_id2, user_id1)
    };
    let mut con = get_con().await?;
    return con
        .hget(
            path::FRIEND_CHAT_MAP,
            index::get_friend_pair_index(id1, id2).as_str(),
        )
        .await
        .map_err(|_| ());
}

pub async fn unfriend(user_id: UserID, friend_id: UserID) -> UserUnfriendResponse {
    if let Ok(mut con) = get_con().await {
        let (id1, id2) = if user_id < friend_id {
            (user_id, friend_id)
        } else {
            (friend_id, user_id)
        };

        let get_result: Result<ChatID, ()> = con
            .hget(
                path::FRIEND_CHAT_MAP,
                index::get_friend_pair_index(id1, id2).as_str(),
            )
            .await
            .map_err(|_| ());

        if let Ok(chat_id) = get_result {
            let del_result: Result<(bool, bool, bool, bool, bool), ()> = redis::pipe()
                .hdel(
                    path::FRIEND_CHAT_MAP,
                    index::get_friend_pair_index(id1, id2).as_str(),
                )
                .hdel(index::get_user_chats_index(user_id).as_str(), chat_id)
                .hdel(index::get_user_chats_index(friend_id).as_str(), chat_id)
                .del(index::get_chat_user_index(chat_id, 0).as_str())
                .del(index::get_chat_user_index(chat_id, 1).as_str())
                .query_async(con.deref_mut())
                .await
                .map_err(|_| ());
            match del_result {
                Ok(_) => return UserUnfriendResponse::Success { chat_id },
                Err(_) => return UserUnfriendResponse::ServerError,
            }
        } else {
            return UserUnfriendResponse::NotFriend;
        }
    } else {
        return UserUnfriendResponse::ServerError;
    }
}

pub async fn set_user_already_read(
    user_id: UserID,
    data: UserSetAlreadyReadData,
) -> SetAlreadyReadResponse {
    if let Ok(mut con) = get_con().await {
        let get_result: Result<MessageID, ()> = con
            .get(index::get_chat_last_id_index(data.chat_id).as_str())
            .await
            .map_err(|_| ());
        match get_result {
            Ok(last_message_id) => {
                if data.in_chat_id > last_message_id {
                    return SetAlreadyReadResponse::ServerError;
                }
                let set_result: Result<bool, ()> = con
                    .hset(
                        index::get_user_chats_index(user_id).as_str(),
                        data.chat_id,
                        data.in_chat_id,
                    )
                    .await
                    .map_err(|_| ());
                match set_result {
                    Ok(_) => return SetAlreadyReadResponse::Success,
                    Err(_) => return SetAlreadyReadResponse::ServerError,
                }
            }
            Err(_) => return SetAlreadyReadResponse::ServerError,
        }
    } else {
        return SetAlreadyReadResponse::ServerError;
    }
}

pub async fn check_user_in_chat(user_id: UserID, chat_id: ChatID) -> Result<bool, ()> {
    let mut con = get_con().await?;

    con.hexists(index::get_user_chats_index(user_id).as_str(), chat_id)
        .await
        .map_err(|_| ())
}

pub async fn write_user_notice(
    user_id: UserID,
    timestamp: Timestamp,
    serialized: &SerializedUserNotice,
) -> Result<(), ()> {
    let mut con = get_con().await.map_err(|_| ())?;

    con.zadd(
        index::get_user_notice_index(user_id).as_str(),
        serialized,
        timestamp,
    )
    .await
    .map_err(|_| ())?;

    return Ok(());
}

// TODO : 切换至u64 Inf
const TIMESTAMP_INF: Timestamp = 1000000000000000000;

pub async fn get_user_notice(
    user_id: UserID,
    start_timestamp: Timestamp,
) -> Result<Vec<SerializedUserNotice>, ()> {
    let mut con = get_con().await.map_err(|_| ())?;

    let res: Vec<SerializedUserNotice> = con
        .zrangebyscore(
            index::get_user_notice_index(user_id).as_str(),
            start_timestamp,
            TIMESTAMP_INF,
        )
        .await
        .map_err(|_| ())?;

    return Ok(res);
}

pub async fn get_user_id(name: &str) -> Result<Vec<UserID>, bool> {
    if let Ok(mut con) = get_con().await {
        let get_result: Result<String, ()> = con.hget(path::NAME_ID, &name).await.map_err(|_| ());
        if let Ok(ids) = get_result {
            if let Ok(user_ids) = serde_json::from_str::<Vec<UserID>>(&ids) {
                return Ok(user_ids);
            } else {
                return Err(false);
            }
        } else {
            return Err(true);
        }
    } else {
        return Err(false);
    }
}

pub async fn add_user_name_id(name: &str, id: UserID) -> Result<(), ()> {
    let user_ids: Vec<UserID>;
    if let Ok(mut ids) = get_user_id(name).await {
        ids.push(id);
        user_ids = ids;
    } else {
        user_ids = vec![id];
    }
    let serialized_user_ids = serde_json::to_string(&user_ids).unwrap();
    if let Ok(mut con) = get_con().await {
        con.hset(path::NAME_ID, &name, serialized_user_ids)
            .await
            .map_err(|_| ())
    } else {
        Err(())
    }
}

pub async fn del_user_name_id(name: &str, id: UserID) -> Result<(), ()> {
    let user_ids: Vec<UserID>;
    if let Ok(mut ids) = get_user_id(name).await {
        ids.retain(|&x| x != id);
        user_ids = ids;
    } else {
        return Err(());
    }
    let serialized_user_ids = serde_json::to_string(&user_ids).unwrap();
    if let Ok(mut con) = get_con().await {
        con.hset(path::NAME_ID, &name, serialized_user_ids)
            .await
            .map_err(|_| ())
    } else {
        Err(())
    }
}

pub async fn user_log_off(user_id: UserID) -> (UserLogOffResponse, Vec<(UserID, ChatID)>) {
    if let Ok(mut con) = get_con().await {
        let chats = get_user_chat_list(user_id).await;
        if chats.is_err() {
            return (UserLogOffResponse::DatabaseError, vec![]);
        }
        let chats = chats.unwrap();
        let mut groups = vec![];
        let mut friends = vec![];
        let mut frineds_chats = vec![];
        for chat in chats {
            if let Ok(members) = get_chat_user_list(chat.0).await {
                match members {
                    ChatMembers::Group(_) => {
                        let check_result = check_user_is_owner(user_id, chat.0).await;
                        if check_result.is_err() {
                            return (UserLogOffResponse::DatabaseError, vec![]);
                        } else if check_result.unwrap() {
                            return (UserLogOffResponse::NoPermission, vec![]);
                        } else {
                            groups.push(chat.0);
                        }
                    }
                    ChatMembers::Private(pair) => {
                        if user_id == pair.0 {
                            friends.push(pair.1);
                            frineds_chats.push((pair.1, chat.0));
                        } else {
                            friends.push(pair.0);
                            frineds_chats.push((pair.0, chat.0));
                        }
                    }
                }
            } else {
                return (UserLogOffResponse::DatabaseError, vec![]);
            }
        }

        let email = get_user_email(user_id).await.unwrap();
        let user_email: Result<bool, ()> =
            con.hdel(path::USER_EMAIL_MAP, email).await.map_err(|_| ());
        if user_email.is_err() {
            return (UserLogOffResponse::DatabaseError, vec![]);
        } else if !user_email.unwrap() {
            return (UserLogOffResponse::UserNotFound, vec![]);
        }
        let user_email: Result<bool, ()> = con
            .del(index::get_user_email_index(user_id).as_str())
            .await
            .map_err(|_| ());
        if user_email.is_err() {
            return (UserLogOffResponse::DatabaseError, vec![]);
        } else if !user_email.unwrap() {
            return (UserLogOffResponse::UserNotFound, vec![]);
        }

        for friend in friends.clone() {
            unfriend(user_id, friend).await;
        }
        for group in groups {
            quit_group_chat(user_id, group).await;
        }
        if let UserUpdateResponse::Success =
            update_user_name(user_id, "用户已注销".to_string()).await
        {
            let old_user_info: GetUserInfoResponse = get_user_info(user_id).await;
            match old_user_info {
                GetUserInfoResponse::Success(old_user_info) => {
                    let del_result = del_user_name_id(&old_user_info.user_name, user_id).await;
                    if del_result.is_err() {
                        return (UserLogOffResponse::DatabaseError, vec![]);
                    }
                }
                _ => return (UserLogOffResponse::DatabaseError, vec![]),
            }
        } else {
            return (UserLogOffResponse::DatabaseError, vec![]);
        }
        let set_result: Result<bool, ()> = con
            .set(index::get_user_exist_index(user_id).as_str(), 0)
            .await
            .map_err(|_| ());
        if set_result.is_err() {
            return (UserLogOffResponse::DatabaseError, vec![]);
        }

        return (UserLogOffResponse::Success, frineds_chats);
    } else {
        return (UserLogOffResponse::ServerError, vec![]);
    }
}

pub async fn apply_for_token(user_id: UserID) -> UserApplyForTokenResponse {
    if let Ok(mut con) = get_con().await {
        let token = Token {
            token: Uuid::new_v4().simple().to_string(),
            timestamp: Utc::now().timestamp_millis() as u64,
        };
        let serialized_token = serde_json::to_string(&token).unwrap();
        let set_result: Result<bool, ()> = con
            .set(
                index::get_user_token_index(user_id).as_str(),
                serialized_token,
            )
            .await
            .map_err(|_| ());
        if set_result.is_err() || !set_result.unwrap() {
            return UserApplyForTokenResponse::DatabaseError;
        }
        return UserApplyForTokenResponse::Success {
            token: token.token,
            timestamp: token.timestamp,
        };
    } else {
        return UserApplyForTokenResponse::ServerError;
    }
}

pub async fn user_login_with_token(data: &UserLoginData) -> Result<UserLoginResponse, ()> {
    let mut con = get_con().await?;

    let user_id_opt: Option<UserID> = con
        .hget(path::USER_EMAIL_MAP, &data.email)
        .await
        .map_err(|_| ())?;

    let user_id = match user_id_opt {
        Some(id) => id,
        None => return Ok(UserLoginResponse::UserNotFound),
    };

    let token: String = con
        .get(index::get_user_token_index(user_id).as_str())
        .await
        .map_err(|_| ())?;

    let token = serde_json::from_str::<Token>(&token).unwrap();

    if &token.token != data.token.as_ref().unwrap() {
        return Ok(UserLoginResponse::TokenError);
    }

    if chrono::Utc::now().timestamp_millis() as u64 - token.timestamp
        > (Config::get().user.token_expire_time * 1000) as u64
    {
        return Ok(UserLoginResponse::TokenExpired);
    }

    return Ok(UserLoginResponse::Success { user_id });
}

pub async fn check_user_exist(user_id: UserID) -> Result<(), ()> {
    let mut con = get_con().await?;
    let exist: u32 = con
        .get(index::get_user_exist_index(user_id).as_str())
        .await
        .map_err(|_| ())?;
    if exist == 1 {
        Ok(())
    } else {
        Err(())
    }
}
