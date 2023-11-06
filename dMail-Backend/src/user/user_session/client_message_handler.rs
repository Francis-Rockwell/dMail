/*!
客户端与服务端之间通信的直接处理函数
*/

use std::future::Future;

use actix::{
    dev::ToEnvelope, fut, Actor, ActorContext, ActorFutureExt, AsyncContext, Recipient, WrapFuture,
};

use aes_gcm::{Aes128Gcm, KeyInit};

use chrono::Utc;
use rand::thread_rng;

use crate::{
    chat::{send_admin_message_to_group_chat, ChatMembers, ChatMessage, ChatType},
    config::{
        config::PWD_PATTERN,
        datatype::{ChatID, EmailCodeValue, Timestamp, UploadId, UserID},
        Config,
    },
    database::{
        self, check_user_in_chat, check_user_is_admin, check_user_is_owner, get_chat_user_list,
    },
    oss::{self, ObjectUploadRequest},
    server::{
        email::check_and_consume_email_code,
        server_state::{user_sessions, workers_handle, UserSessionGetter},
    },
    user::*,
    utils::{self, base64, rsa::PubKeyHelper},
};

use super::client_message_data::*;
use super::{
    actor_message_handler::ActorMessageHandler,
    protocol::{ClientToServerMessage, DataChecker, ServerMessageSender, ServerToClientMessage},
    UserSession, UserSessionActorMessage, UserSessionDataRc, UserSessionState,
};

use user_request::UserRequsetContent;

// 这里原本打算用Unstable特性，trait别名，但是使用了以后发现编辑器插件傻了（）
// 希望早日变成Stable
pub trait ClientMessageHandler
where
    Self: Actor + UserSession + ActorMessageHandler,
    Self::Context: ServerMessageSender<Self> + ToEnvelope<Self, UserSessionActorMessage>,
{
    fn handle_client_message(&mut self, msg: ClientToServerMessage, ctx: &mut Self::Context) {
        // TODO : 对用户请求速度进行限制，避免被攻击

        // 访问除了SetPubkey, Login, Register外的消息均需要登陆
        // 因此在编写其他接口的函数时可以假设一定已经登陆
        if msg != ClientToServerMessage::Close {
            // 为什么要加括号？
            // `Borrow Checker`仍不是很智能（）
            let cur_state = &self.get_data().state;
            if *cur_state == UserSessionState::Started && !msg.is_set_pub_key() {
                ctx.send_server_message(
                    &ServerToClientMessage::SetConnectionPubKeyResponse(
                        UserSetPubKeyResponse::NeedSetPubKey,
                    ),
                    None,
                );
                return;
            } else if *cur_state == UserSessionState::Approved
                && !msg.is_login()
                && !msg.is_register()
            {
                ctx.send_server_message(
                    &ServerToClientMessage::LoginResponse(UserLoginResponse::NeedLogin),
                    self.get_cipher(),
                );
                return;
            }
        }

        let user_id = self.get_data().user_id;

        // TODO : match的编译行为不太确定，没法保证会编译成跳转表
        // 这里需要性能测试，如果不够理想可能需要自己实现跳转，并用Unsafe提取参数
        match msg {
            ClientToServerMessage::Ping => {
                ctx.send_server_message(&ServerToClientMessage::Pong, None)
            }
            ClientToServerMessage::Pong => {}
            ClientToServerMessage::Close => ctx.stop(),
            ClientToServerMessage::SetConnectionPubKey(key) => {
                ctx.send_server_message(&set_connection_pub_key(self, key), None);
            }
            ClientToServerMessage::Register(data) => {
                self.excute_and_send_response_wait(ctx, register(data));
            }
            ClientToServerMessage::Login(data) => self.excute_and_send_response_wait(
                ctx,
                login(self.get_data_rc(), data, ctx.address().recipient()),
            ),
            ClientToServerMessage::SendMessage(msg) => {
                self.excute_and_send_response(ctx, send_message(user_id.unwrap(), msg))
            }
            ClientToServerMessage::Pull(data) => {
                // TODO : 对Pull这种非常耗时的请求单独进行请求速度限制
                workers_handle.spawn(user_pull(data, user_id.unwrap(), ctx.address().recipient()));
            }
            ClientToServerMessage::SendRequest(req) => {
                self.excute_and_send_response(ctx, send_request(user_id.unwrap(), req))
            }
            ClientToServerMessage::SolveRequest(data) => {
                self.excute_and_send_response(ctx, solve_request(user_id.unwrap(), data))
            }
            ClientToServerMessage::GetUserInfo(user_id) => {
                self.excute_and_send_response(ctx, get_user_info(user_id))
            }
            ClientToServerMessage::CreateGroupChat(data) => {
                self.excute_and_send_response(ctx, create_group_chat(user_id.unwrap(), data))
            }
            ClientToServerMessage::GetMessages(data) => {
                self.excute_and_send_response(ctx, get_messages(data))
            }
            ClientToServerMessage::GetChatInfo(chat_id) => {
                self.excute_and_send_response(ctx, get_chat_info(chat_id))
            }
            ClientToServerMessage::UpdateUserInfo(data) => {
                self.excute_and_send_response(ctx, update_user_info(user_id.unwrap(), data))
            }
            ClientToServerMessage::Unfriend(friend_id) => {
                self.excute_and_send_response(ctx, unfriend(user_id.unwrap(), friend_id))
            }
            ClientToServerMessage::SetUserSetting(content) => {
                self.excute_and_send_response(ctx, set_user_setting(user_id.unwrap(), content))
            }
            ClientToServerMessage::SetAlreadyRead(data) => {
                self.excute_and_send_response(ctx, set_user_already_read(user_id.unwrap(), data))
            }
            ClientToServerMessage::UploadFileRequest(req) => {
                self.excute_and_send_response(ctx, upload_file_req(user_id.unwrap(), req))
            }
            ClientToServerMessage::FileUploaded(upload_id) => {
                self.excute_and_send_response(ctx, file_uploaded(user_id.unwrap(), upload_id))
            }
            ClientToServerMessage::GetFileUrl(hash) => {
                self.excute_and_send_response(ctx, get_file_pub_url(hash))
            }
            ClientToServerMessage::RevokeMessage(data) => {
                self.excute_and_send_response(ctx, revoke_message(user_id.unwrap(), data))
            }
            ClientToServerMessage::QuitGroupChat(data) => {
                self.excute_and_send_response(ctx, quit_group_chat(user_id.unwrap(), data))
            }
            ClientToServerMessage::SetGroupAdmin(data) => {
                self.excute_and_send_response(ctx, set_as_admin(user_id.unwrap(), data))
            }
            ClientToServerMessage::GroupOwnerTransfer(data) => {
                self.excute_and_send_response(ctx, owner_transfer(user_id.unwrap(), data))
            }
            ClientToServerMessage::SendGroupNotice(data) => {
                self.excute_and_send_response(ctx, group_notice(user_id.unwrap(), data))
            }
            ClientToServerMessage::RemoveGroupMember(data) => {
                self.excute_and_send_response(ctx, remove_member(user_id.unwrap(), data))
            }
            ClientToServerMessage::GetGroupUsers(chat_id) => {
                self.excute_and_send_response(ctx, get_group_users(user_id.unwrap(), chat_id))
            }
            ClientToServerMessage::UpdateGroupInfo(data) => {
                self.excute_and_send_response(ctx, update_group_info(user_id.unwrap(), data))
            }
            ClientToServerMessage::PullGroupNotice(data) => {
                self.excute_and_send_response(ctx, pull_group_notice(user_id.unwrap(), data))
            }
            ClientToServerMessage::UnsetGroupAdmin(data) => {
                self.excute_and_send_response(ctx, unset_group_admin(user_id.unwrap(), data))
            }
            ClientToServerMessage::GetGroupOwner(chat_id) => {
                self.excute_and_send_response(ctx, get_group_owner(user_id.unwrap(), chat_id))
            }
            ClientToServerMessage::GetGroupAdmin(chat_id) => {
                self.excute_and_send_response(ctx, get_group_admin(user_id.unwrap(), chat_id))
            }
            ClientToServerMessage::MediaCall(data) => {
                self.excute_and_send_response(ctx, media_call(user_id.unwrap(), data))
            }
            ClientToServerMessage::MediaCallAnswer(data) => {
                self.excute(ctx, media_call_answer(user_id.unwrap(), data))
            }
            ClientToServerMessage::MediaIceCandidate(data) => {
                self.excute(ctx, media_ice_candidate(user_id.unwrap(), data))
            }
            ClientToServerMessage::MediaCallStop(data) => {
                self.excute(ctx, media_call_stop(user_id.unwrap(), data))
            }
            ClientToServerMessage::GetUserID(name) => {
                self.excute_and_send_response(ctx, get_user_id(name))
            }
            ClientToServerMessage::GetUserReadInGroup(data) => {
                workers_handle.spawn(get_user_read_in_group(
                    user_id.unwrap(),
                    data,
                    ctx.address().recipient(),
                ));
            }
            ClientToServerMessage::GetUserReadInPrivate(chat_id) => self
                .excute_and_send_response(ctx, get_user_read_in_private(user_id.unwrap(), chat_id)),
            ClientToServerMessage::LogOff(email_code) => self.excute_and_send_response(
                ctx,
                user_log_off(user_id.unwrap(), email_code, ctx.address().recipient()),
            ),
            ClientToServerMessage::ApplyForToken => {
                self.excute_and_send_response(ctx, user_apply_for_token(user_id.unwrap()))
            }
        };
    }

    fn excute_and_send_response<F>(&self, ctx: &mut Self::Context, fut: F)
    where
        F: Future<Output = ServerToClientMessage> + 'static,
    {
        ctx.spawn(fut.into_actor(self).then(|res, self_actor, ctx| {
            ctx.send_server_message(&res, self_actor.get_cipher());
            fut::ready(())
        }));
    }

    fn excute_and_send_response_wait<F>(&self, ctx: &mut Self::Context, fut: F)
    where
        F: Future<Output = ServerToClientMessage> + 'static,
    {
        ctx.wait(fut.into_actor(self).then(|res, self_actor, ctx| {
            ctx.send_server_message(&res, self_actor.get_cipher());
            fut::ready(())
        }));
    }

    fn excute<F>(&self, ctx: &mut Self::Context, fut: F)
    where
        F: Future<Output = ()> + 'static,
    {
        ctx.spawn(fut.into_actor(self));
    }
}

