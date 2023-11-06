use crate::database;
use crate::user::user_session::protocol::DataChecker;
use crate::user::{
    GetUserInfoResponse, SetAlreadyReadResponse, SetSettingResponse, UserApplyForTokenResponse,
    UserCreateGroupChatData, UserGetUserIDResponse, UserLogOffResponse, UserLoginData,
    UserLoginResponse, UserRegisterData, UserRegisterResponse, UserSetAlreadyReadData,
    UserUnfriendResponse, UserUpdateResponse,
};

//register_password format
pub fn test_for_check_password() -> Result<(), ()> {
    let data = UserRegisterData {
        email: "yzr21@mails.tsinghua.edu.cn".to_string(),
        password: "6B86B273FF34FCE19D6B804EFF5A3F5747ADA4EAA22F1D49C01E52DDB7875B4B".to_string(),
        email_code: 123456,
        user_name: "MyName".to_string(),
    };
    if let Err(UserRegisterResponse::PasswordFormatError) = data.check_data() {
        panic!("DataChecker<UserLoginResponse> for UserLoginData Error");
    }

    let data = UserRegisterData {
        email: "yzr21@mails.tsinghua.edu.cn".to_string(),
        password: "20A10?179".to_string(),
        email_code: 123456,
        user_name: "MyName".to_string(),
    };
    if let Err(UserRegisterResponse::PasswordFormatError) = data.check_data() {
        return Ok(());
    } else {
        panic!("DataChecker<UserLoginResponse> for UserLoginData Error");
    }
}

//register_username format
pub fn test_for_check_username() -> Result<(), ()> {
    let data = UserRegisterData {
        email: "zhaochon21@mails.tsinghua.edu.cn".to_string(),
        password: "6B86B273FF34FCE19D6B804EFF5A3F5747ADA4EAA22F1D49C01E52DDB7875B4B".to_string(),
        email_code: 123456,
        user_name: "MyName".to_string(),
    };
    if let Err(UserRegisterResponse::UserNameFormatError) = data.check_data() {
        panic!("DataChecker<UserRegisterResponse> for UserName Format Error");
    }

    let data = UserRegisterData {
        email: "zhaochon21@mails.tsinghua.edu.cn".to_string(),
        password: "6B86B273FF34FCE19D6B804EFF5A3F5747ADA4EAA22F1D49C01E52DDB7875B4B".to_string(),
        email_code: 123456,
        user_name: "1111111111111111111111111111111111111111".to_string(),
    };
    if let Err(UserRegisterResponse::UserNameFormatError) = data.check_data() {
        return Ok(());
    } else {
        panic!("DataChecker<UserRegisterResponse> for UserName Format Error");
    }
}

pub async fn test_for_user_register() -> Result<(), ()> {
    let data = UserRegisterData {
        user_name: "name".to_string(),
        password: "1234".to_string(),
        email_code: 123456,
        email: "a@b.com".to_string(),
    };
    match database::user_register(data).await {
        UserRegisterResponse::Success { user_id: _ } => Ok(()),
        _ => panic!("user_register"),
    }
}

pub async fn test_for_user_login_with_password() -> Result<(), ()> {
    let data = UserLoginData {
        email: "a@b.com".to_string(),
        address: None,
        token: None,
        password: Some("1234".to_string()),
        email_code: None,
    };
    match database::user_login_with_password(data).await {
        UserLoginResponse::Success { user_id: _ } => Ok(()),
        _ => panic!("user_login_with_password"),
    }
}

pub async fn test_for_apply_for_token() -> Result<String, ()> {
    match database::apply_for_token(1).await {
        UserApplyForTokenResponse::Success {
            token,
            timestamp: _,
        } => Ok(token),
        _ => panic!("apply_for_token"),
    }
}

pub async fn test_for_user_login_with_token(token: String) -> Result<(), ()> {
    let data = UserLoginData {
        email: "a@b.com".to_string(),
        address: None,
        token: Some(token),
        password: None,
        email_code: None,
    };
    match database::user_login_with_token(data).await {
        UserLoginResponse::Success { user_id: _ } => Ok(()),
        _ => panic!("login_with_token"),
    }
}

pub async fn test_for_get_user_id_by_email() -> Result<(), ()> {
    let email = "a@b.com".to_string();
    match database::get_user_id_by_email(&email).await {
        Ok(_) => Ok(()),
        _ => panic!("get_user_id_by_email"),
    }
}

