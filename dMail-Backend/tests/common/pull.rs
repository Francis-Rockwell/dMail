use super::errors;
use super::testcase::TestCase;
use crate::common::testcase::WsContent;
use errors as ERRORS;
use serde_json;
use std::panic;
//use std::process::Command;
use tungstenite::{Message, WebSocket};

impl TestCase {
    pub fn pull<Stream>(&mut self, c: &WsContent, socket: &mut WebSocket<Stream>) -> Vec<String>
    where
        Stream: std::io::Read + std::io::Write,
    {
        // 存数据
        // Command::new("redis-cli")
        //     .arg("FLUSHALL")
        //     .output()
        //     .expect("failed to clear database");
        let after_write = socket.write_message(Message::Text(
            self.encode(serde_json::to_string(&c.request).expect(ERRORS::JSON_TO_STRING_FAILED)),
        ));
        if after_write.is_err() {
            self.kill_server();
            panic!("{}", ERRORS::SOCKET_WRITE_MESSAGE_FAILED);
        }

        let mut responses = vec![];
        for _ in 0..4 {
            let after_read = socket.read_message();
            if after_read.is_err() {
                self.kill_server();
                panic!("{}", ERRORS::SOCKET_READ_MESSAGE_FAILED);
            }
            responses.push(self.decode(after_read.unwrap().to_string()));
        }

        return responses;
    }
}
