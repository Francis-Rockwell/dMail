use actix_web::get;
use actix_web::web;
use actix_web::App;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web_actors::ws;
use tungstenite::connect;
use tungstenite::WebSocket;
use url::Url;

use super::database_chat_test;
use super::database_file_test;
use super::database_request_test;
use super::database_user_test;
use super::user_session_test;
use super::user_session_test_supplement;
use crate::server::server_state::ServerState;
use crate::user::user_session::UserSessionData;
use crate::user::user_session::UserSessionState;
use crate::user::user_session::WsUserSession;
use crate::{config::Config, database};
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::thread;

#[test]
pub fn test_database() -> Result<(), ()> {
    Config::init();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();
    rt.block_on(database::connect_database());
    rt.block_on(database_test())?;
    return Ok(());
}

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

pub async fn test_main() {
    Config::init();
    env_logger::init();
    ServerState::start().await;
    let server = HttpServer::new(move || App::new().service(connect_web_socket)).workers(2);
    server.bind("0.0.0.0:8080").unwrap().run().await.unwrap();
}

#[test]
pub fn test_user_session() -> Result<(), ()> {
    thread::spawn(|| {
        actix_web::rt::System::with_tokio_rt(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .worker_threads(2)
                .thread_name("main-tokio")
                .build()
                .unwrap()
        })
        .block_on(test_main());
    });

    thread::sleep(std::time::Duration::from_secs(2));

    let connection_0 = connect(Url::parse("ws://127.0.0.1:8080/ws").unwrap());
    let mut socket_0 = connection_0.unwrap().0;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();
    rt.block_on(user_session_test_0(&mut socket_0))?;
    let connection = connect(Url::parse("ws://127.0.0.1:8080/ws").unwrap());
    let mut socket = connection.unwrap().0;
    rt.block_on(user_session_test(&mut socket))?;

    return Ok(());
}

pub async fn database_test() -> Result<(), ()> {
    database_user_test::test_for_check_password()?;
    database_user_test::test_for_check_username()?;
    database_user_test::test_for_user_register().await?;
    database_user_test::test_for_user_login_with_password().await?;
    let token = database_user_test::test_for_apply_for_token().await?;
    database_user_test::test_for_user_login_with_token(token).await?;
    database_user_test::test_for_get_user_id_by_email().await?;
    database_user_test::test_for_get_user_chat_list().await?;
    database_user_test::test_for_get_user_info().await?;
    database_user_test::test_for_get_user_email().await?;
    database_user_test::test_for_check_make_friend_error().await?;
    database_user_test::test_for_make_two_users_be_friends().await?;
    database_user_test::test_for_set_user_setting().await?;
    database_user_test::test_for_get_user_setting().await?;
    database_user_test::test_for_update_user_name().await?;
    database_user_test::test_for_update_user_avater().await?;
    database_user_test::test_for_update_user_password().await?;
    database_user_test::test_for_check_user_in_chat().await?;
    database_user_test::test_for_get_chat_id_by_friends().await?;
    database_user_test::test_for_get_user_id().await?;
    database_user_test::test_for_write_user_notice().await?;
    database_user_test::test_for_get_user_notice().await?;
    database_chat_test::test_for_creat_group_chat().await?;
    database_chat_test::test_for_add_user_to_chat().await?;
    database_chat_test::test_for_check_user_can_send_in_chat(1).await?;
    database_chat_test::test_for_check_user_can_send_in_chat(2).await?;
    database_user_test::test_for_user_log_off().await?;
    database_chat_test::test_for_get_chat_info().await?;
    database_chat_test::test_for_get_chat_owner().await?;
    database_chat_test::test_for_get_chat_user_list(1).await?;
    database_chat_test::test_for_get_chat_user_list(2).await?;
    database_chat_test::test_for_write_message_to_chat().await?;
    database_user_test::test_for_set_user_already_read().await?;
    database_chat_test::test_for_get_chats_last_messages().await?;
    database_chat_test::test_for_get_messages_in_chat().await?;
    database_chat_test::test_for_revoke_message().await?;
    database_chat_test::test_for_check_group_invitation_error().await?;
    database_chat_test::test_for_set_as_admin().await?;
    database_chat_test::test_for_quit_group_chat().await?;
    database_chat_test::test_for_check_invited_json_group_error().await?;
    database_chat_test::test_for_check_join_group_error().await?;
    database_chat_test::test_for_get_chat_admins_list().await?;
    database_chat_test::test_for_get_user_read_in_group().await?;
    database_chat_test::test_for_get_user_read_in_private().await?;
    database_chat_test::test_for_check_user_is_owner().await?;
    database_chat_test::test_for_check_user_is_admin().await?;
    database_chat_test::test_for_owner_transfer().await?;
    database_chat_test::test_for_add_group_notice().await?;
    database_chat_test::test_for_pull_group_notice().await?;
    database_chat_test::test_for_update_group_name().await?;
    database_chat_test::test_for_update_group_avater().await?;
    database_chat_test::test_for_unset_admin().await?;
    database_chat_test::test_for_check_is_group().await?;
    database_file_test::test_for_write_upload_request().await?;
    database_file_test::test_for_write_file_public_url().await?;
    database_file_test::test_for_get_file_public_url().await?;
    database_file_test::test_for_get_file_url().await?;
    database_file_test::test_for_get_upload_request().await?;
    database_chat_test::test_for_get_private_chat_user_list().await?;
    database_request_test::test_for_write_user_request_group().await?;
    database_request_test::test_for_write_user_request_one().await?;
    database_request_test::test_for_get_user_requests().await?;
    database_request_test::test_for_store_user_request().await?;
    database_request_test::test_for_get_user_request().await?;
    database_request_test::test_for_set_user_request_state().await?;
    database_request_test::test_for_write_friend_request_send().await?;
    database_request_test::test_for_delete_friend_request_send().await?;
    database_request_test::test_for_write_join_group_request_send().await?;
    database_request_test::test_for_write_invite_request_send().await?;
    database_request_test::test_for_delete_invite_request_send().await?;
    database_request_test::test_for_delete_join_group_request_send().await?;
    database_user_test::test_for_unfriend().await?;
    Command::new("redis-cli")
        .arg("FLUSHALL")
        .output()
        .expect("failed to clear database");
    return Ok(());
}

