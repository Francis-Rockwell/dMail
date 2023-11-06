use crate::{
    config::datatype::ChatID,
    database,
    user::{
        UserCreateGroupChatData, UserGetUserReadInGroupResponse, UserGetUserReadInPrivateResponse,
        UserGroupOwnerTransferResponse, UserPullGroupNoticeResponse, UserQuitGroupChatResponse,
        UserSendGroupNoticeResponse, UserSetGroupAdminResponse, UserUnsetGroupAdminResponse,
        UserUpdateGroupContent, UserUpdateGroupInfoResponse,
    },
};

pub async fn test_for_creat_group_chat() -> Result<(), ()> {
    let data = UserCreateGroupChatData {
        name: "name".to_string(),
        avater_hash: "avater".to_string(),
    };
    match database::create_group_chat(1, data).await {
        Ok(_) => Ok(()),
        _ => panic!("create_group_chat"),
    }
}

pub async fn test_for_add_user_to_chat() -> Result<(), ()> {
    match database::add_user_to_chat(2, 2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("add_user_to_chat"),
    }
}

pub async fn test_for_write_message_to_chat() -> Result<(), ()> {
    match database::write_message_to_chat("String", "Message".to_string(), 1, 1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_message_to_chat"),
    }
}

pub async fn test_for_check_user_can_send_in_chat(chat_id: ChatID) -> Result<(), ()> {
    match database::check_user_can_send_in_chat(1, chat_id).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("check_user_can_send_in_chat"),
    }
}

pub async fn test_for_get_chat_user_list(chat_id: ChatID) -> Result<(), ()> {
    match database::get_chat_user_list(chat_id).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_chat_user_list"),
    }
}

pub async fn test_for_get_messages_in_chat() -> Result<(), ()> {
    match database::get_messages_in_chat(1, 1, None).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_messages_in_chat"),
    }
}

pub async fn test_for_get_chats_last_messages() -> Result<(), ()> {
    match database::get_chats_last_messages(&vec![(1, 1)], 1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_chats_last_messages"),
    }
}

pub async fn test_for_get_chat_owner() -> Result<(), ()> {
    match database::get_chat_owner(2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_chat_owner"),
    }
}

pub async fn test_for_get_chat_info() -> Result<(), ()> {
    match database::get_chat_info(1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_chat_info"),
    }
}

pub async fn test_for_revoke_message() -> Result<(), ()> {
    match database::revoke_message(1, 1, 1, 1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("revoke_message"),
    }
}

pub async fn test_for_check_group_invitation_error() -> Result<(), ()> {
    let data = UserCreateGroupChatData {
        name: "name".to_string(),
        avater_hash: "avater".to_string(),
    };
    database::create_group_chat(1, data).await?;
    match database::check_group_invitation_error(1, 2, 3).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("check_group_invitation_error"),
    }
}

pub async fn test_for_set_as_admin() -> Result<(), ()> {
    database::add_user_to_chat(3, 2).await?;
    match database::set_as_admin(2, 3).await {
        UserSetGroupAdminResponse::Success {
            chat_id: _,
            user_id: _,
        } => Ok(()),
        _ => panic!("set_as_admin"),
    }
}

pub async fn test_for_quit_group_chat() -> Result<(), ()> {
    match database::quit_group_chat(2, 3).await {
        UserQuitGroupChatResponse::Success { chat_id: _ } => Ok(()),
        _ => panic!("quit_group_chat"),
    }
}

pub async fn test_for_check_invited_json_group_error() -> Result<(), ()> {
    match database::check_invited_join_group_error(1, 2, 3).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("check_group_invitation_error"),
    }
}

pub async fn test_for_check_join_group_error() -> Result<(), ()> {
    match database::check_join_group_error(2, 3).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("check_join_group_error"),
    }
}

pub async fn test_for_get_chat_admins_list() -> Result<(), ()> {
    match database::get_chat_admins_list(2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_chat_admins_list"),
    }
}

pub async fn test_for_check_user_is_owner() -> Result<(), ()> {
    match database::check_user_is_owner(1, 2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("check_user_is_owner"),
    }
}

pub async fn test_for_check_user_is_admin() -> Result<(), ()> {
    match database::check_user_is_admin(1, 2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("check_user_is_admin"),
    }
}

pub async fn test_for_owner_transfer() -> Result<(), ()> {
    match database::owner_transfer(2, 2).await {
        UserGroupOwnerTransferResponse::Success {
            chat_id: _,
            user_id: _,
        } => Ok(()),
        _ => panic!("owner_transfer"),
    }
}

pub async fn test_for_add_group_notice() -> Result<(), ()> {
    match database::add_group_notice(2, 2, 0, "notice".to_string()).await {
        UserSendGroupNoticeResponse::Success {
            chat_id: _,
            client_id: _,
            notice_id: _,
            timestamp: _,
        } => Ok(()),
        _ => panic!("add_group_notice"),
    }
}

pub async fn test_for_pull_group_notice() -> Result<(), ()> {
    match database::pull_group_notice(2, 1).await {
        UserPullGroupNoticeResponse::Success {
            chat_id: _,
            group_notice: _,
        } => Ok(()),
        _ => panic!("pull_group_notice"),
    }
}

pub async fn test_for_update_group_name() -> Result<(), ()> {
    let data = UserUpdateGroupContent::GroupName {
        new_name: "new_name".to_string(),
    };
    match database::update_group_info(2, data).await {
        UserUpdateGroupInfoResponse::Success => Ok(()),
        _ => panic!("update_group_info"),
    }
}

pub async fn test_for_update_group_avater() -> Result<(), ()> {
    let data = UserUpdateGroupContent::Avater {
        new_avater: "new_avater".to_string(),
    };
    match database::update_group_info(2, data).await {
        UserUpdateGroupInfoResponse::Success => Ok(()),
        _ => panic!("update_group_info"),
    }
}

pub async fn test_for_unset_admin() -> Result<(), ()> {
    match database::unset_admin(1, 2).await {
        UserUnsetGroupAdminResponse::Success {
            chat_id: _,
            user_id: _,
        } => Ok(()),
        _ => panic!("unset_admin"),
    }
}

pub async fn test_for_check_is_group() -> Result<(), ()> {
    match database::check_is_group(2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("check_is_group"),
    }
}

pub async fn test_for_get_user_read_in_group() -> Result<(), ()> {
    database::write_message_to_chat("String", "Message".to_string(), 2, 1).await?;
    match database::get_user_read_in_group(2, 1).await {
        UserGetUserReadInGroupResponse::Success {
            chat_id: _,
            in_chat_id: _,
            user_ids: _,
        } => Ok(()),
        _ => panic!("get_user_read_in_group"),
    }
}

pub async fn test_for_get_user_read_in_private() -> Result<(), ()> {
    match database::get_user_read_in_private(2, 1).await {
        UserGetUserReadInPrivateResponse::Success {
            chat_id: _,
            in_chat_id: _,
        } => Ok(()),
        _ => panic!("get_user_read_in_private"),
    }
}

pub async fn test_for_get_private_chat_user_list() -> Result<(), ()> {
    match database::get_private_chat_user_list(1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_private_chat_user_list"),
    }
}
