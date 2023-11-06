/*!
服务器状态有关的函数
*/

use std::sync::Arc;

use actix::Recipient;
use chashmap::CHashMap;

use log::info;
use once_cell::sync::Lazy;

use super::email::{self, test_sender, EmailCode, EmailSender};
use crate::config::datatype::UserID;
use crate::config::Config;
use crate::database;
use crate::user::user_session::protocol::ServerToClientMessage;
use crate::user::user_session::UserSessionActorMessage;

use tokio::runtime::Handle as TokioHandle;
/** `UserSessionPool` 用户ID到用户连接的哈希表
*/
pub type UserSessionsPool = CHashMap<UserID, Recipient<UserSessionActorMessage>>;
/** `UserEmailCodeMap` 邮箱到验证码的哈希表
*/
pub type UserEmailCodeMap = CHashMap<String, EmailCode>;
/** `UserSymKeyMap` 用户ID到对称秘钥的哈希表
*/
pub type UserSymKeyMap = CHashMap<UserID, String>;

/** `user_sessions` 用户连接池
*/
#[allow(non_upper_case_globals)]
pub static user_sessions: Lazy<UserSessionsPool> = Lazy::new(|| CHashMap::new());

// #[allow(non_upper_case_globals)]
// pub static users_sym_token: Lazy<UserSymKeyMap> = Lazy::new(|| CHashMap::new());

/** `users_email_codes` 存储从邮箱到验证码的哈希表
*/
// TODO : 定时遍历，清除长期不用的CODE
#[allow(non_upper_case_globals)]
pub static users_email_codes: Lazy<UserEmailCodeMap> = Lazy::new(|| CHashMap::new());

/** `workers_handle` server_worker的tokio句柄
*/
#[allow(non_upper_case_globals)]
pub static workers_handle: Lazy<TokioHandle> = Lazy::new(|| {
    let num = Config::get().server_worker_num;
    info!("正在启动{}个server_worker", num);
    let local_runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num)
        .enable_io()
        .build()
        .unwrap();

    let runtime_box = Box::leak(Box::new(local_runtime));

    runtime_box.handle().clone()
});

/** `email_sender` 负责发送邮件验证码
*/
#[allow(non_upper_case_globals)]
pub static email_sender: Lazy<EmailSender> = Lazy::new(|| email::start_sender());

pub struct ServerState;

impl ServerState {
    pub async fn start() {
        Lazy::force(&workers_handle);
        if Config::get().email.enable {
            Lazy::force(&email_sender);
            test_sender().await;
        }
        database::connect_database().await;
    }
}

/** `UserSessionGetter` 可以获取用户连接的特征
*/
pub trait UserSessionGetter {
    fn get_user_session(&self, user_id: UserID) -> Option<Recipient<UserSessionActorMessage>>;

    fn do_send_message_to(&self, user_id: UserID, actor_msg: UserSessionActorMessage);

    fn send_message_to_online_with_exclusion(
        &self,
        ids: Vec<UserID>,
        arc: Arc<ServerToClientMessage>,
        exclude_id: UserID,
    ) {
        for id in ids {
            if id == exclude_id {
                continue;
            }
            self.do_send_message_to(
                id,
                UserSessionActorMessage::SendServerMessageArc(arc.clone()),
            );
        }
    }

    fn send_message_to_online(&self, ids: &Vec<UserID>, arc: Arc<ServerToClientMessage>) {
        for id in ids {
            self.do_send_message_to(
                *id,
                UserSessionActorMessage::SendServerMessageArc(arc.clone()),
            );
        }
    }
}

impl UserSessionGetter for UserSessionsPool {
    fn get_user_session(&self, user_id: UserID) -> Option<Recipient<UserSessionActorMessage>> {
        return self.get(&user_id).map(|guard| guard.clone());
    }

    fn do_send_message_to(&self, user_id: UserID, actor_msg: UserSessionActorMessage) {
        if let Some(session) = self.get(&user_id) {
            session.do_send(actor_msg);
        }
    }
}
