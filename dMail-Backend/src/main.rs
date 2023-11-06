use actix_web::{get, http::header::ContentType, web, App, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use dMail::{
    chat::send_admin_message_to_group_chat,
    config::Config,
    database::{self},
    server::server_state::ServerState,
    user::{
        user_session::{UserSessionData, UserSessionState},
        UserCreateGroupChatData,
    },
};
use dotenv::dotenv;
use log::debug;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};

use std::{cell::RefCell, fs::File, io::BufReader, rc::Rc};

use dMail::user;
use dMail::user::user_session::WsUserSession;

#[get("/ws")]
async fn connect_web_socket(
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let addr = match req.peer_addr() {
        Some(addr) => addr,
        None => {
            return Ok(HttpResponse::BadRequest().into());
        }
    };
    ws::start(
        WsUserSession {
            data: Rc::new(RefCell::new(UserSessionData {
                state: UserSessionState::Started,
                client_ip: addr,
                user_id: None,
                user_pub_key: None,
            })),
            cipher: None,
            last_receive_time: std::time::Instant::now(),
        },
        &req,
        stream,
    )
}

#[actix_web::main]
pub async fn main() -> std::io::Result<()> {
    dotenv().ok();
    Config::init();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));

    ServerState::start().await;

    let server = HttpServer::new(move || {
        App::new()
            .service(connect_web_socket)
            .service(user::http_request::apply_for_email_code)
    })
    .workers(Config::get().http_worker_num);

    let tls_config = &Config::get().tls;

    if tls_config.enable {
        let rust_tls_config =
            load_rustls_config(&tls_config.cert_chain_file, &tls_config.private_key_file);
        return server
            .bind_rustls("0.0.0.0:8080", rust_tls_config)?
            .run()
            .await;
    } else {
        return server.bind("0.0.0.0:8080")?.run().await;
    }
}

fn load_rustls_config(cert_path: &str, key_path: &str) -> rustls::ServerConfig {
    // init server config builder with safe defaults
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth();

    // load TLS key/cert files
    let cert_file = &mut BufReader::new(File::open(cert_path).unwrap());
    let key_file = &mut BufReader::new(File::open(key_path).unwrap());

    // convert files to key/cert objects
    let cert_chain = certs(cert_file)
        .unwrap()
        .into_iter()
        .map(Certificate)
        .collect();
    let mut keys: Vec<PrivateKey> = pkcs8_private_keys(key_file)
        .unwrap()
        .into_iter()
        .map(PrivateKey)
        .collect();

    // exit if no keys could be parsed
    if keys.is_empty() {
        eprintln!("Could not locate PKCS 8 private keys.");
        std::process::exit(1);
    }

    config.with_single_cert(cert_chain, keys.remove(0)).unwrap()
}
