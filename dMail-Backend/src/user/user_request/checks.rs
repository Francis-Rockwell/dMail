/*!
 检查与请求有关的可能错误
*/

use crate::{
    config::datatype::{ChatID, UserID},
    database,
};

use super::{
    GroupInvitationError, InvitedJoinGroupError, MakeFriendError, UserRequestError,
    UserRequsetContent,
};

/** `check_error` 检查请求是否有误
*/
pub async fn check_error(
    content: &UserRequsetContent,
    sender_id: UserID,
) -> Result<(), UserRequestError> {
    return match content {
        UserRequsetContent::JoinGroup { chat_id } => {
            check_join_group_error(sender_id, chat_id).await
        }
        UserRequsetContent::MakeFriend {
            receiver_id: user_id,
        } => check_make_frined_error(sender_id, *user_id).await,

        UserRequsetContent::GroupInvitation {
            chat_id,
            receiver_id,
        } => check_group_invitation_error(sender_id, *receiver_id, *chat_id).await,
        UserRequsetContent::InvitedJoinGroup {
            inviter_id,
            chat_id,
        } => check_invited_join_group_err(*inviter_id, sender_id, *chat_id).await,
    };
}

async fn check_make_frined_error(
    sender_id: UserID,
    receiver_id: UserID,
) -> Result<(), UserRequestError> {
    // TODO

    if sender_id == receiver_id {
        return Err(UserRequestError::MakeFriend(MakeFriendError::SameUser));
    }

    if let Err(err) = database::check_make_friend_error(sender_id, receiver_id).await {
        return Err(UserRequestError::MakeFriend(err));
    }

    return Ok(());
}

async fn check_join_group_error(
    sender_id: UserID,
    chat_id: &ChatID,
) -> Result<(), UserRequestError> {
    if let Err(err) = database::check_join_group_error(sender_id, *chat_id).await {
        return Err(UserRequestError::JoinGroup(err));
    }
    return Ok(());
}

async fn check_group_invitation_error(
    sender_id: UserID,
    receiver_id: UserID,
    chat_id: ChatID,
) -> Result<(), UserRequestError> {
    if sender_id == receiver_id {
        return Err(UserRequestError::GroupInvation(
            GroupInvitationError::SameUser,
        ));
    }

    if let Err(err) = database::check_group_invitation_error(sender_id, receiver_id, chat_id).await
    {
        return Err(UserRequestError::GroupInvation(err));
    }

    return Ok(());
}

async fn check_invited_join_group_err(
    inviter_id: UserID,
    user_id: UserID,
    chat_id: ChatID,
) -> Result<(), UserRequestError> {
    if inviter_id == user_id {
        return Err(UserRequestError::InvitedJoinGroup(
            InvitedJoinGroupError::SameUser,
        ));
    }

    if let Err(err) = database::check_invited_join_group_error(inviter_id, user_id, chat_id).await {
        return Err(UserRequestError::InvitedJoinGroup(err));
    }

    return Ok(());
}