pub async fn register(register_data: UserRegisterData) -> ServerToClientMessage {
    // TODO : 数据合法性检查
    if let Err(res) = register_data.check_data() {
        return ServerToClientMessage::RegisterResponse(res);
    }
    if check_and_consume_email_code(&register_data.email, register_data.email_code.clone()) == false
    {
        return ServerToClientMessage::RegisterResponse(UserRegisterResponse::EmailCodeError);
    }

    let res = database::user_register(register_data).await;

    // For Test
    // if let UserRegisterResponse::Success { user_id } = res {
    //     database::add_user_to_chat(1, user_id).await.ok();
    //     database::add_user_to_chat(2, user_id).await.ok();
    //     database::add_user_to_chat(3, user_id).await.ok();
    // }

    return ServerToClientMessage::RegisterResponse(res);
}

fn set_connection_pub_key<T: UserSession>(
    session: &mut T,
    base64_pkcs1_pub_key: String,
) -> ServerToClientMessage {
    {
        // data需要在await前释放，不然会panic
        let mut data = session.get_mut_data();

        if data.state == UserSessionState::Approved {
            return ServerToClientMessage::SetConnectionPubKeyResponse(
                crate::user::UserSetPubKeyResponse::HasApproved,
            );
        }

        if let Ok(pub_key) = utils::rsa::get_pub_key_from_base64_pkcs1_pem(base64_pkcs1_pub_key) {
            data.user_pub_key = Some(pub_key);
        } else {
            return ServerToClientMessage::SetConnectionPubKeyResponse(
                crate::user::UserSetPubKeyResponse::PubKeyError,
            );
        }
    }

    let sym_key = Aes128Gcm::generate_key(thread_rng());
    let encoded_sym_key = session
        .get_data()
        .user_pub_key
        .as_ref()
        .unwrap()
        .encrypt_to_base64(base64::encode(sym_key).as_bytes())
        .unwrap();

    session.on_sym_key_set_success(Aes128Gcm::new_from_slice(&sym_key).unwrap());

    return ServerToClientMessage::SetConnectionSymKey(encoded_sym_key);
}

async fn login(
    session_data_rc: UserSessionDataRc,
    login_data: UserLoginData,
    self_recipient: Recipient<UserSessionActorMessage>,
) -> ServerToClientMessage {
    if let Err(res) = login_data.check_data() {
        // TODO : 数据合法性检查
        return ServerToClientMessage::LoginResponse(res);
    }

    {
        let data = session_data_rc.borrow();

        match data.state {
            user_session::UserSessionState::Started => {
                return ServerToClientMessage::LoginResponse(UserLoginResponse::Unapproved)
            }
            user_session::UserSessionState::Approved => {}
            user_session::UserSessionState::Logged => {
                return ServerToClientMessage::LoginResponse(UserLoginResponse::UserLogged)
            }
        }
    }

    let res = if let Some(input_code) = login_data.email_code {
        // 使用验证码登录
        if check_and_consume_email_code(&login_data.email, input_code) {
            let res = database::get_user_id_by_email(&login_data.email).await;
            if res.is_err() {
                return ServerToClientMessage::LoginResponse(UserLoginResponse::ServerError);
            }
            let opt = res.unwrap();
            if let Some(id) = opt {
                UserLoginResponse::Success { user_id: id }
            } else {
                UserLoginResponse::UserNotFound
            }
        } else {
            UserLoginResponse::EmailCodeError
        }
    } else if login_data.password.is_some() {
        // 使用密码登录
        database::user_login_with_password(login_data).await
    } else {
        database::user_login_with_token(login_data).await
    };

    let user_id = match &res {
        UserLoginResponse::Success { user_id } => user_id.to_owned(),
        _ => return ServerToClientMessage::LoginResponse(res),
    };

    // 处理第二次登陆逻辑，是挤掉还是登不上？
    if let Some(_) = user_sessions.get_user_session(user_id) {
        return ServerToClientMessage::LoginResponse(UserLoginResponse::UserLogged);
    }

    session_data_rc
        .borrow_mut()
        .on_login_success(user_id, self_recipient);

    return ServerToClientMessage::LoginResponse(UserLoginResponse::Success { user_id: user_id });
}