pub async fn test_for_get_user_chat_list() -> Result<(), ()> {
    match database::get_user_chat_list(1).await {
        Ok(_) => Ok(()),
        _ => panic!("get_user_chat_list"),
    }
}

pub async fn test_for_get_user_info() -> Result<(), ()> {
    match database::get_user_info(1).await {
        GetUserInfoResponse::Success(_) => Ok(()),
        _ => panic!("get_user_id_by_email"),
    }
}

pub async fn test_for_get_user_email() -> Result<(), ()> {
    match database::get_user_email(1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_user_email"),
    }
}

pub async fn test_for_check_make_friend_error() -> Result<(), ()> {
    let data = UserRegisterData {
        user_name: "other".to_string(),
        password: "1234".to_string(),
        email_code: 123456,
        email: "c@d.com".to_string(),
    };
    database::user_register(data).await;
    match database::check_make_friend_error(1, 2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("check_make_friend_error"),
    }
}

pub async fn test_for_make_two_users_be_friends() -> Result<(), ()> {
    match database::make_two_users_be_friends(1, 2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_user_email"),
    }
}

pub async fn test_for_set_user_setting() -> Result<(), ()> {
    match database::set_user_setting(1, "user_setting".to_string()).await {
        SetSettingResponse::Success => Ok(()),
        _ => panic!("set_user_setting"),
    }
}

pub async fn test_for_get_user_setting() -> Result<(), ()> {
    match database::get_user_setting(1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_user_setting"),
    }
}

pub async fn test_for_update_user_name() -> Result<(), ()> {
    match database::update_user_name(1, "new_name".to_string()).await {
        UserUpdateResponse::Success => Ok(()),
        _ => panic!("update_user_name"),
    }
}

pub async fn test_for_update_user_avater() -> Result<(), ()> {
    match database::update_user_avater(1, "new_hash".to_string()).await {
        UserUpdateResponse::Success => Ok(()),
        _ => panic!("update_user_avater"),
    }
}

pub async fn test_for_update_user_password() -> Result<(), ()> {
    match database::update_user_password(1, "new_password".to_string()).await {
        UserUpdateResponse::Success => Ok(()),
        _ => panic!("update_user_password"),
    }
}

pub async fn test_for_check_user_in_chat() -> Result<(), ()> {
    match database::check_user_in_chat(1, 1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("check_user_in_chat"),
    }
}

pub async fn test_for_unfriend() -> Result<(), ()> {
    database::make_two_users_be_friends(1, 2).await?;
    match database::unfriend(1, 2).await {
        UserUnfriendResponse::Success { chat_id: _ } => Ok(()),
        _ => panic!("unfriend"),
    }
}

// set_user_already_read
pub async fn test_for_get_chat_id_by_friends() -> Result<(), ()> {
    match database::get_chat_id_by_friends(1, 2).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_chat_id_by_friends"),
    }
}

pub async fn test_for_get_user_id() -> Result<(), ()> {
    match database::get_user_id("new_name".to_string()).await {
        UserGetUserIDResponse::Success { user_ids: _ } => Ok(()),
        _ => panic!("get_user_id"),
    }
}

pub async fn test_for_write_user_notice() -> Result<(), ()> {
    match database::write_user_notice(1, 1, &"notice".to_string()).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_user_notice"),
    }
}

pub async fn test_for_get_user_notice() -> Result<(), ()> {
    match database::get_user_notice(1, 0).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_user_notice"),
    }
}

pub async fn test_for_set_user_already_read() -> Result<(), ()> {
    let data = UserSetAlreadyReadData {
        chat_id: 1,
        in_chat_id: 1,
        private: true,
    };
    match database::set_user_already_read(1, data).await {
        SetAlreadyReadResponse::Success => Ok(()),
        _ => panic!("set_user_already_read"),
    }
}

pub async fn test_for_user_log_off() -> Result<(), ()> {
    let data = UserRegisterData {
        user_name: "name".to_string(),
        password: "1234".to_string(),
        email_code: 123456,
        email: "e@f.com".to_string(),
    };
    database::user_register(data).await;
    database::make_two_users_be_friends(1, 3).await?;
    let data = UserCreateGroupChatData {
        name: "name".to_string(),
        avater_hash: "avater".to_string(),
    };
    database::create_group_chat(1, data).await?;
    database::add_user_to_chat(3, 3).await?;
    match database::user_log_off(3).await {
        (UserLogOffResponse::Success, _) => Ok(()),
        _ => panic!("user_log_off"),
    }
}
