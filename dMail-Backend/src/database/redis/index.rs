use crate::config::datatype::{ChatID, UserID, UserReqId};
use smartstring::alias::String;
use std::fmt::Write;

pub fn get_user_info_index(user_id: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "user:{}:info", user_id).ok();
    return str;
}

pub fn get_user_setting_index(user_id: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "user:{}:setting", user_id).ok();
    return str;
}

pub fn get_user_email_index(user_id: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "user:{}:mail", user_id).ok();
    return str;
}

pub fn get_user_password_index(user_id: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "user:{}:pass", user_id).ok();
    return str;
}

pub fn get_user_chats_index(user_id: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "user:{}:chats", user_id).ok();
    return str;
}

pub fn get_friend_pair_index(user_id1: UserID, user_id2: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "{}:{}", user_id1, user_id2).ok();
    return str;
}

pub fn get_chat_info_index(chat_id: ChatID) -> String {
    let mut str: String = String::new();
    write!(str, "chat:{}:info", chat_id).ok();
    return str;
}

pub fn get_chat_user_index(chat_id: ChatID, order: u8) -> String {
    let mut str: String = String::new();
    write!(str, "chat:{}:{}", chat_id, order).ok();
    return str;
}

pub fn get_chat_last_id_index(chat_id: ChatID) -> String {
    let mut str: String = String::new();
    write!(str, "chat:{}:last_id", chat_id).ok();
    return str;
}

pub fn get_chat_owner_index(chat_id: ChatID) -> String {
    let mut str: String = String::new();
    write!(str, "chat:{}:owner", chat_id).ok();
    return str;
}

pub fn get_chat_users_index(chat_id: ChatID) -> String {
    let mut str: String = String::new();
    write!(str, "chat:{}:users", chat_id).ok();
    return str;
}

pub fn get_chat_msgs_index(chat_id: ChatID) -> String {
    let mut str: String = String::new();
    write!(str, "chat:{}:msgs", chat_id).ok();
    return str;
}

pub fn get_req_info_index(req_id: UserReqId) -> String {
    let mut str: String = String::new();
    write!(str, "req:{}:info", req_id).ok();
    return str;
}

pub fn get_user_reqs_index(user_id: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "user:{}:reqs", user_id).ok();
    return str;
}

pub fn get_req_state_index(req_id: UserReqId) -> String {
    let mut str: String = String::new();
    write!(str, "req:{}:state", req_id).ok();
    return str;
}

pub fn get_chat_admins_index(chat_id: ChatID) -> String {
    let mut str: String = String::new();
    write!(str, "chat:{}:admins", chat_id).ok();
    return str;
}

pub fn get_user_notice_index(user_id: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "user:{}:not", user_id).ok();
    return str;
}

pub fn get_chat_notice_index(chat_id: ChatID) -> String {
    let mut str: String = String::new();
    write!(str, "chat:{}:notices", chat_id).ok();
    return str;
}

pub fn get_chat_last_notice_id_index(chat_id: ChatID) -> String {
    let mut str: String = String::new();
    write!(str, "chat:{}:last_notice_id", chat_id).ok();
    return str;
}

pub fn get_user_token_index(user_id: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "user:{}:token", user_id).ok();
    return str;
}

pub fn get_user_exist_index(user_id: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "user:{}:exist", user_id).ok();
    return str;
}

pub fn get_user_pre_join_index(user_id: UserID) -> String {
    let mut str: String = String::new();
    write!(str, "user:{}:pre_join", user_id).ok();
    return str;
}
