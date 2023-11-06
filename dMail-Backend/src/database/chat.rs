use crate::chat::*;
use crate::config::datatype::*;

use crate::user::*;

use super::redis;

/// 创建一个群聊，传入创建者ID，传出群聊ID
pub async fn create_group_chat(
    creator_id: UserID,
    data: UserCreateGroupChatData,
) -> Result<ChatID, ()> {
    return redis::create_group_chat(creator_id, data).await;
}

/// 向一个群聊中添加用户
pub async fn add_user_to_chat(chat_id: ChatID, user_id: UserID) -> Result<(), ()> {
    return redis::add_user_to_group_chat(chat_id, user_id).await;
}

/// 向消息列表写入一条Message
/// 成功时，返回序列化后的ChatMessage 与 In chat ID
pub async fn write_message_to_chat(
    r#type: &str,
    serialized_content: String,
    chat_id: ChatID,
    user_id: UserID,
) -> Result<(SerializedChatMessage, MessageID, Timestamp), ()> {
    return redis::write_message_to_chat(r#type, serialized_content, chat_id, user_id).await;
}

/// 检查用户是否能在Chat中发表信息
/// 这个阶段不可以把群聊的列表拉下来
pub async fn check_user_can_send_in_chat(
    user_id: UserID,
    chat_id: ChatID,
) -> Result<ChatType, UserSendMessageResponseState> {
    return redis::check_user_can_send_in_chat(user_id, chat_id).await;
}

/// 获得一个Chat中所有的用户
pub async fn get_chat_user_list(chat_id: ChatID) -> Result<ChatMembers, ()> {
    return redis::get_chat_user_list(chat_id).await;
}

/// 获得一个Chat中从start_msg_id到end_msg_id的所有消息
pub async fn get_messages_in_chat(
    chat_id: ChatID,
    start_msg_id: MessageID,
    end_msg_id: Option<MessageID>,
) -> Result<Vec<SerializedChatMessage>, ()> {
    return redis::get_messages_in_chat(chat_id, start_msg_id, end_msg_id).await;
}

/// 获取一个Chat最后一条消息
pub async fn get_chats_last_messages(
    chats: &Vec<(ChatID, MessageID)>,
    max_messages_per_chat: u8,
) -> Result<Vec<SerializedChatMessage>, ()> {
    return redis::get_chats_last_messages(chats, max_messages_per_chat).await;
}

/// 获得一个Chat的基本信息
pub async fn get_chat_info(chat_id: ChatID) -> Result<Option<SerializedChatInfo>, ()> {
    return redis::get_chat_info(chat_id).await;
}

/// 撤回消息
pub async fn revoke_message(
    chat_id: ChatID,
    in_chat_id: MessageID,
    sender_id: UserID,
    timestamp: Timestamp,
) -> Result<(), ()> {
    return redis::revoke_message(chat_id, in_chat_id, sender_id, timestamp).await;
}

/// 检查进群邀请是否出错
pub async fn check_group_invitation_error(
    sender_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), GroupInvitationError> {
    return redis::check_group_invitation_error(sender_id, receiver_id, chat_id).await;
}
/// 检查受邀进群是否出错
pub async fn check_invited_join_group_error(
    sender_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), InvitedJoinGroupError> {
    return redis::check_invited_join_group_error(sender_id, receiver_id, chat_id).await;
}
/// 退群
pub async fn quit_group_chat(user_id: UserID, chat_id: ChatID) -> UserQuitGroupChatResponse {
    return redis::quit_group_chat(user_id, chat_id).await;
}

/// 设为管理员
pub async fn set_as_admin(user_id: UserID, chat_id: ChatID) -> UserSetGroupAdminResponse {
    return redis::set_as_admin(user_id, chat_id).await;
}

/// 检查进群申请是否出错
pub async fn check_join_group_error(
    user_id: UserID,
    chat_id: ChatID,
) -> Result<(), JoinGroupError> {
    return redis::check_join_group_error(user_id, chat_id).await;
}

/// 获得一个群聊的所有管理员
pub async fn get_chat_admins_list(chat_id: ChatID) -> Result<UserRequestHandler, ()> {
    return redis::get_chat_admins_list(chat_id).await;
}

/// 获得一个群聊的群主
pub async fn get_chat_owner(chat_id: ChatID) -> Result<UserID, ()> {
    return redis::get_chat_owner(chat_id).await;
}

/// 检查一个用户是否是某群群主
pub async fn check_user_is_owner(user_id: UserID, chat_id: ChatID) -> Result<bool, ()> {
    return redis::check_user_is_owner(user_id, chat_id).await;
}

/// 检查一个用户是否是某群管理员
pub async fn check_user_is_admin(user_id: UserID, chat_id: ChatID) -> Result<bool, ()> {
    return redis::check_user_is_admin(user_id, chat_id).await;
}

/// 群主转移
pub async fn owner_transfer(user_id: UserID, chat_id: ChatID) -> UserGroupOwnerTransferResponse {
    return redis::owner_transfer(user_id, chat_id).await;
}

/// 添加群公告
pub async fn add_group_notice(
    user_id: UserID,
    chat_id: ChatID,
    client_id: ClientID,
    notice: String,
) -> UserSendGroupNoticeResponse {
    return redis::add_group_notice(user_id, chat_id, client_id, notice).await;
}

/// 获取群公告
pub async fn pull_group_notice(
    chat_id: ChatID,
    last_notice_id: NoticeID,
) -> UserPullGroupNoticeResponse {
    return redis::pull_group_notice(chat_id, last_notice_id).await;
}

/// 更新群聊基本信息
pub async fn update_group_info(
    chat_id: ChatID,
    data: UserUpdateGroupContent,
) -> UserUpdateGroupInfoResponse {
    return redis::update_group_info(chat_id, data).await;
}

/// 取消管理员权限
pub async fn unset_admin(user_id: UserID, chat_id: ChatID) -> UserUnsetGroupAdminResponse {
    return redis::unset_admin(user_id, chat_id).await;
}

/// 检查某聊天是否是群聊
pub async fn check_is_group(chat_id: ChatID) -> Result<bool, ()> {
    return redis::check_is_group_chat(chat_id).await;
}

/// 获取群聊中已读某条消息的用户列表
pub async fn get_user_read_in_group(
    chat_id: ChatID,
    in_chat_id: MessageID,
) -> UserGetUserReadInGroupResponse {
    return redis::get_user_read_in_group(chat_id, in_chat_id).await;
}

/// 获取私聊中用户已经读到的消息id
pub async fn get_user_read_in_private(
    user_id: UserID,
    chat_id: ChatID,
) -> UserGetUserReadInPrivateResponse {
    return redis::get_user_read_in_private(user_id, chat_id).await;
}

/// 获取私聊中的用户列表
pub async fn get_private_chat_user_list(chat_id: ChatID) -> Result<Option<(UserID, UserID)>, ()> {
    return redis::get_private_chat_user_list(chat_id).await;
}