pub async fn user_session_test_0<Stream>(socket: &mut WebSocket<Stream>) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    user_session_test::test_for_user_session_set_connection_pub_key_0(socket);
    user_session_test::test_for_user_session_reigister(socket).await?;
    return Ok(());
}

pub async fn user_session_test<Stream>(socket: &mut WebSocket<Stream>) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    user_session_test::test_for_user_session_set_connection_pub_key(socket);
    user_session_test::test_for_user_sesssion_login(socket)?;
    user_session_test::test_for_user_session_send_request_1(socket).await?;
    user_session_test::test_for_user_session_create_group_chat(socket).await?;
    user_session_test::test_for_user_session_send_request_2(socket).await?;
    user_session_test::test_for_user_session_send_request_3(socket).await?;
    user_session_test::test_for_user_session_send_message_1(socket).await?;
    user_session_test::test_for_user_session_solve_request_1(socket).await?;
    user_session_test::test_for_user_session_solve_request_2(socket).await?;
    user_session_test::test_for_user_session_solve_request_3(socket).await?;
    user_session_test::test_for_user_session_send_message_2(socket).await?;
    user_session_test::test_for_user_session_get_user_info(socket).await?;
    user_session_test::test_for_user_session_get_messages(socket).await?;
    user_session_test::test_for_user_session_get_user_info(socket).await?;
    user_session_test::test_for_user_session_get_chat_info(socket).await?;
    user_session_test::test_for_user_session_set_user_setting(socket).await?;
    user_session_test::test_for_user_session_update_user_info_1(socket).await?;
    user_session_test::test_for_user_session_update_user_info_2(socket).await?;
    user_session_test::test_for_user_session_update_user_info_3(socket).await?;
    user_session_test::test_for_user_session_set_user_already_read_1(socket).await?;
    user_session_test::test_for_user_session_set_user_already_read_2(socket).await?;
    user_session_test::test_for_user_session_pull(socket).await?;
    user_session_test::test_for_user_session_upload_file_req(socket).await?;
    user_session_test::test_for_user_session_file_uploaded(socket).await?;
    user_session_test::test_for_user_session_get_file_pub_url(socket).await?;
    user_session_test::test_for_user_session_get_group_users(socket).await?;
    user_session_test::test_for_user_session_set_as_admin(socket).await?;
    user_session_test::test_for_user_session_unset_group_admin(socket).await?;
    user_session_test::test_for_user_session_revoke_message_1(socket).await?;
    user_session_test::test_for_user_session_revoke_message_2(socket).await?;
    user_session_test::test_for_user_session_revoke_message_3(socket).await?;
    user_session_test::test_for_user_session_group_notice(socket).await?;
    user_session_test::test_for_user_session_pull_group_notice(socket).await?;
    user_session_test::test_for_user_session_update_group_info_1(socket).await?;
    user_session_test::test_for_user_session_update_group_info_2(socket).await?;
    user_session_test::test_for_user_session_remove_member(socket).await?;
    user_session_test::test_for_user_session_get_group_admin(socket).await?;
    user_session_test::test_for_user_session_get_group_owner(socket).await?;
    user_session_test::test_for_user_session_media_call(socket).await?;
    user_session_test::test_for_user_session_media_call_answer_ice_candidate_stop(socket).await?;
    user_session_test::test_for_user_session_get_user_id(socket).await?;
    user_session_test::test_for_user_session_get_user_read_in_private(socket).await?;
    user_session_test::test_for_user_session_get_user_read_in_group(socket).await?;
    user_session_test::test_for_user_session_user_apply_for_token(socket).await?;
    user_session_test::test_for_user_session_unfriend(socket).await?;
    user_session_test::test_for_user_session_owner_transfer(socket).await?;
    user_session_test::test_for_user_session_quit_group_chat(socket).await?;
    user_session_test::test_for_user_session_logoff(socket).await?;
    Command::new("redis-cli")
        .arg("FLUSHALL")
        .output()
        .expect("failed to clear database");
    return Ok(());
}

#[test]
pub fn test_user_session_supplement() -> Result<(), ()> {
    Config::init();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap();
    rt.block_on(database::connect_database());
    rt.block_on(user_session_supplement_test())?;
    return Ok(());
}

pub async fn user_session_supplement_test() -> Result<(), ()> {
    user_session_test_supplement::test_for_user_session_quit_group_chat().await?;
    user_session_test_supplement::test_for_user_session_file().await?;
    user_session_test_supplement::test_for_user_session_owner_transfer().await?;
    user_session_test_supplement::test_for_user_session_remove_member().await?;
    Command::new("redis-cli")
        .arg("FLUSHALL")
        .output()
        .expect("failed to clear database");
    return Ok(());
}
