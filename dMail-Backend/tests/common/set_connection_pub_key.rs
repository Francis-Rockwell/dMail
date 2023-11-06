use super::testcase::TestCase;
use crate::common::errors;
use crate::common::testcase::WsContent;
use aes_gcm::{Aes128Gcm, KeyInit};
use dMail::utils::AesGcmHelper;
use dMail::{
    user::user_session::protocol::ServerToClientMessage,
    utils::{base64::decode, rsa::get_private_key_from_base64_pkcs1_pem},
};
use errors as ERRORS;
use rsa::Pkcs1v15Encrypt;
use tungstenite::{Message, WebSocket};

impl TestCase {
    pub fn set_connection_pub_key<Stream>(
        &mut self,
        c: &WsContent,
        socket: &mut WebSocket<Stream>,
    ) -> String
    where
        Stream: std::io::Read + std::io::Write,
    {
        let after_write = socket.write_message(Message::Text(
            serde_json::to_string(&c.request).expect(ERRORS::JSON_TO_STRING_FAILED),
        ));
        if after_write.is_err() {
            self.kill_server();
            panic!("{}", ERRORS::SOCKET_WRITE_MESSAGE_FAILED);
        }

        let after_read = socket.read_message();
        if after_read.is_err() {
            self.kill_server();
            panic!("{}", ERRORS::SOCKET_READ_MESSAGE_FAILED);
        }
        let resp = after_read.unwrap().to_string();

        if let ServerToClientMessage::SetConnectionSymKey(private_key_str) = &c.response[0] {
            let private_key = get_private_key_from_base64_pkcs1_pem(private_key_str.to_string())
                .expect(ERRORS::DECODE_FAILED);

            let encoded_sym_key;
            if let Ok(ServerToClientMessage::SetConnectionSymKey(key)) =
                serde_json::from_str::<ServerToClientMessage>(&resp)
            {
                let decoded_key = decode(&key);
                if decoded_key.is_err() {
                    self.kill_server();
                    panic!("{}", ERRORS::DECODE_FAILED);
                }
                encoded_sym_key = decoded_key.unwrap();
            } else {
                return resp;
            };

            let decoded_sym_key = private_key
                .decrypt(Pkcs1v15Encrypt, &encoded_sym_key)
                .expect(ERRORS::DECODE_FAILED);

            let after_decode = Aes128Gcm::new_from_slice(
                &decode((std::str::from_utf8(&decoded_sym_key)).expect(ERRORS::TYPE_PARSE_FAILED))
                    .expect(ERRORS::DECODE_FAILED),
            );
            if after_decode.is_err() {
                self.kill_server();
                panic!("{}", ERRORS::DECODE_FAILED);
            }
            let sym_key = after_decode.unwrap();

            *(&mut self.sym_key) = Some(sym_key.clone());

            let test_request = r#"{
                "command": "Ping"
            }"#
            .to_string();

            let encoded_test_request = sym_key
                .encrypt_with_default_nouce_to_base64(&test_request)
                .expect(ERRORS::ENCODE_FAILED);

            let after_write = socket.write_message(Message::Text(encoded_test_request));
            if after_write.is_err() {
                self.kill_server();
                panic!("{}", ERRORS::SOCKET_WRITE_MESSAGE_FAILED);
            }

            let after_read = socket.read_message();
            if after_read.is_err() {
                self.kill_server();
                panic!("{}", ERRORS::SOCKET_READ_MESSAGE_FAILED);
            }

            after_read.unwrap().to_string();

            return serde_json::to_string(&c.response[0]).expect(ERRORS::JSON_TO_STRING_FAILED);
        } else {
            return serde_json::to_string(&resp).expect(ERRORS::JSON_TO_STRING_FAILED);
        }
    }
}