pub async fn send_message(sender_id: UserID, msg: UserSendMessageData) -> ServerToClientMessage {
    // TODO : 数据合法性检查

    // 此处一定已经登陆
    let client_id = msg.client_id;
    let self_id = sender_id;
    let chat_id = msg.chat_id;

    let chat_type = match database::check_user_can_send_in_chat(self_id, msg.chat_id).await {
        Ok(data) => data,
        Err(state) => {
            return ServerToClientMessage::SendMessageResponse(UserSendMessageResponse {
                state,
                client_id,
                in_chat_id: None,
                timestamp: None,
                chat_id,
            });
        }
    };

    let mut user_ids = vec![];
    if ChatMessageType::MentionText == msg.r#type {
        let users_result = serde_json::from_str::<MentionTextType>(&msg.serialized_content);
        if users_result.is_err() {
            return ServerToClientMessage::SendMessageResponse(UserSendMessageResponse {
                state: UserSendMessageResponseState::ContentError,
                client_id,
                chat_id,
                in_chat_id: None,
                timestamp: None,
            });
        }
        user_ids = users_result.unwrap().user_ids;
    }

    let (chat_msg, in_chat_id, timestamp) = match database::write_message_to_chat(
        msg.r#type.get_str(),
        serde_json::to_string::<String>(&msg.serialized_content).unwrap(),
        msg.chat_id,
        self_id,
    )
    .await
    {
        Ok(data) => data,
        Err(_) => {
            return ServerToClientMessage::SendMessageResponse(UserSendMessageResponse {
                state: UserSendMessageResponseState::DatabaseError,
                client_id,
                in_chat_id: None,
                timestamp: None,
                chat_id,
            });
        }
    };

    if ChatMessageType::MentionText == msg.r#type {
        let notice = UserNotice::Mentioned {
            chat_id,
            in_chat_id,
            timestamp,
        };
        let serialized_notice = serde_json::to_string(&notice).unwrap();
        if send_notice(user_ids, serialized_notice, timestamp)
            .await
            .is_err()
        {
            return ServerToClientMessage::SendMessageResponse(UserSendMessageResponse {
                state: UserSendMessageResponseState::SendNoticeError,
                client_id,
                in_chat_id: None,
                timestamp: None,
                chat_id,
            });
        }
    }

    match chat_type {
        ChatType::Private(ids) => {
            send_msg_to_online_user_in_private_chat(sender_id, chat_msg, ids);
        }
        ChatType::Group(num) => {
            if num as u8
                > Config::get()
                    .protocol
                    .worker_send_messages_member_num_threshold
            {
                workers_handle.spawn(send_msg_to_online_users_in_chat(
                    chat_msg, sender_id, chat_id,
                ));
            } else {
                send_msg_to_online_users_in_chat(chat_msg, sender_id, chat_id).await;
            }
        }
    }

    return ServerToClientMessage::SendMessageResponse(UserSendMessageResponse {
        state: UserSendMessageResponseState::Success,
        client_id,
        in_chat_id: Some(in_chat_id),
        timestamp: Some(timestamp),
        chat_id,
    });
}

pub async fn send_request(user_id: UserID, data: UserSendRequestData) -> ServerToClientMessage {
    let client_id = data.client_id;

    if let Err(err) = check_error(&data.content, user_id).await {
        return ServerToClientMessage::SendRequestResponse(UserSendRequestResponse {
            req_id: None,
            client_id,
            state: UserSendRequestState::RequestError(err),
        });
    }
    let handlers = match database::get_handlers_of_request(&data.content).await {
        Ok(handler) => handler,
        Err(_) => {
            return ServerToClientMessage::SendRequestResponse(UserSendRequestResponse {
                req_id: None,
                client_id,
                state: UserSendRequestState::DatabaseError,
            });
        }
    };
    let (serialized_req, req_info) =
        match database::write_user_request(user_id, data, &handlers).await {
            Ok(req) => req,
            Err(_) => {
                return ServerToClientMessage::SendRequestResponse(UserSendRequestResponse {
                    req_id: None,
                    client_id,
                    state: UserSendRequestState::DatabaseError,
                });
            }
        };
    if let Err(_) = on_request_send(&req_info).await {
        return ServerToClientMessage::SendRequestResponse(UserSendRequestResponse {
            req_id: None,
            client_id,
            state: UserSendRequestState::DatabaseError,
        });
    }
    // TODO : 根据数量卸载到Workers里
    send_msg_to_online_handlers(ServerToClientMessage::Request(serialized_req), handlers).await;
    return ServerToClientMessage::SendRequestResponse(UserSendRequestResponse {
        req_id: Some(req_info.req_id),
        client_id,
        state: UserSendRequestState::Success,
    });
}

pub async fn solve_request(user_id: UserID, data: UserSolveRequestData) -> ServerToClientMessage {
    let req = match database::get_user_request(data.req_id).await {
        Ok(opt) => match opt {
            Some(req) => req,
            None => {
                return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
                    state: UserSolveRequestState::RequestNotFound,
                    req_id: data.req_id,
                })
            }
        },
        Err(_) => {
            return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
                state: UserSolveRequestState::DatabaseError,
                req_id: data.req_id,
            })
        }
    };

    if data.answer == UserRequestState::Unsolved {
        return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
            state: UserSolveRequestState::AnswerUnsolved,
            req_id: data.req_id,
        });
    }
    let handlers = match database::get_handlers_of_request(&req.info.content).await {
        Ok(handler) => handler,
        Err(_) => {
            return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
                state: UserSolveRequestState::DatabaseError,
                req_id: data.req_id,
            });
        }
    };

    if handlers.is_handler(user_id) == false {
        return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
            state: UserSolveRequestState::NotHandler,
            req_id: data.req_id,
        });
    }

    // TODO : 这里可以不用再查询请求状态，可以做优化
    if let Err(err) = database::set_user_request_state(data.req_id, data.answer).await {
        return ServerToClientMessage::SolveRequestResponse(err);
    }

    if let Err(_) = if data.answer == UserRequestState::Approved {
        if database::check_user_exist(req.info.sender_id)
            .await
            .is_err()
        {
            send_msg_to_online_handlers(
                ServerToClientMessage::RequestStateUpdate(UserRequsetStateUpdated {
                    req_id: data.req_id,
                    state: UserRequestState::Approved,
                }),
                UserRequestHandler::One(user_id),
            )
            .await;
            send_msg_to_online_handlers(
                ServerToClientMessage::RequestMessage(RequestMessageResponse {
                    req_id: data.req_id,
                    r#type: RequstMessageType::UserLogOff,
                }),
                UserRequestHandler::One(user_id),
            )
            .await;
            return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
                state: UserSolveRequestState::Success,
                req_id: data.req_id,
            });
        }
        match req.info.content {
            UserRequsetContent::InvitedJoinGroup {
                inviter_id: _,
                chat_id,
            } => {
                let check = database::check_user_in_chat(req.info.sender_id, chat_id).await;
                if check.is_err() {
                    return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
                        state: UserSolveRequestState::DatabaseError,
                        req_id: data.req_id,
                    });
                }
                if check.unwrap() {
                    send_msg_to_online_handlers(
                        ServerToClientMessage::RequestMessage(RequestMessageResponse {
                            req_id: data.req_id,
                            r#type: RequstMessageType::UserAlreadyInChat,
                        }),
                        UserRequestHandler::One(user_id),
                    )
                    .await;
                    Ok(())
                } else {
                    on_request_approved(&req.info).await
                }
            }
            UserRequsetContent::GroupInvitation {
                receiver_id,
                chat_id,
            } => {
                let check = database::check_user_in_chat(receiver_id, chat_id).await;
                if check.is_err() {
                    return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
                        state: UserSolveRequestState::DatabaseError,
                        req_id: data.req_id,
                    });
                }
                if check.unwrap() {
                    Ok(())
                } else {
                    on_request_approved(&req.info).await
                }
            }
            UserRequsetContent::JoinGroup { chat_id } => {
                let check = database::check_user_in_chat(req.info.sender_id, chat_id).await;
                if check.is_err() {
                    return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
                        state: UserSolveRequestState::DatabaseError,
                        req_id: data.req_id,
                    });
                }
                if check.unwrap() {
                    send_msg_to_online_handlers(
                        ServerToClientMessage::RequestMessage(RequestMessageResponse {
                            req_id: data.req_id,
                            r#type: RequstMessageType::UserAlreadyInChat,
                        }),
                        UserRequestHandler::One(user_id),
                    )
                    .await;
                    Ok(())
                } else {
                    on_request_approved(&req.info).await
                }
            }
            _ => on_request_approved(&req.info).await,
        }
    } else {
        if database::check_user_exist(req.info.sender_id)
            .await
            .is_err()
        {
            send_msg_to_online_handlers(
                ServerToClientMessage::RequestStateUpdate(UserRequsetStateUpdated {
                    req_id: data.req_id,
                    state: UserRequestState::Refused,
                }),
                UserRequestHandler::One(user_id),
            )
            .await;
            send_msg_to_online_handlers(
                ServerToClientMessage::RequestMessage(RequestMessageResponse {
                    req_id: data.req_id,
                    r#type: RequstMessageType::UserLogOff,
                }),
                UserRequestHandler::One(user_id),
            )
            .await;
            return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
                state: UserSolveRequestState::Success,
                req_id: data.req_id,
            });
        }
        on_request_refused(&req.info).await
    } {
        return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
            state: UserSolveRequestState::DatabaseError,
            req_id: data.req_id,
        });
    }

    // TODO : 换用arc

    // 测试用Start
    let msg = ServerToClientMessage::RequestStateUpdate(UserRequsetStateUpdated {
        req_id: req.info.req_id,
        state: data.answer,
    });
    user_sessions.do_send_message_to(
        req.info.sender_id,
        UserSessionActorMessage::SendServerMessage(msg),
    );
    send_msg_to_online_handlers(
        ServerToClientMessage::RequestStateUpdate(UserRequsetStateUpdated {
            req_id: req.info.req_id,
            state: data.answer,
        }),
        handlers,
    )
    .await;
    // 测试用 End

    return ServerToClientMessage::SolveRequestResponse(UserSolveRequestResponse {
        state: UserSolveRequestState::Success,
        req_id: data.req_id,
    });
}

