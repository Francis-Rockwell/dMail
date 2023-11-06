use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use crate::{
    config::{datatype::SymCipher, Config},
    server::server_state::user_sessions,
};

use super::client_message_handler::ClientMessageHandler;
use super::{
    protocol::ClientToServerMessage, user_session::UserSessionActorMessage, UserSession,
    UserSessionData,
};
use crate::user::user_session::actor_message_handler::ActorMessageHandler;

use actix::prelude::*;
use actix_web_actors::ws::{self, WebsocketContext};
use log::debug;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

use crate::utils::AesGcmHelper;

/** `WsUserSession` Websocket用户连接数据类型
*/
pub struct WsUserSession {
    pub data: Rc<RefCell<UserSessionData>>,
    pub cipher: Option<SymCipher>,
    pub last_receive_time: Instant,
}

/** `WsProtocolError` Websocket连接中出现的错误类型
*/
#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "command", content = "data")]
pub enum WsProtocolError {
    TextFormatError,
    TextDecodeFailed,
    DataParseFailed(String),
    UnsupportedWsMessage,
}

const HEART_BEAT_INTERVAL: Lazy<Duration> =
    Lazy::new(|| Duration::from_secs(Config::get().user.heart_beat_time.into()));

impl Actor for WsUserSession {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        debug!("WebSocket {} 连接", self.get_data().get_info());

        ctx.run_interval(*HEART_BEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.last_receive_time) > *HEART_BEAT_INTERVAL {
                ctx.stop();
                return;
            }
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        debug!("WebSocket {} 断开", self.get_data().get_info());

        if let Some(id) = self.get_mut_data().user_id {
            user_sessions.remove(&id);
        }

        Running::Stop
    }
}

impl UserSession for WsUserSession {
    fn get_data(&self) -> Ref<'_, UserSessionData> {
        return self.data.borrow();
    }

    fn get_mut_data(&mut self) -> RefMut<'_, UserSessionData> {
        return self.data.borrow_mut();
    }

    fn get_data_rc(&self) -> Rc<RefCell<UserSessionData>> {
        return self.data.clone();
    }

    fn get_cipher(&self) -> Option<&crate::config::datatype::SymCipher> {
        return self.cipher.as_ref();
    }

    fn set_cipher(&mut self, cipher: Option<SymCipher>) {
        self.cipher = cipher;
    }
}

impl ActorMessageHandler for WsUserSession {}
impl ClientMessageHandler for WsUserSession {}

impl WsUserSession {
    pub fn ws_message_to_client_message(
        &self,
        ws_msg: ws::Message,
    ) -> Result<ClientToServerMessage, WsProtocolError> {
        match ws_msg {
            ws::Message::Text(bytes) => {
                let text_str = if let Ok(str) = self.decode_string(&bytes) {
                    str
                } else {
                    return Err(WsProtocolError::TextDecodeFailed);
                };

                match serde_json::from_str::<ClientToServerMessage>(&text_str) {
                    Ok(msg) => Ok(msg),
                    Err(err) => Err(WsProtocolError::DataParseFailed(err.to_string())),
                }
            }
            ws::Message::Ping(_) => Ok(ClientToServerMessage::Ping),
            ws::Message::Pong(_) => Ok(ClientToServerMessage::Pong),
            ws::Message::Close(_) => Ok(ClientToServerMessage::Close),
            _ => Err(WsProtocolError::UnsupportedWsMessage),
        }
    }

    fn decode_string(&self, data: &str) -> Result<String, ()> {
        if let Some(cipher) = self.get_cipher() {
            return cipher.decrypt_with_default_nouce_from_base64(data);
        } else {
            return Ok(data.to_string());
        }
    }
}

impl Handler<UserSessionActorMessage> for WsUserSession {
    type Result = MessageResult<UserSessionActorMessage>;

    fn handle(&mut self, msg: UserSessionActorMessage, ctx: &mut Self::Context) -> Self::Result {
        return self.handle_actor_message(&msg, ctx);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsUserSession {
    fn handle(&mut self, item: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        self.last_receive_time = Instant::now();

        let ws_msg = match item {
            Ok(msg) => msg,
            Err(err) => {
                debug!(
                    "{} 发生ws::ProtocolError，关闭连接：{}",
                    self.get_data().get_info(),
                    err
                );
                ctx.stop();
                return;
            }
        };

        let client_message = match self.ws_message_to_client_message(ws_msg) {
            Ok(msg) => msg,
            Err(err) => {
                debug!(
                    "{} 发生WsProtocolError {:?}",
                    self.get_data().get_info(),
                    err
                );
                ctx.stop();
                return;
            }
        };

        if client_message != ClientToServerMessage::Ping {
            debug!(
                "收到来自 {} 的请求：{}",
                self.get_data().get_info(),
                serde_json::to_string(&client_message).unwrap()
            );
        }

        self.handle_client_message(client_message, ctx);
    }
}
