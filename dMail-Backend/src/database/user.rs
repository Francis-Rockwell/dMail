use crate::config::datatype::*;

use crate::user::*;

use super::redis;

/// 在数据库内写入一个用户。
/// 此接口仅负责检查 UserRegisterResponse::EmailRegistered
pub async fn user_register(data: UserRegisterData) -> UserRegisterResponse {
    match redis::user_register(data).await {
        Ok(res) => res,
        Err(_) => UserRegisterResponse::ServerError,
    }
}

/// 数据库用户登录接口。
// 此接口仅负责检查 UserRegisterResponse::PasswordError, UserRegisterResponse::UserNotFound
pub async fn user_login_with_password(data: UserLoginData) -> UserLoginResponse {
    match redis::user_login_with_password(&data).await {
        Ok(res) => res,
        Err(_) => UserLoginResponse::ServerError,
    }
}

/// 用户使用token自动登录时检查
pub async fn user_login_with_token(data: UserLoginData) -> UserLoginResponse {
    match redis::user_login_with_token(&data).await {
        Ok(res) => res,
        Err(_) => UserLoginResponse::ServerError,
    }
}

/// 通过邮件查找用户ID
pub async fn get_user_id_by_email(email: &String) -> Result<Option<UserID>, ()> {
    return redis::get_user_id_by_email(email).await;
}

/// 获得用户会话列表
pub async fn get_user_chat_list(user_id: UserID) -> Result<Vec<(ChatID, MessageID)>, ()> {
    return redis::get_user_chat_list(user_id).await;
}

/// 获得用户Info
pub async fn get_user_info(user_id: UserID) -> GetUserInfoResponse {
    return redis::get_user_info(user_id).await;
}

/// 获得用户邮箱
pub async fn get_user_email(user_id: UserID) -> Result<String, ()> {
    return redis::get_user_email(user_id).await;
}

/// 直接让两个用户成为好友，在用户确认好友申请后
// 需要保证两个用户之前不是好友，成功后返回两个人的专属对话ID
// 有趣的是，如果之后支持删除好友，需要返回删除好友之前的对话ID，而不是生成一个新ID
pub async fn make_two_users_be_friends(
    user_one_id: UserID,
    user_two_id: UserID,
) -> Result<ChatID, ()> {
    return redis::make_two_users_be_friends(user_one_id, user_two_id).await;
}

/// 发送好友邀请时检查错误
pub async fn check_make_friend_error(
    user_one_id: UserID,
    user_two_id: UserID,
) -> Result<(), MakeFriendError> {
    return redis::check_make_friend_error(user_one_id, user_two_id).await;
}

/// 设置user_setting
pub async fn set_user_setting(user_id: UserID, user_setting: String) -> SetSettingResponse {
    return redis::set_user_setting(user_id, user_setting).await;
}

/// 获取uesr_setting
pub async fn get_user_setting(user_id: UserID) -> Result<Option<String>, ()> {
    return redis::get_user_setting(user_id).await;
}

/// 更新用户姓名
pub async fn update_user_name(user_id: UserID, new_name: String) -> UserUpdateResponse {
    return redis::update_user_name(user_id, new_name).await;
}

/// 更新用户头像
pub async fn update_user_avater(user_id: UserID, new_hash: String) -> UserUpdateResponse {
    return redis::update_user_avater(user_id, new_hash).await;
}

/// 更新密码
pub async fn update_user_password(user_id: UserID, new_password: String) -> UserUpdateResponse {
    return redis::update_user_password(user_id, new_password).await;
}

// 从一对好友ID获取chat_id
// pub async fn get_chat_id_by_friend(user_id1: UserID, user_id2: UserID) -> Result<ChatID, ()> {
//     return redis::get_chat_id_by_friend(user_id1, user_id2).await;
// }

/// 解除好友关系
pub async fn unfriend(user_id: UserID, friend_id: UserID) -> UserUnfriendResponse {
    return redis::unfriend(user_id, friend_id).await;
}

/// 更新已读消息
pub async fn set_user_already_read(
    user_id: UserID,
    data: UserSetAlreadyReadData,
) -> SetAlreadyReadResponse {
    return redis::set_user_already_read(user_id, data).await;
}

/// 检查用户是否在Chat中
pub async fn check_user_in_chat(user_id: UserID, chat_id: ChatID) -> Result<bool, ()> {
    return redis::check_user_in_chat(user_id, chat_id).await;
}

/// 写入用户的notice
pub async fn write_user_notice(
    user_id: UserID,
    timestamp: Timestamp,
    serialized: &SerializedUserNotice,
) -> Result<(), ()> {
    return redis::write_user_notice(user_id, timestamp, serialized).await;
}

/// 获取用户的notice
pub async fn get_user_notice(
    user_id: UserID,
    start_timestamp: Timestamp,
) -> Result<Vec<SerializedUserNotice>, ()> {
    return redis::get_user_notice(user_id, start_timestamp).await;
}

/// 通过好友的UserID获取与他的ChatID
pub async fn get_chat_id_by_friends(
    user_id1: UserID,
    user_id2: UserID,
) -> Result<Option<ChatID>, ()> {
    return redis::get_chat_id_by_friends(user_id1, user_id2).await;
}

/// 通过好友的名字获取UserID
pub async fn get_user_id(name: String) -> UserGetUserIDResponse {
    match redis::get_user_id(&name).await {
        Ok(user_ids) => UserGetUserIDResponse::Success { user_ids },
        Err(not_found) => {
            if not_found {
                return UserGetUserIDResponse::NotFound;
            } else {
                return UserGetUserIDResponse::DatabaseError;
            }
        }
    }
}

/// 用户注销
pub async fn user_log_off(user_id: UserID) -> (UserLogOffResponse, Vec<(UserID, ChatID)>) {
    return redis::user_log_off(user_id).await;
}

/// 客户端申请token
pub async fn apply_for_token(user_id: UserID) -> UserApplyForTokenResponse {
    return redis::apply_for_token(user_id).await;
}

/// 检查用户是否存在
pub async fn check_user_exist(user_id: UserID) -> Result<(), ()> {
    return redis::check_user_exist(user_id).await;
}