pub async fn create_group_chat(
    user_id: UserID,
    data: UserCreateGroupChatData,
) -> ServerToClientMessage {
    if data.name.len() > Config::get().user.max_user_name_length.try_into().unwrap() {
        return ServerToClientMessage::CreateGroupChatResponse(
            UserCreateGroupChatResponse::ChatNameFormatError,
        );
    }
    match database::create_group_chat(user_id, data).await {
        Ok(id) => {
            send_admin_message_to_group_chat(id, "建立群聊成功".to_string()).await;
            ServerToClientMessage::CreateGroupChatResponse(UserCreateGroupChatResponse::Success {
                chat_id: id,
            })
        }
        Err(_) => ServerToClientMessage::CreateGroupChatResponse(
            UserCreateGroupChatResponse::DatabaseError,
        ),
    }
}

pub async fn get_user_info(user_id: UserID) -> ServerToClientMessage {
    return ServerToClientMessage::GetUserInfoResponse(database::get_user_info(user_id).await);
}

pub async fn get_messages(data: UserGetMessagesData) -> ServerToClientMessage {
    // TODO : 判断请求id范围是否合法

    // TODO : 判断用户是否在聊天中

    match database::get_messages_in_chat(data.chat_id, data.start_id, data.end_id).await {
        Ok(msgs) => ServerToClientMessage::Messages(msgs),
        Err(_) => ServerToClientMessage::DatabaseError,
    }
}

pub async fn get_chat_info(chat_id: ChatID) -> ServerToClientMessage {
    match database::get_chat_info(chat_id).await {
        Err(_) => ServerToClientMessage::DatabaseError,
        Ok(opt) => match opt {
            Some(serialized) => ServerToClientMessage::Chat(serialized),
            None => ServerToClientMessage::NotFound,
        },
    }
}

pub async fn set_user_setting(user_id: UserID, user_setting: String) -> ServerToClientMessage {
    return ServerToClientMessage::SetUserSettingResponse(
        database::set_user_setting(user_id, user_setting).await,
    );
}

pub async fn update_user_info(user_id: UserID, data: UserUpdateData) -> ServerToClientMessage {
    match data {
        UserUpdateData::Password {
            new_password,
            email_code,
        } => {
            let email = database::get_user_email(user_id).await;
            if let Ok(email) = email {
                let check_email = check_and_consume_email_code(&email, email_code);
                if check_email {
                    if !PWD_PATTERN.is_match(&new_password) {
                        return ServerToClientMessage::UpdateUserInfoResponse(
                            UserUpdateResponse::PasswordFormatError,
                        );
                    }
                    return ServerToClientMessage::UpdateUserInfoResponse(
                        database::update_user_password(user_id, new_password).await,
                    );
                } else {
                    return ServerToClientMessage::UpdateUserInfoResponse(
                        UserUpdateResponse::EmailCodeError,
                    );
                }
            } else {
                return ServerToClientMessage::UpdateUserInfoResponse(
                    UserUpdateResponse::ServerError,
                );
            }
        }
        UserUpdateData::UserName { new_name } => {
            if new_name.len() > Config::get().user.max_user_name_length as usize {
                return ServerToClientMessage::UpdateUserInfoResponse(
                    UserUpdateResponse::UserNameFormatError,
                );
            }
            return ServerToClientMessage::UpdateUserInfoResponse(
                database::update_user_name(user_id, new_name).await,
            );
        }
        UserUpdateData::AvaterHash { new_hash } => {
            // AvaterHashFormatError
            return ServerToClientMessage::UpdateUserInfoResponse(
                database::update_user_avater(user_id, new_hash).await,
            );
        }
    }
}

pub async fn unfriend(user_id: UserID, friend_id: UserID) -> ServerToClientMessage {
    match database::unfriend(user_id, friend_id).await {
        UserUnfriendResponse::Success { chat_id } => {
            send_delete_chat_msg(friend_id, chat_id).await;
            return ServerToClientMessage::UnfriendResponse(UserUnfriendResponse::Success {
                chat_id,
            });
        }
        UserUnfriendResponse::ServerError => {
            return ServerToClientMessage::UnfriendResponse(UserUnfriendResponse::ServerError)
        }
        UserUnfriendResponse::NotFriend => {
            return ServerToClientMessage::UnfriendResponse(UserUnfriendResponse::NotFriend)
        }
    }
}

