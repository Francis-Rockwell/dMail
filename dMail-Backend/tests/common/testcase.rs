use super::errors;
use aes_gcm::{aes::Aes128, AesGcm};
use dMail::user::user_session::protocol::{ClientToServerMessage, ServerToClientMessage};
use errors as ERRORS;
use generic_array::typenum::{
    bit::{B0, B1},
    UInt, UTerm,
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::File;
use std::panic;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::time::Duration;
use tungstenite::{connect, Message, WebSocket};
use url::Url;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WsContent {
    pub request: ClientToServerMessage,
    pub response: Vec<ServerToClientMessage>,
}

pub struct TestCase {
    pub name: String,
    pub data: Vec<WsContent>, // a sequence of Websocket requests and responses
    pub sym_key: Option<AesGcm<Aes128, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>>,
    pub prefix: String, // the prefix of the path of the Websocket requests
    pub running_process: Option<Child>,
    pub stdout_file: PathBuf,
    pub stderr_file: PathBuf,
}

impl TestCase {
    pub fn read(name: &str) -> Self {
        let case_dir = Path::new("tests").join("cases");
        let data_file = case_dir.join(format!("{}.data.json", &name));
        let stdout_file = case_dir.join(format!("{}.stdout", &name));
        let stderr_file = case_dir.join(format!("{}.stderr", &name));
        Self {
            name: name.to_string(),
            data: serde_json::from_reader(File::open(data_file).expect(ERRORS::FILE_ERR))
                .expect(ERRORS::JSON_FROM_FILE_FAILED),
            sym_key: None,
            prefix: "ws://127.0.0.1:8080/ws".to_string(),
            running_process: None,
            stdout_file,
            stderr_file,
        }
    }

    pub fn common_communicate<Stream>(
        &mut self,
        c: &WsContent,
        socket: &mut WebSocket<Stream>,
    ) -> String
    where
        Stream: std::io::Read + std::io::Write,
    {
        let after_write = socket.write_message(Message::Text(
            self.encode(serde_json::to_string(&c.request).expect(ERRORS::JSON_TO_STRING_FAILED)),
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
        self.decode(after_read.unwrap().to_string())
    }

    pub fn process<Stream>(&mut self, c: &WsContent, socket: &mut WebSocket<Stream>)
    where
        Stream: std::io::Read + std::io::Write,
    {
        match &c.request {
            ClientToServerMessage::SetConnectionPubKey(_) => {
                let expected = self.set_connection_pub_key(c, socket);
                self.single_check(expected, &c.response[0]);
            }
            ClientToServerMessage::Pull(_) => {
                let expecteds = self.pull(c, socket);
                self.multiple_check(expecteds, &c.response);
            }
            _ => {
                let expected = self.common_communicate(c, socket);
                self.single_check(expected, &c.response[0]);
            }
        };
    }

    pub fn run(&mut self) -> String {
        self.start_server(false);

        let connection = connect(Url::parse(&self.prefix).expect(ERRORS::TYPE_PARSE_FAILED));
        let mut socket;
        if connection.is_err() {
            self.kill_server();
            return ERRORS::CONNECTION_FAILED.to_string();
        } else {
            socket = connection.unwrap().0;
        }
        //send requests sequentially
        let _: Vec<()> = self
            .data
            .clone()
            .iter()
            .map(|d| self.process(d, &mut socket))
            .collect();

        let afterwrite = socket.write_message(Message::Close(None));
        if afterwrite.is_err() {
            self.kill_server();
            return ERRORS::SOCKET_WRITE_MESSAGE_FAILED.to_string();
        }

        self.kill_server();
        Command::new("redis-cli")
            .arg("FLUSHALL")
            .output()
            .expect("failed to clear database");
        // sleep 1 second for server startup
        std::thread::sleep(Duration::from_secs(1));
        return "Success".to_string();
    }
}
