use std::{
    cell::{Ref, RefCell, RefMut},
    net::SocketAddr,
    rc::Rc,
    sync::Arc,
};

use actix::{Message, Recipient};

use log::info;
use rsa::RsaPublicKey;

use crate::{
    config::datatype::{SymCipher, UserID},
    server::server_state::user_sessions,
};

use super::protocol::ServerToClientMessage;

/** `UserSessionState` 用户连接的状态
*/
#[derive(PartialEq)]
pub enum UserSessionState {
    Started,
    Approved,
    Logged,
}

/** `UserSessionData` 用户连接数据类型
*/
pub struct UserSessionData {
    pub state: UserSessionState,
    pub client_ip: SocketAddr,
    pub user_id: Option<UserID>,
    pub user_pub_key: Option<RsaPublicKey>,
}

impl UserSessionData {
    pub fn on_login_success(
        &mut self,
        user_id: UserID,
        self_recipient: Recipient<UserSessionActorMessage>,
    ) {
        self.state = UserSessionState::Logged;
        self.user_id = Some(user_id);
        user_sessions.insert(user_id, self_recipient);
        info!("{} 登录成功", self.get_info());
    }
}

pub type UserSessionDataRc = Rc<RefCell<UserSessionData>>;

impl UserSessionData {
    pub fn get_info(&self) -> String {
        return format!(
            "UserSession (ip : {}, id : {})",
            self.client_ip.to_string(),
            self.user_id.map_or(0, |id| id)
        );
    }
}

/** `UserSession` 定义一个用户连接应该具有的特征
*/
pub trait UserSession {
    fn get_mut_data(&mut self) -> RefMut<'_, UserSessionData>;

    fn get_data(&self) -> Ref<'_, UserSessionData>;

    fn get_data_rc(&self) -> UserSessionDataRc;

    fn get_cipher(&self) -> Option<&SymCipher>;

    fn set_cipher(&mut self, cipher: Option<SymCipher>);

    fn on_sym_key_set_success(&mut self, cipher: SymCipher) {
        self.set_cipher(Some(cipher));
        let mut data = self.get_mut_data();
        data.state = UserSessionState::Approved;
        info!("{} 加密协议握手成功", data.get_info());
    }
}

/** `UserSessionActorMessage` 用户连接从服务端其他Actor收到的消息
*/
#[derive(Message)]
#[rtype(result = "UserSessionActorResponse")]
pub enum UserSessionActorMessage {
    SendServerMessage(ServerToClientMessage),
    SendServerMessageArc(Arc<ServerToClientMessage>),
    SendStringSerializedServerMessage(Arc<String>),
}

/** `UserSessionActorResponse` 用户连接收到从服务端其他Actor消息时的响应
*/
pub enum UserSessionActorResponse {
    Ok,
}