pub async fn quit_group_chat(user_id: UserID, chat_id: ChatID) -> ServerToClientMessage {
    if let Ok(check) = database::check_user_is_owner(user_id, chat_id).await {
        if check {
            return ServerToClientMessage::QuitGroupChatResponse(
                UserQuitGroupChatResponse::NoPermission,
            );
        }
    } else {
        return ServerToClientMessage::QuitGroupChatResponse(
            UserQuitGroupChatResponse::DatabaseError,
        );
    }
    let quit_result = database::quit_group_chat(user_id, chat_id).await;
    match quit_result {
        UserQuitGroupChatResponse::Success { chat_id } => {
            let user_name = match database::get_user_info(user_id).await {
                GetUserInfoResponse::Success(user_info) => user_info.user_name,
                _ => {
                    return ServerToClientMessage::QuitGroupChatResponse(
                        UserQuitGroupChatResponse::DatabaseError,
                    )
                }
            };
            send_admin_message_to_group_chat(chat_id, format!("{}退出群聊", user_name)).await;
            if let Ok(users) = get_chat_user_list(chat_id).await {
                match users {
                    ChatMembers::Group(group_users) => {
                        let handlers = UserRequestHandler::Group(group_users);
                        let msg = ServerToClientMessage::GroupMemberChange(MemberChangeData {
                            chat_id,
                            user_id,
                            r#type: MemberChangeType::DeleteMember,
                        });
                        send_msg_to_online_handlers(msg, handlers).await;
                    }
                    ChatMembers::Private(_) => {
                        return ServerToClientMessage::QuitGroupChatResponse(
                            UserQuitGroupChatResponse::DatabaseError,
                        );
                    }
                }
            } else {
                return ServerToClientMessage::QuitGroupChatResponse(
                    UserQuitGroupChatResponse::DatabaseError,
                );
            }
            return ServerToClientMessage::QuitGroupChatResponse(
                UserQuitGroupChatResponse::Success { chat_id },
            );
        }
        _ => return ServerToClientMessage::QuitGroupChatResponse(quit_result),
    }
}

pub async fn set_user_already_read(
    user_id: UserID,
    data: UserSetAlreadyReadData,
) -> ServerToClientMessage {
    if data.private {
        let (id1, id2) = match database::get_private_chat_user_list(data.chat_id).await {
            Ok(opt) => match opt {
                Some(pair) => pair,
                None => {
                    return ServerToClientMessage::SetAlreadyReadResponse(
                        SetAlreadyReadResponse::NotPrivate,
                    )
                }
            },
            Err(_) => {
                return ServerToClientMessage::SetAlreadyReadResponse(
                    SetAlreadyReadResponse::DatabaseError,
                )
            }
        };

        let opposite = if id1 == user_id {
            id2
        } else if id2 == user_id {
            id1
        } else {
            return ServerToClientMessage::SetAlreadyReadResponse(
                SetAlreadyReadResponse::NotInChat,
            );
        };

        user_sessions.do_send_message_to(
            opposite,
            UserSessionActorMessage::SendServerMessage(
                ServerToClientMessage::SetOppositeReadCursor(UserSetOppositeReadCursorData {
                    chat_id: data.chat_id,
                    in_chat_id: data.in_chat_id,
                }),
            ),
        );
    }

    return ServerToClientMessage::SetAlreadyReadResponse(
        database::set_user_already_read(user_id, data).await,
    );
}

pub async fn upload_file_req(
    user_id: UserID,
    req: UserUploadFileRequestData,
) -> ServerToClientMessage {
    if let Ok(url_opt) = oss::get_public_url_and_auto_renew(&req.user_hash).await {
        if let Some(url) = url_opt {
            return ServerToClientMessage::UploadFileRequestResponse(
                UserUploadFileRequestResponse {
                    user_hash: req.user_hash,
                    state: UserUploadFileRequestResponseState::Existed,
                    url: Some(url),
                    upload_id: None,
                },
            );
        }
    } else {
        return ServerToClientMessage::UploadFileRequestResponse(UserUploadFileRequestResponse {
            user_hash: req.user_hash,
            state: UserUploadFileRequestResponseState::OSSError,
            url: None,
            upload_id: None,
        });
    }

    let presign = match oss::get_presign_put_file_url(req.suffix) {
        Ok(presign) => presign,
        Err(_) => {
            return ServerToClientMessage::UploadFileRequestResponse(
                UserUploadFileRequestResponse {
                    user_hash: req.user_hash,
                    state: UserUploadFileRequestResponseState::OSSError,
                    url: None,
                    upload_id: None,
                },
            )
        }
    };

    // TODO : 去掉Clone
    let upload_req = ObjectUploadRequest {
        user_id,
        user_hash: req.user_hash.clone(),
        file_size: req.size,
        path: presign.path.clone(),
    };

    let upload_id = match database::write_upload_request(upload_req).await {
        Ok(id) => id,
        Err(_) => {
            return ServerToClientMessage::UploadFileRequestResponse(
                UserUploadFileRequestResponse {
                    user_hash: req.user_hash,
                    state: UserUploadFileRequestResponseState::DatabaseError,
                    url: None,
                    upload_id: None,
                },
            )
        }
    };

    return ServerToClientMessage::UploadFileRequestResponse(UserUploadFileRequestResponse {
        user_hash: req.user_hash,
        state: UserUploadFileRequestResponseState::Approve,
        url: Some(presign.url),
        upload_id: Some(upload_id),
    });
}

pub async fn file_uploaded(user_id: UserID, upload_id: UploadId) -> ServerToClientMessage {
    let req = match database::get_upload_request(upload_id).await {
        Ok(req) => match req {
            Some(req) => req,
            None => {
                return ServerToClientMessage::FileUploadedResponse(UserFileUploadedResponse {
                    upload_id,
                    state: UserFileUploadedState::RequestNotFound,
                    url: None,
                })
            }
        },
        Err(_) => {
            return ServerToClientMessage::FileUploadedResponse(UserFileUploadedResponse {
                upload_id,
                state: UserFileUploadedState::DatabaseError,
                url: None,
            })
        }
    };

    if req.user_id != user_id {
        return ServerToClientMessage::FileUploadedResponse(UserFileUploadedResponse {
            upload_id,
            state: UserFileUploadedState::NotUploader,
            url: None,
        });
    }

    let object_stat = match oss::get_object_stat(&req.path).await {
        Ok(stat) => stat,
        Err(_) => {
            return ServerToClientMessage::FileUploadedResponse(UserFileUploadedResponse {
                upload_id,
                state: UserFileUploadedState::ObjectNotFound,
                url: None,
            })
        }
    };

    let hash = object_stat.e_tag.unwrap();

    if &hash[1..hash.len() - 1] != &req.user_hash {
        return ServerToClientMessage::FileUploadedResponse(UserFileUploadedResponse {
            upload_id,
            state: UserFileUploadedState::FileHashError,
            url: None,
        });
    }

    if object_stat.content_length.unwrap() != req.file_size as i64 {
        return ServerToClientMessage::FileUploadedResponse(UserFileUploadedResponse {
            upload_id,
            state: UserFileUploadedState::FileSizeError,
            url: None,
        });
    }

    let url = match oss::create_pub_url(
        &req.user_hash,
        req.path,
        Config::get().s3_oss.presign_get_expire,
    )
    .await
    {
        Ok(presign_url) => presign_url.url,
        Err(_) => {
            return ServerToClientMessage::FileUploadedResponse(UserFileUploadedResponse {
                upload_id,
                state: UserFileUploadedState::OSSError,
                url: None,
            })
        }
    };

    return ServerToClientMessage::FileUploadedResponse(UserFileUploadedResponse {
        upload_id,
        state: UserFileUploadedState::Success,
        url: Some(url),
    });
}

pub async fn get_file_pub_url(hash: String) -> ServerToClientMessage {
    match oss::get_public_url_and_auto_renew(&hash).await {
        Ok(url_opt) => ServerToClientMessage::GetFileUrlResponse(UserGetFileUrlResponse {
            hash,
            state: if url_opt.is_some() {
                UserGetFilePubUrlState::Success
            } else {
                UserGetFilePubUrlState::FileNotExisted
            },
            url: url_opt,
        }),
        Err(_) => ServerToClientMessage::GetFileUrlResponse(UserGetFileUrlResponse {
            hash,
            state: UserGetFilePubUrlState::OSSError,
            url: None,
        }),
    }
}

