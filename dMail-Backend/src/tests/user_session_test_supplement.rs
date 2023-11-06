use futures::executor::block_on;

use crate::{
    database,
    user::{
        user_session::client_message_handler::{
            file_uploaded, owner_transfer, quit_group_chat, remove_member, upload_file_req,
        },
        UserCreateGroupChatData, UserGroupOwnerTransferData, UserRegisterData,
        UserRemoveGroupMemberData, UserUploadFileRequestData,
    },
};

pub async fn test_for_user_session_quit_group_chat() -> Result<(), ()> {
    let data = UserRegisterData {
        user_name: "name".to_string(),
        password: "6B86B273FF34FCE19D6B804EFF5A3F5747ADA4EAA22F1D49C01E52DDB7875B4B".to_string(),
        email_code: 123456,
        email: "a@b.com".to_string(),
    };
    database::user_register(data).await;
    let data = UserCreateGroupChatData {
        name: "name".to_string(),
        avater_hash: "avater_hash".to_string(),
    };
    database::create_group_chat(1, data).await?;
    quit_group_chat(2, 1).await;
    let data = UserRegisterData {
        user_name: "name".to_string(),
        password: "6B86B273FF34FCE19D6B804EFF5A3F5747ADA4EAA22F1D49C01E52DDB7875B4B".to_string(),
        email_code: 123456,
        email: "e@f.com".to_string(),
    };
    database::user_register(data).await;
    database::add_user_to_chat(1, 2).await?;
    quit_group_chat(2, 1).await;
    database::add_user_to_chat(1, 2).await?;
    return Ok(());
}

pub async fn test_for_user_session_file() -> Result<(), ()> {
    let data = UserUploadFileRequestData {
        suffix: "suffix".to_string(),
        user_hash: "user_hash".to_string(),
        size: 1,
    };
    block_on(upload_file_req(1, data));
    block_on(file_uploaded(1, 1));
    return Ok(());
}

pub async fn test_for_user_session_owner_transfer() -> Result<(), ()> {
    let data = UserGroupOwnerTransferData {
        chat_id: 1,
        user_id: 2,
    };
    owner_transfer(1, data).await;
    return Ok(());
}

pub async fn test_for_user_session_remove_member() -> Result<(), ()> {
    let data = UserRemoveGroupMemberData {
        chat_id: 1,
        user_id: 1,
    };
    remove_member(2, data).await;
    return Ok(());
}
