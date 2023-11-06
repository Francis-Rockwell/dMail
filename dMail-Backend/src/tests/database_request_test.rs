use crate::{
    database,
    user::{UserRequestHandler, UserRequestState, UserRequsetContent, UserSendRequestData},
};

pub async fn test_for_write_user_request_group() -> Result<(), ()> {
    let data = UserSendRequestData {
        message: "request".to_string(),
        content: UserRequsetContent::GroupInvitation {
            receiver_id: 2,
            chat_id: 2,
        },
        client_id: 1,
    };
    match database::write_user_request(1, data, &UserRequestHandler::Group(vec![1])).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_user_request"),
    }
}

pub async fn test_for_write_user_request_one() -> Result<(), ()> {
    let data = UserSendRequestData {
        message: "request".to_string(),
        content: UserRequsetContent::GroupInvitation {
            receiver_id: 2,
            chat_id: 2,
        },
        client_id: 1,
    };
    match database::write_user_request(1, data, &UserRequestHandler::One(1)).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_user_request"),
    }
}

pub async fn test_for_get_user_requests() -> Result<(), ()> {
    match database::get_user_requests(1, 1).await {
        Ok(_) => Ok(()),
        _ => panic!("get_user_requests"),
    }
}

pub async fn test_for_store_user_request() -> Result<(), ()> {
    match database::store_user_request(2, 3).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("store_user_request"),
    }
}

pub async fn test_for_get_user_request() -> Result<(), ()> {
    match database::get_user_request(3).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_user_request"),
    }
}

pub async fn test_for_set_user_request_state() -> Result<(), ()> {
    match database::set_user_request_state(3, UserRequestState::Approved).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_user_request"),
    }
}

pub async fn test_for_write_friend_request_send() -> Result<(), ()> {
    match database::write_friend_request_send(1, 2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_friend_request_send"),
    }
}

pub async fn test_for_delete_friend_request_send() -> Result<(), ()> {
    match database::delete_friend_request_send(1, 2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_friend_request_send"),
    }
}

pub async fn test_for_write_join_group_request_send() -> Result<(), ()> {
    match database::write_join_group_request_send(1, 2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_friend_request_send"),
    }
}

pub async fn test_for_delete_join_group_request_send() -> Result<(), ()> {
    match database::delete_join_group_request_send(1, 2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_friend_request_send"),
    }
}

pub async fn test_for_write_invite_request_send() -> Result<(), ()> {
    match database::write_invite_request_send(1, 2, 1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_friend_request_send"),
    }
}

pub async fn test_for_delete_invite_request_send() -> Result<(), ()> {
    match database::delete_invite_request_send(1, 2, 1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_friend_request_send"),
    }
}