pub async fn get_group_users(user_id: UserID, chat_id: ChatID) -> ServerToClientMessage {
    if let Ok(check) = database::check_user_in_chat(user_id, chat_id).await {
        if !check {
            return ServerToClientMessage::GetGroupUsersResponse(
                UserGetGroupUsersResponse::UserNotInChat,
            );
        }
    } else {
        return ServerToClientMessage::GetGroupUsersResponse(
            UserGetGroupUsersResponse::DatabaseError,
        );
    }
    match database::get_chat_user_list(chat_id).await {
        Ok(users) => {
            let user_ids = match users {
                ChatMembers::Group(user_ids) => user_ids,
                ChatMembers::Private(_) => {
                    return ServerToClientMessage::GetGroupUsersResponse(
                        UserGetGroupUsersResponse::ServerError,
                    )
                }
            };
            ServerToClientMessage::GetGroupUsersResponse(UserGetGroupUsersResponse::Success {
                chat_id,
                user_ids,
            })
        }
        Err(_) => {
            ServerToClientMessage::GetGroupUsersResponse(UserGetGroupUsersResponse::ServerError)
        }
    }
}

pub async fn set_as_admin(user_id: UserID, data: UserSetGroupAdminData) -> ServerToClientMessage {
    let check_owner = database::check_user_is_owner(user_id, data.chat_id).await;
    let check_in_chat = database::check_user_in_chat(data.user_id, data.chat_id).await;
    let check_admin = database::check_user_is_admin(data.user_id, data.chat_id).await;

    if check_owner.is_err() || check_in_chat.is_err() || check_in_chat.is_err() {
        return ServerToClientMessage::SetGroupAdminResponse(
            UserSetGroupAdminResponse::DatabaseError,
        );
    }

    if !check_owner.unwrap() {
        return ServerToClientMessage::SetGroupAdminResponse(UserSetGroupAdminResponse::NotOwner);
    }

    if !check_in_chat.unwrap() {
        return ServerToClientMessage::SetGroupAdminResponse(
            UserSetGroupAdminResponse::UserNotInChat,
        );
    }

    if check_admin.unwrap() {
        return ServerToClientMessage::SetGroupAdminResponse(
            UserSetGroupAdminResponse::AlreadyAdmin,
        );
    }

    return ServerToClientMessage::SetGroupAdminResponse(
        database::set_as_admin(data.user_id, data.chat_id).await,
    );
}

pub async fn revoke_message(user_id: UserID, data: UserRevokeMessageData) -> ServerToClientMessage {
    let chat_id = data.chat_id;
    let in_chat_id = data.in_chat_id;

    let chat_msg: ChatMessage =
        match database::get_messages_in_chat(chat_id, in_chat_id, Some(in_chat_id)).await {
            Ok(msgs) => {
                if msgs.len() != 1 {
                    return ServerToClientMessage::RevokeMessageResponse(
                        UserRevokeMessageResponse {
                            chat_id,
                            in_chat_id,
                            state: UserRevokeMessageResponseState::MessageNotExisted,
                        },
                    );
                }
                serde_json::from_str(&msgs[0]).unwrap()
            }
            Err(_) => {
                return ServerToClientMessage::RevokeMessageResponse(UserRevokeMessageResponse {
                    chat_id,
                    in_chat_id,
                    state: UserRevokeMessageResponseState::DatabaseError,
                })
            }
        };

    // 鉴权
    match data.method {
        UserRevokeMethod::Sender => {
            if chat_msg.sender_id != user_id {
                return ServerToClientMessage::RevokeMessageResponse(UserRevokeMessageResponse {
                    chat_id,
                    in_chat_id,
                    state: UserRevokeMessageResponseState::PermissionsDenied,
                });
            }
        }
        UserRevokeMethod::GroupAdmin => {
            let check_revoker = check_user_is_admin(user_id, chat_id).await;
            let check_sender = check_user_is_admin(chat_msg.sender_id, chat_id).await;
            if check_revoker.is_err() || check_sender.is_err() {
                return ServerToClientMessage::RevokeMessageResponse(UserRevokeMessageResponse {
                    chat_id,
                    in_chat_id,
                    state: UserRevokeMessageResponseState::DatabaseError,
                });
            }
            if !check_revoker.unwrap() || (user_id != chat_msg.sender_id && check_sender.unwrap()) {
                return ServerToClientMessage::RevokeMessageResponse(UserRevokeMessageResponse {
                    chat_id,
                    in_chat_id,
                    state: UserRevokeMessageResponseState::PermissionsDenied,
                });
            }
        }
        UserRevokeMethod::GroupOwner => {
            let check_owner = check_user_is_owner(user_id, chat_id).await;
            if let Ok(check) = check_owner {
                if !check {
                    return ServerToClientMessage::RevokeMessageResponse(
                        UserRevokeMessageResponse {
                            chat_id,
                            in_chat_id,
                            state: UserRevokeMessageResponseState::PermissionsDenied,
                        },
                    );
                }
            } else {
                return ServerToClientMessage::RevokeMessageResponse(UserRevokeMessageResponse {
                    chat_id,
                    in_chat_id,
                    state: UserRevokeMessageResponseState::DatabaseError,
                });
            }
        }
    }

    if let Err(_) =
        database::revoke_message(chat_id, in_chat_id, chat_msg.sender_id, chat_msg.timestamp).await
    {
        return ServerToClientMessage::RevokeMessageResponse(UserRevokeMessageResponse {
            chat_id,
            in_chat_id,
            state: UserRevokeMessageResponseState::DatabaseError,
        });
    }

    let timestamp = Utc::now().timestamp_millis() as Timestamp;
    let notice = UserNotice::Revoked {
        chat_id: chat_id,
        in_chat_id: in_chat_id,
        timestamp,
    };
    let serialized_notice = serde_json::to_string(&notice).unwrap();

    if let Err(_) =
        user_notice::send_notice_to_user_in_chat(chat_id, serialized_notice, timestamp).await
    {
        return ServerToClientMessage::RevokeMessageResponse(UserRevokeMessageResponse {
            chat_id,
            in_chat_id,
            state: UserRevokeMessageResponseState::DatabaseError,
        });
    }

    return ServerToClientMessage::RevokeMessageResponse(UserRevokeMessageResponse {
        chat_id,
        in_chat_id,
        state: UserRevokeMessageResponseState::Success,
    });
}

pub async fn owner_transfer(
    user_id: UserID,
    data: UserGroupOwnerTransferData,
) -> ServerToClientMessage {
    let check_owner = database::check_user_is_owner(user_id, data.chat_id).await;
    let check_in_chat = database::check_user_in_chat(user_id, data.chat_id).await;

    if check_owner.is_err() || check_in_chat.is_err() {
        return ServerToClientMessage::GroupOwnerTransferResponse(
            UserGroupOwnerTransferResponse::DatabaseError,
        );
    }

    if !check_owner.unwrap() {
        return ServerToClientMessage::GroupOwnerTransferResponse(
            UserGroupOwnerTransferResponse::NotOwner,
        );
    }

    if !check_in_chat.unwrap() {
        return ServerToClientMessage::GroupOwnerTransferResponse(
            UserGroupOwnerTransferResponse::UserNotInChat,
        );
    }

    return ServerToClientMessage::GroupOwnerTransferResponse(
        database::owner_transfer(data.user_id, data.chat_id).await,
    );
}

