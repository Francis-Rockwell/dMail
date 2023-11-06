/*!
actor的消息处理函数
*/

use actix::{dev::ToEnvelope, Actor, Handler, MessageResult};

use super::{
    protocol::ServerMessageSender, UserSession, UserSessionActorMessage, UserSessionActorResponse,
};

pub trait ActorMessageHandler
where
    Self: Actor
        + UserSession
        + Handler<UserSessionActorMessage, Result = MessageResult<UserSessionActorMessage>>,
    Self::Context: ServerMessageSender<Self> + ToEnvelope<Self, UserSessionActorMessage>,
{
    fn handle_actor_message(
        &mut self,
        msg: &UserSessionActorMessage,
        ctx: &mut Self::Context,
    ) -> MessageResult<UserSessionActorMessage> {
        let cipher = self.get_cipher();
        let res = match msg {
            UserSessionActorMessage::SendServerMessage(msg) => {
                ctx.send_server_message(&msg, cipher);
                UserSessionActorResponse::Ok
            }
            UserSessionActorMessage::SendStringSerializedServerMessage(_arc_msg) => {
                todo!()
            }
            UserSessionActorMessage::SendServerMessageArc(arc_msg) => {
                ctx.send_server_message(&arc_msg, cipher);
                UserSessionActorResponse::Ok
            }
        };

        return MessageResult(res);
    }
}
