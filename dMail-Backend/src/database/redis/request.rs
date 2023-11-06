use std::ops::DerefMut;

use mobc_redis::redis;
use mobc_redis::redis::AsyncCommands;

use super::common::*;
use super::index;
use super::path;
use crate::config::datatype::ChatID;
use crate::config::datatype::SerializedRequest;
use crate::config::datatype::UserReqId;
use crate::{config::datatype::UserID, user::*};

pub async fn write_user_request(
    sender_id: UserID,
    data: UserSendRequestData,
    handler: &UserRequestHandler,
) -> Result<(SerializedRequest, UserRequestInfo), ()> {
    let mut con = get_con().await?;

    let req_id: UserReqId = con.incr(path::LAST_REQ_ID, 1).await.map_err(|_| ())?;

    let req_info = UserRequestInfo {
        req_id,
        sender_id,
        message: data.message,
        content: data.content,
    };

    let serialized_info = serde_json::to_string(&req_info).unwrap();

    con.set::<_, _, ()>(index::get_req_info_index(req_id).as_str(), &serialized_info)
        .await
        .map_err(|_| ())?;

    let mut pipeline = redis::pipe();

    match handler {
        UserRequestHandler::One(user_id) => {
            pipeline
                .zadd(
                    index::get_user_reqs_index(*user_id).as_str(),
                    req_id,
                    req_id,
                )
                .ignore();
        }
        UserRequestHandler::Group(ids) => {
            for user_id in ids {
                pipeline
                    .zadd(
                        index::get_user_reqs_index(*user_id).as_str(),
                        req_id,
                        req_id,
                    )
                    .ignore();
            }
        }
    }

    pipeline
        .zadd(
            index::get_user_reqs_index(sender_id).as_str(),
            req_id,
            req_id,
        )
        .ignore();

    pipeline
        .query_async::<_, ()>(con.deref_mut())
        .await
        .map_err(|_| ())?;

    return Ok((
        format!(r#"{{"info":{},"state":"Unsolved"}}"#, serialized_info),
        req_info,
    ));
}

pub async fn store_user_request(user_id: UserID, req_id: UserReqId) -> Result<(), ()> {
    let mut con = get_con().await?;
    con.zadd(index::get_user_reqs_index(user_id).as_str(), req_id, req_id)
        .await
        .map_err(|_| ())
}

pub async fn get_user_request(req_id: UserReqId) -> Result<Option<UserRequset>, ()> {
    let mut con = get_con().await?;

    let (serialized_info_opt, state_opt): (Option<String>, Option<bool>) = redis::pipe()
        .get(index::get_req_info_index(req_id).as_str())
        .get(index::get_req_state_index(req_id).as_str())
        .query_async(con.deref_mut())
        .await
        .map_err(|_| ())?;

    let serialized_info = match serialized_info_opt {
        Some(info) => info,
        None => return Ok(None),
    };

    let state = match state_opt {
        Some(state_bool) => {
            if state_bool {
                UserRequestState::Approved
            } else {
                UserRequestState::Refused
            }
        }
        None => UserRequestState::Unsolved,
    };

    let info: UserRequestInfo =
        serde_json::from_str(&serialized_info).expect("UserRqeuestInfo 反序列化失败");

    return Ok(Some(UserRequset { info, state }));
}

pub async fn set_user_request_state(
    req_id: UserReqId,
    state: UserRequestState,
) -> Result<(), UserSolveRequestState> {
    let mut con = get_con()
        .await
        .map_err(|_| UserSolveRequestState::DatabaseError)?;

    let state_index = index::get_req_state_index(req_id);

    let state_opt: Option<bool> = con
        .get(state_index.as_str())
        .await
        .map_err(|_| UserSolveRequestState::DatabaseError)?;

    if state_opt.is_some() {
        return Err(UserSolveRequestState::AlreadySolved);
    };

    let state_bool = match state {
        UserRequestState::Approved => true,
        UserRequestState::Unsolved => return Err(UserSolveRequestState::AnswerUnsolved),
        UserRequestState::Refused => false,
    };

    con.set::<_, _, ()>(state_index.as_str(), state_bool)
        .await
        .map_err(|_| UserSolveRequestState::DatabaseError)?;

    return Ok(());
}

pub async fn get_user_requests(
    user_id: UserID,
    start_id: UserReqId,
) -> Result<Vec<SerializedRequest>, ()> {
    let mut con = get_con().await?;

    let reqs_id_opt: Option<Vec<UserReqId>> = con
        .zrangebyscore(
            index::get_user_reqs_index(user_id).as_str(),
            start_id,
            "+inf",
        )
        .await
        .map_err(|_| ())?;

    if reqs_id_opt.is_none() {
        return Ok(vec![]);
    }

    let req_ids = reqs_id_opt.unwrap();

    let mut reqs: Vec<SerializedRequest> = Vec::new();

    for req_id in req_ids {
        let (serialized_info, state_opt): (String, Option<bool>) = redis::pipe()
            .get(index::get_req_info_index(req_id).as_str())
            .get(index::get_req_state_index(req_id).as_str())
            .query_async(con.deref_mut())
            .await
            .map_err(|_| ())?;

        let serialized_req = match state_opt {
            Some(state_bool) => {
                if state_bool {
                    format!(r#"{{"info":{},"state":"Approved"}}"#, serialized_info)
                } else {
                    format!(r#"{{"info":{},"state":"Refused"}}"#, serialized_info)
                }
            }
            None => format!(r#"{{"info":{},"state":"Unsolved"}}"#, serialized_info),
        };

        reqs.push(serialized_req);
    }

    return Ok(reqs);
}

pub async fn write_friend_request_send(user_one_id: UserID, user_two_id: UserID) -> Result<(), ()> {
    let (id1, id2) = if user_one_id < user_two_id {
        (user_one_id, user_two_id)
    } else {
        (user_two_id, user_one_id)
    };

    let mut con = get_con().await?;

    con.hset(
        path::FRIEND_CHAT_MAP,
        index::get_friend_pair_index(id1, id2).as_str(),
        0,
    )
    .await
    .map_err(|_| ())?;

    return Ok(());
}

pub async fn delete_friend_request_send(
    user_one_id: UserID,
    user_two_id: UserID,
) -> Result<(), ()> {
    let (id1, id2) = if user_one_id < user_two_id {
        (user_one_id, user_two_id)
    } else {
        (user_two_id, user_one_id)
    };

    let mut con = get_con().await?;

    con.hdel(
        path::FRIEND_CHAT_MAP,
        index::get_friend_pair_index(id1, id2).as_str(),
    )
    .await
    .map_err(|_| ())?;

    return Ok(());
}

pub async fn write_join_group_request_send(user_id: UserID, chat_id: ChatID) -> Result<(), ()> {
    let mut con = get_con().await?;
    let _: bool = con
        .sadd(index::get_user_pre_join_index(user_id).as_str(), chat_id)
        .await
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn delete_join_group_request_send(user_id: UserID, chat_id: ChatID) -> Result<(), ()> {
    let mut con = get_con().await?;
    con.srem(index::get_user_pre_join_index(user_id).as_str(), chat_id)
        .await
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn write_invite_request_send(
    inviter_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), ()> {
    let mut con = get_con().await?;
    let _: bool = con
        .hset(
            path::INVITAION_MAP,
            format!("{}:{}:{}", inviter_id, receiver_id, chat_id),
            1,
        )
        .await
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn delete_invite_request_send(
    inviter_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), ()> {
    let mut con = get_con().await?;
    con.hdel(
        path::INVITAION_MAP,
        format!("{}:{}:{}", inviter_id, receiver_id, chat_id),
    )
    .await
    .map_err(|_| ())?;
    return Ok(());
}