pub async fn group_notice(user_id: UserID, data: UserSendGroupNoticeData) -> ServerToClientMessage {
    if let Ok(check) = database::check_user_is_admin(user_id, data.chat_id).await {
        if !check {
            return ServerToClientMessage::GroupNoticeResponse(
                UserSendGroupNoticeResponse::NoPermission,
            );
        }
    } else {
        return ServerToClientMessage::GroupNoticeResponse(
            UserSendGroupNoticeResponse::DatabaseError,
        );
    }

    if data.notice.len() > Config::get().safety.max_notice_length as usize {
        return ServerToClientMessage::GroupNoticeResponse(
            UserSendGroupNoticeResponse::LenthLimitExceeded {
                chat_id: data.chat_id,
                client_id: data.client_id,
            },
        );
    }

    return ServerToClientMessage::GroupNoticeResponse(
        database::add_group_notice(user_id, data.chat_id, data.client_id, data.notice).await,
    );
}

pub async fn pull_group_notice(
    user_id: UserID,
    data: UserPullGroupNoticeData,
) -> ServerToClientMessage {
    if let Ok(check) = database::check_user_in_chat(user_id, data.chat_id).await {
        if !check {
            return ServerToClientMessage::PullGroupNoticeResponse(
                UserPullGroupNoticeResponse::UserNotInChat,
            );
        }
    } else {
        return ServerToClientMessage::PullGroupNoticeResponse(
            UserPullGroupNoticeResponse::DatabaseError,
        );
    }

    return ServerToClientMessage::PullGroupNoticeResponse(
        database::pull_group_notice(data.chat_id, data.last_notice_id).await,
    );
}

pub async fn update_group_info(
    user_id: UserID,
    data: UserUpdateGroupData,
) -> ServerToClientMessage {
    match &data.content {
        UserUpdateGroupContent::GroupName { new_name } => {
            if new_name.len() > Config::get().user.max_user_name_length as usize {
                return ServerToClientMessage::UpdateGroupInfoResponse(
                    UserUpdateGroupInfoResponse::GroupNameFormatError,
                );
            }
        }
        UserUpdateGroupContent::Avater { new_avater: _ } => {
            // check_avater
        }
    }
    if let Ok(check) = database::check_user_is_admin(user_id, data.chat_id).await {
        if !check {
            return ServerToClientMessage::UpdateGroupInfoResponse(
                UserUpdateGroupInfoResponse::NoPermission,
            );
        }
    } else {
        return ServerToClientMessage::UpdateGroupInfoResponse(
            UserUpdateGroupInfoResponse::DatabaseError,
        );
    }
    return ServerToClientMessage::UpdateGroupInfoResponse(
        database::update_group_info(data.chat_id, data.content).await,
    );
}

pub async fn remove_member(
    user_id: UserID,
    data: UserRemoveGroupMemberData,
) -> ServerToClientMessage {
    if user_id == data.user_id {
        return ServerToClientMessage::RemoveGroupMemberResponse(
            UserRemoveGroupMemberResponse::SameUser,
        );
    }

    let check_in_chat = check_user_in_chat(user_id, data.chat_id).await;
    let check_owner = database::check_user_is_owner(user_id, data.chat_id).await;
    let check_admin = database::check_user_is_admin(user_id, data.chat_id).await;
    let check_removed = database::check_user_is_admin(data.user_id, data.chat_id).await;

    if check_owner.is_err()
        || check_admin.is_err()
        || check_removed.is_err()
        || check_in_chat.is_err()
    {
        return ServerToClientMessage::RemoveGroupMemberResponse(
            UserRemoveGroupMemberResponse::DatabaseError,
        );
    }
    if !check_in_chat.unwrap() {
        return ServerToClientMessage::RemoveGroupMemberResponse(
            UserRemoveGroupMemberResponse::UserNotInChat,
        );
    }
    if check_owner.unwrap() || (check_admin.unwrap() && !check_removed.unwrap()) {
        let quit_result = database::quit_group_chat(data.user_id, data.chat_id).await;
        match quit_result {
            UserQuitGroupChatResponse::Success { chat_id } => {
                let user_name = match database::get_user_info(data.user_id).await {
                    GetUserInfoResponse::Success(user_info) => user_info.user_name,
                    _ => {
                        return ServerToClientMessage::RemoveGroupMemberResponse(
                            UserRemoveGroupMemberResponse::DatabaseError,
                        )
                    }
                };
                let admin_name = match database::get_user_info(user_id).await {
                    GetUserInfoResponse::Success(user_info) => user_info.user_name,
                    _ => {
                        return ServerToClientMessage::RemoveGroupMemberResponse(
                            UserRemoveGroupMemberResponse::DatabaseError,
                        )
                    }
                };
                send_delete_chat_msg(data.user_id, chat_id).await;
                send_admin_message_to_group_chat(
                    chat_id,
                    format!("{}被{}移出群聊", user_name, admin_name),
                )
                .await;
                if let Ok(users) = get_chat_user_list(chat_id).await {
                    match users {
                        ChatMembers::Group(group_users) => {
                            let handlers = UserRequestHandler::Group(group_users);
                            let msg = ServerToClientMessage::GroupMemberChange(MemberChangeData {
                                chat_id,
                                user_id,
                                r#type: MemberChangeType::DeleteMember,
                            });
                            send_msg_to_online_handlers(msg, handlers).await;
                        }
                        ChatMembers::Private(_) => {
                            return ServerToClientMessage::RemoveGroupMemberResponse(
                                UserRemoveGroupMemberResponse::DatabaseError,
                            );
                        }
                    }
                } else {
                    return ServerToClientMessage::RemoveGroupMemberResponse(
                        UserRemoveGroupMemberResponse::DatabaseError,
                    );
                }
                return ServerToClientMessage::RemoveGroupMemberResponse(
                    UserRemoveGroupMemberResponse::Success {
                        chat_id: data.chat_id,
                        user_id: data.user_id,
                    },
                );
            }
            _ => ServerToClientMessage::RemoveGroupMemberResponse(
                UserRemoveGroupMemberResponse::DatabaseError,
            ),
        }
    } else {
        ServerToClientMessage::RemoveGroupMemberResponse(
            UserRemoveGroupMemberResponse::NoPermission,
        )
    }
}

pub async fn unset_group_admin(
    user_id: UserID,
    data: UserUnsetGroupAdminData,
) -> ServerToClientMessage {
    if user_id == data.user_id {
        return ServerToClientMessage::UnsetGroupAdminResponse(
            UserUnsetGroupAdminResponse::SameUser,
        );
    }
    let check_owner = check_user_is_owner(user_id, data.chat_id).await;
    let check_admin = check_user_is_admin(data.user_id, data.chat_id).await;

    if check_owner.is_err() || check_admin.is_err() {
        return ServerToClientMessage::UnsetGroupAdminResponse(
            UserUnsetGroupAdminResponse::DatabaseError,
        );
    }

    if !check_owner.unwrap() {
        return ServerToClientMessage::UnsetGroupAdminResponse(
            UserUnsetGroupAdminResponse::NotOwner,
        );
    }
    if !check_admin.unwrap() {
        return ServerToClientMessage::UnsetGroupAdminResponse(
            UserUnsetGroupAdminResponse::NotAdmin,
        );
    }

    return ServerToClientMessage::UnsetGroupAdminResponse(
        database::unset_admin(data.user_id, data.chat_id).await,
    );
}

pub async fn get_group_owner(_user_id: UserID, chat_id: ChatID) -> ServerToClientMessage {
    match database::get_chat_owner(chat_id).await {
        Ok(owner) => {
            return ServerToClientMessage::GetGroupOwnerResponse(
                UserGetGroupOwnerResponse::Success {
                    chat_id,
                    user_id: owner,
                },
            )
        }
        Err(_) => {
            ServerToClientMessage::GetGroupOwnerResponse(UserGetGroupOwnerResponse::ServerError)
        }
    }
}

pub async fn get_group_admin(_user_id: UserID, chat_id: ChatID) -> ServerToClientMessage {
    match database::get_chat_admins_list(chat_id).await {
        Ok(users) => {
            let user_ids = match users {
                UserRequestHandler::Group(user_ids) => user_ids,
                UserRequestHandler::One(_) => {
                    return ServerToClientMessage::GetGroupAdminResponse(
                        UserGetGroupAdminResponse::DatabaseError,
                    )
                }
            };
            ServerToClientMessage::GetGroupAdminResponse(UserGetGroupAdminResponse::Success {
                chat_id,
                user_ids,
            })
        }
        Err(_) => {
            ServerToClientMessage::GetGroupAdminResponse(UserGetGroupAdminResponse::DatabaseError)
        }
    }
}

pub async fn media_call(user_id: UserID, data: UserMediaCallData) -> ServerToClientMessage {
    match database::get_chat_id_by_friends(user_id, data.friend_id).await {
        Ok(opt) => {
            if opt.is_none() {
                return ServerToClientMessage::MediaCallResponse(UserMediaCallResponse::NotFriend);
            }
        }
        Err(_) => {
            return ServerToClientMessage::MediaCallResponse(UserMediaCallResponse::DatabaseError)
        }
    }

    user_sessions.do_send_message_to(
        data.friend_id,
        UserSessionActorMessage::SendServerMessage(ServerToClientMessage::MediaCallOffer(
            UserMediaCallData {
                friend_id: user_id,
                call_type: data.call_type,
                serialized_offer: data.serialized_offer,
            },
        )),
    );

    return ServerToClientMessage::MediaCallResponse(UserMediaCallResponse::Success);
}

pub async fn media_call_answer(user_id: UserID, data: UserMediaCallAnswer) {
    // 后端本身不保存状态，由客户端判定消息是否合法
    user_sessions.do_send_message_to(
        data.friend_id,
        UserSessionActorMessage::SendServerMessage(ServerToClientMessage::MediaCallAnswer(
            UserMediaCallAnswer {
                friend_id: user_id,
                accept: data.accept,
                serialized_answer: data.serialized_answer,
            },
        )),
    );
}

pub async fn media_ice_candidate(user_id: UserID, data: UserMediaIceCandidate) {
    user_sessions.do_send_message_to(
        data.friend_id,
        UserSessionActorMessage::SendServerMessage(ServerToClientMessage::MediaIceCandidate(
            UserMediaIceCandidate {
                friend_id: user_id,
                serialized_candidate: data.serialized_candidate,
            },
        )),
    );
}

pub async fn media_call_stop(user_id: UserID, data: UserMediaCallStop) {
    user_sessions.do_send_message_to(
        data.friend_id,
        UserSessionActorMessage::SendServerMessage(ServerToClientMessage::MediaCallStop(
            UserMediaCallStop {
                friend_id: user_id,
                reason: data.reason,
            },
        )),
    );
}

pub async fn get_user_id(name: String) -> ServerToClientMessage {
    return ServerToClientMessage::GetUserIDResponse(database::get_user_id(name).await);
}

pub async fn get_user_read_in_group(
    user_id: UserID,
    data: UserGetUserReadInGroupData,
    receiver: Recipient<UserSessionActorMessage>,
) {
    if let Ok(check) = database::check_is_group(data.chat_id).await {
        if !check {
            receiver.do_send(UserSessionActorMessage::SendServerMessage(
                ServerToClientMessage::GetUserReadInGroupResponse(
                    UserGetUserReadInGroupResponse::NotGroupChat,
                ),
            ));
        }
    } else {
        receiver.do_send(UserSessionActorMessage::SendServerMessage(
            ServerToClientMessage::GetUserReadInGroupResponse(
                UserGetUserReadInGroupResponse::DatabaseError,
            ),
        ));
    }

    if let Ok(check) = database::check_user_in_chat(user_id, data.chat_id).await {
        if !check {
            receiver.do_send(UserSessionActorMessage::SendServerMessage(
                ServerToClientMessage::GetUserReadInGroupResponse(
                    UserGetUserReadInGroupResponse::UserNotInChat,
                ),
            ));
        }
    } else {
        receiver.do_send(UserSessionActorMessage::SendServerMessage(
            ServerToClientMessage::GetUserReadInGroupResponse(
                UserGetUserReadInGroupResponse::DatabaseError,
            ),
        ));
    }
    receiver.do_send(UserSessionActorMessage::SendServerMessage(
        ServerToClientMessage::GetUserReadInGroupResponse(
            database::get_user_read_in_group(data.chat_id, data.in_chat_id).await,
        ),
    ));
}

pub async fn get_user_read_in_private(user_id: UserID, chat_id: ChatID) -> ServerToClientMessage {
    if let Ok(check) = database::check_is_group(chat_id).await {
        if check {
            return ServerToClientMessage::GetUserReadInPrivateResponse(
                UserGetUserReadInPrivateResponse::NotPrivateChat,
            );
        }
    } else {
        return ServerToClientMessage::GetUserReadInPrivateResponse(
            UserGetUserReadInPrivateResponse::DatabaseError,
        );
    }

    if let Ok(check) = database::check_user_in_chat(user_id, chat_id).await {
        if !check {
            return ServerToClientMessage::GetUserReadInPrivateResponse(
                UserGetUserReadInPrivateResponse::UserNotInChat,
            );
        }
    } else {
        return ServerToClientMessage::GetUserReadInPrivateResponse(
            UserGetUserReadInPrivateResponse::DatabaseError,
        );
    }

    return ServerToClientMessage::GetUserReadInPrivateResponse(
        database::get_user_read_in_private(user_id, chat_id).await,
    );
}

pub async fn user_log_off(
    user_id: UserID,
    email_code: EmailCodeValue,
    self_recipient: Recipient<UserSessionActorMessage>,
) -> ServerToClientMessage {
    if let Ok(email) = database::get_user_email(user_id).await {
        if !check_and_consume_email_code(&email, email_code) {
            return ServerToClientMessage::LogOffResponse(UserLogOffResponse::EmailCodeError);
        }
        let result = database::user_log_off(user_id).await;
        match result.0 {
            UserLogOffResponse::Success {} => {
                for friend_chat in result.1 {
                    send_delete_chat_msg(friend_chat.0, friend_chat.1).await;
                }
                self_recipient.do_send(UserSessionActorMessage::SendServerMessage(
                    ServerToClientMessage::LogOffResponse(UserLogOffResponse::Success),
                ));
                self_recipient.do_send(UserSessionActorMessage::SendServerMessage(
                    ServerToClientMessage::Close,
                ));
            }
            _ => {}
        }
        return ServerToClientMessage::LogOffResponse(result.0);
    } else {
        return ServerToClientMessage::LogOffResponse(UserLogOffResponse::DatabaseError);
    }
}

pub async fn user_apply_for_token(user_id: UserID) -> ServerToClientMessage {
    return ServerToClientMessage::ApplyForTokenResponse(database::apply_for_token(user_id).await);
}
