use crate::user::user_session::protocol::ClientToServerMessage;
use crate::user::{UserGetUserReadInGroupData, UserLoginData, UserPullData, UserRequestHandler};
use crate::utils;
use crate::utils::aes::AesGcmHelper;
use crate::{
    database,
    user::{
        user_session::{protocol::ServerToClientMessage, send_message},
        ChatMessageType, UserCreateGroupChatData, UserGetMessagesData, UserGroupOwnerTransferData,
        UserMediaCallAnswer, UserMediaCallData, UserMediaCallStop, UserMediaCallStopReason,
        UserMediaCallType, UserMediaIceCandidate, UserPullGroupNoticeData, UserRegisterData,
        UserRemoveGroupMemberData, UserRequestState, UserRequsetContent, UserRevokeMessageData,
        UserRevokeMethod, UserSendGroupNoticeData, UserSendMessageData, UserSendRequestData,
        UserSetAlreadyReadData, UserSetGroupAdminData, UserSolveRequestData,
        UserUnsetGroupAdminData, UserUpdateData, UserUpdateGroupContent, UserUpdateGroupData,
        UserUploadFileRequestData,
    },
    utils::rsa::get_private_key_from_base64_pkcs1_pem,
};
use aes_gcm::{aes::Aes128, Aes128Gcm, AesGcm, KeyInit};
use generic_array::typenum::{
    bit::{B0, B1},
    UInt, UTerm,
};

use once_cell::sync::OnceCell;
use rsa::Pkcs1v15Encrypt;
use tungstenite::{Message, WebSocket};

static SYM_KEY_0: OnceCell<AesGcm<Aes128, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>> =
    OnceCell::new();

static SYM_KEY: OnceCell<AesGcm<Aes128, UInt<UInt<UInt<UInt<UTerm, B1>, B1>, B0>, B0>>> =
    OnceCell::new();

pub fn encode_0(msg: String) -> String {
    SYM_KEY_0
        .get()
        .unwrap()
        .encrypt_with_default_nouce_to_base64(&msg)
        .unwrap()
}

pub fn decode_0(msg: String) -> String {
    SYM_KEY_0
        .get()
        .unwrap()
        .decrypt_with_default_nouce_from_base64(&msg)
        .unwrap()
}

pub fn encode(msg: String) -> String {
    SYM_KEY
        .get()
        .unwrap()
        .encrypt_with_default_nouce_to_base64(&msg)
        .unwrap()
}

pub fn decode(msg: String) -> String {
    SYM_KEY
        .get()
        .unwrap()
        .decrypt_with_default_nouce_from_base64(&msg)
        .unwrap()
}

pub fn test_for_user_session_set_connection_pub_key_0<Stream>(socket: &mut WebSocket<Stream>)
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::SetConnectionPubKey("MIIBCgKCAQEAuE9wq7rtUvWmPUYVKqK+zg+HDvMIzZsZccwnoFgGF8U3xyIBSoEdEMV5qBKIPOI7DSHzChi3l061S49/eh4Xv4AovIoxjTK2WXnFBS8oFTWB8hx1yuh4E8bGFo2R7xilSZXcUJ+rs03oXlR+Gsx/e0y1l/nb+I1Sb3HDfYKC1IU4YkQ7YY/7KgqdAVnWt9SRoFbWQeM/RodlbgMBaXx+a8Wjn6H9ZA9gJulNUtvzgccnlFR6wd6yxiC5yJaOPj4K5v1uHs8iGaqRQ/XX6j3tja/z0TU8kej1wPB74d7PZTXJWj97054nNdoYZqfFRLz6GbacF7cGKgckmEg7aM7JAQIDAQAB".to_string());
    let _ = socket.write_message(Message::Text(serde_json::to_string(&request).unwrap()));
    let after_read = socket.read_message();
    let resp = after_read.unwrap().to_string();
    let private_key_str = "MIIEowIBAAKCAQEAuE9wq7rtUvWmPUYVKqK+zg+HDvMIzZsZccwnoFgGF8U3xyIBSoEdEMV5qBKIPOI7DSHzChi3l061S49/eh4Xv4AovIoxjTK2WXnFBS8oFTWB8hx1yuh4E8bGFo2R7xilSZXcUJ+rs03oXlR+Gsx/e0y1l/nb+I1Sb3HDfYKC1IU4YkQ7YY/7KgqdAVnWt9SRoFbWQeM/RodlbgMBaXx+a8Wjn6H9ZA9gJulNUtvzgccnlFR6wd6yxiC5yJaOPj4K5v1uHs8iGaqRQ/XX6j3tja/z0TU8kej1wPB74d7PZTXJWj97054nNdoYZqfFRLz6GbacF7cGKgckmEg7aM7JAQIDAQABAoIBAGfJFAEf+ZPFkB7g3/pqOld+lubsJADjXaie9ZFs/8FS5N3VYDS8D8np6V+jT+Q44Fe8zkbZNEiXoa8y1u3FFEpZuJaymsSP0e8AitkofMG0p7/WFt5zmWpJfDIm9g5VKn4NTUp5Hw6QyFCV84zTqtWblIZHxH5p1gm7XgHHBDT2+jBSzYEtxXLuho/oss6HJo/I54yELR+5fciRgg0Qud8ro7+QwcV9kK3uBWIbOfWShk9lpCzPvshf8pD2oDoJiXUyH1O6y6qPT8RI7QPIMu+0yxwkz7vAFYIz33PhRbVVWqymv9+hUpiERIUgip5UcSXfukilysu4ZW+whuDH0SECgYEA6tKF43bYnZSsaFN9pAWpL9wLZmP6BOAprpBcFNb2opMr4bdRziAulG73dFRSFIVINYSG25RZaqD6QGfUEdGQk6Zk1tZ0USaPc8/aIyKrxDON1j4OGK05/PftxpkfO8J+DtV0AkId7um3s7rf2sYgjf138fi/DFF1ML/YJ+5yo3sCgYEAyO63iFYwIgg3PNkKaCSywVZOfmysuhFNI3S3HE7pCUwov6pZYziPyQfPwCGpsnk34nRHIb3IBI0L3xg4EojpmTlpjhXW2jxWz9ibAOLzELZ6uNgiVzj7b4eEOQglkO0PaUJ3z9/2hUG1iC2pD/Y1wnHsa1pnVMrXM+Lwx5tVTrMCgYBwwUfzEkUvXY1vxu9kjCdSSNncf5M1NiItpTnh89qX9A01JB6O2JslQSdnX3nOSrWCpTFQTKqm7cdcl76YE8XVcCeplW5i7R4i4SKAjoxl+M9ZmZCRPtTCaJZvL2V0/44iN1KuJutSpj1Eey40UcCeDaDDusqZ8p9QGj6D5hZ78wKBgH3Lpi++9ed4iUyY/UDyKM+N/xp7YzAigM6/1Zvtc0wU2DYWqlvKH4rWTySUbq+D4I7wCVCAhmcC/vmvKfvAp678GK+R0K9Us2zwySom69H8zJxJBEbjL9dFWmxyQ0KWh914dZY5Oxd2afZVz9BkbofL1x3mvWaCj3S2kdQF1cStAoGBALAuKY5JpV4uv6542ono1zaHtdAzh3G+zQS5RhieP4aNVD4EgW/nAzB3uw0gF/ZrBe7VJOMbkegJ2h3KQbKU80xrTIVHuO8EB1zJk7bCF/D8juDumxpwRPMhjl7LRRfFxX6gsJf6YbLqf/90UUp39noOHIktbA5TvgoTlr064yqU".to_string();
    let private_key = get_private_key_from_base64_pkcs1_pem(private_key_str.to_string()).unwrap();
    let encoded_sym_key;
    if let Ok(ServerToClientMessage::SetConnectionSymKey(key)) =
        serde_json::from_str::<ServerToClientMessage>(&resp)
    {
        let decoded_key = utils::base64::decode(&key);
        encoded_sym_key = decoded_key.unwrap();
    } else {
        panic!("set_sym_key");
    };
    let decoded_sym_key = private_key
        .decrypt(Pkcs1v15Encrypt, &encoded_sym_key)
        .unwrap();
    let after_decode = Aes128Gcm::new_from_slice(
        &utils::base64::decode((std::str::from_utf8(&decoded_sym_key)).unwrap()).unwrap(),
    );
    let sym_key = after_decode.unwrap();
    let _ = SYM_KEY_0.set(sym_key);
}

pub fn test_for_user_session_set_connection_pub_key<Stream>(socket: &mut WebSocket<Stream>)
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::SetConnectionPubKey("MIIBCgKCAQEAuE9wq7rtUvWmPUYVKqK+zg+HDvMIzZsZccwnoFgGF8U3xyIBSoEdEMV5qBKIPOI7DSHzChi3l061S49/eh4Xv4AovIoxjTK2WXnFBS8oFTWB8hx1yuh4E8bGFo2R7xilSZXcUJ+rs03oXlR+Gsx/e0y1l/nb+I1Sb3HDfYKC1IU4YkQ7YY/7KgqdAVnWt9SRoFbWQeM/RodlbgMBaXx+a8Wjn6H9ZA9gJulNUtvzgccnlFR6wd6yxiC5yJaOPj4K5v1uHs8iGaqRQ/XX6j3tja/z0TU8kej1wPB74d7PZTXJWj97054nNdoYZqfFRLz6GbacF7cGKgckmEg7aM7JAQIDAQAB".to_string());
    let _ = socket.write_message(Message::Text(serde_json::to_string(&request).unwrap()));
    let after_read = socket.read_message();
    let resp = after_read.unwrap().to_string();
    let private_key_str = "MIIEowIBAAKCAQEAuE9wq7rtUvWmPUYVKqK+zg+HDvMIzZsZccwnoFgGF8U3xyIBSoEdEMV5qBKIPOI7DSHzChi3l061S49/eh4Xv4AovIoxjTK2WXnFBS8oFTWB8hx1yuh4E8bGFo2R7xilSZXcUJ+rs03oXlR+Gsx/e0y1l/nb+I1Sb3HDfYKC1IU4YkQ7YY/7KgqdAVnWt9SRoFbWQeM/RodlbgMBaXx+a8Wjn6H9ZA9gJulNUtvzgccnlFR6wd6yxiC5yJaOPj4K5v1uHs8iGaqRQ/XX6j3tja/z0TU8kej1wPB74d7PZTXJWj97054nNdoYZqfFRLz6GbacF7cGKgckmEg7aM7JAQIDAQABAoIBAGfJFAEf+ZPFkB7g3/pqOld+lubsJADjXaie9ZFs/8FS5N3VYDS8D8np6V+jT+Q44Fe8zkbZNEiXoa8y1u3FFEpZuJaymsSP0e8AitkofMG0p7/WFt5zmWpJfDIm9g5VKn4NTUp5Hw6QyFCV84zTqtWblIZHxH5p1gm7XgHHBDT2+jBSzYEtxXLuho/oss6HJo/I54yELR+5fciRgg0Qud8ro7+QwcV9kK3uBWIbOfWShk9lpCzPvshf8pD2oDoJiXUyH1O6y6qPT8RI7QPIMu+0yxwkz7vAFYIz33PhRbVVWqymv9+hUpiERIUgip5UcSXfukilysu4ZW+whuDH0SECgYEA6tKF43bYnZSsaFN9pAWpL9wLZmP6BOAprpBcFNb2opMr4bdRziAulG73dFRSFIVINYSG25RZaqD6QGfUEdGQk6Zk1tZ0USaPc8/aIyKrxDON1j4OGK05/PftxpkfO8J+DtV0AkId7um3s7rf2sYgjf138fi/DFF1ML/YJ+5yo3sCgYEAyO63iFYwIgg3PNkKaCSywVZOfmysuhFNI3S3HE7pCUwov6pZYziPyQfPwCGpsnk34nRHIb3IBI0L3xg4EojpmTlpjhXW2jxWz9ibAOLzELZ6uNgiVzj7b4eEOQglkO0PaUJ3z9/2hUG1iC2pD/Y1wnHsa1pnVMrXM+Lwx5tVTrMCgYBwwUfzEkUvXY1vxu9kjCdSSNncf5M1NiItpTnh89qX9A01JB6O2JslQSdnX3nOSrWCpTFQTKqm7cdcl76YE8XVcCeplW5i7R4i4SKAjoxl+M9ZmZCRPtTCaJZvL2V0/44iN1KuJutSpj1Eey40UcCeDaDDusqZ8p9QGj6D5hZ78wKBgH3Lpi++9ed4iUyY/UDyKM+N/xp7YzAigM6/1Zvtc0wU2DYWqlvKH4rWTySUbq+D4I7wCVCAhmcC/vmvKfvAp678GK+R0K9Us2zwySom69H8zJxJBEbjL9dFWmxyQ0KWh914dZY5Oxd2afZVz9BkbofL1x3mvWaCj3S2kdQF1cStAoGBALAuKY5JpV4uv6542ono1zaHtdAzh3G+zQS5RhieP4aNVD4EgW/nAzB3uw0gF/ZrBe7VJOMbkegJ2h3KQbKU80xrTIVHuO8EB1zJk7bCF/D8juDumxpwRPMhjl7LRRfFxX6gsJf6YbLqf/90UUp39noOHIktbA5TvgoTlr064yqU".to_string();
    let private_key = get_private_key_from_base64_pkcs1_pem(private_key_str.to_string()).unwrap();
    let encoded_sym_key;
    if let Ok(ServerToClientMessage::SetConnectionSymKey(key)) =
        serde_json::from_str::<ServerToClientMessage>(&resp)
    {
        let decoded_key = utils::base64::decode(&key);
        encoded_sym_key = decoded_key.unwrap();
    } else {
        panic!("set_sym_key");
    };
    let decoded_sym_key = private_key
        .decrypt(Pkcs1v15Encrypt, &encoded_sym_key)
        .unwrap();
    let after_decode = Aes128Gcm::new_from_slice(
        &utils::base64::decode((std::str::from_utf8(&decoded_sym_key)).unwrap()).unwrap(),
    );
    let sym_key = after_decode.unwrap();
    let _ = SYM_KEY.set(sym_key);
}

pub async fn test_for_user_session_reigister<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserRegisterData {
        user_name: "name".to_string(),
        password: "6B86B273FF34FCE19D6B804EFF5A3F5747ADA4EAA22F1D49C01E52DDB7875B4B".to_string(),
        email_code: 123456,
        email: "a@b.com".to_string(),
    };
    let request = ClientToServerMessage::Register(data);
    socket
        .write_message(Message::Text(encode_0(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;

    return Ok(());
}

pub fn test_for_user_sesssion_login<Stream>(socket: &mut WebSocket<Stream>) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserLoginData {
        email: "a@b.com".to_string(),
        password: Some(
            "6B86B273FF34FCE19D6B804EFF5A3F5747ADA4EAA22F1D49C01E52DDB7875B4B".to_string(),
        ),
        email_code: None,
        address: None,
        token: None,
    };
    let request = ClientToServerMessage::Login(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

// 刷情况数
pub async fn test_for_user_session_send_request_1<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserRegisterData {
        user_name: "name".to_string(),
        password: "1234".to_string(),
        email_code: 123456,
        email: "c@d.com".to_string(),
    };
    database::user_register(data).await;
    let data = UserSendRequestData {
        message: "message".to_string(),
        content: UserRequsetContent::MakeFriend { receiver_id: 2 },
        client_id: 1,
    };
    let request = ClientToServerMessage::SendRequest(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    database::make_two_users_be_friends(1, 2).await?;
    return Ok(());
}

pub async fn test_for_user_session_send_request_2<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserSendRequestData {
        message: "message".to_string(),
        content: UserRequsetContent::JoinGroup { chat_id: 2 },
        client_id: 1,
    };
    let request = ClientToServerMessage::SendRequest(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_send_request_3<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserSendRequestData {
        message: "message".to_string(),
        content: UserRequsetContent::GroupInvitation {
            receiver_id: 2,
            chat_id: 2,
        },
        client_id: 1,
    };
    let request = ClientToServerMessage::SendRequest(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_send_message_1<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let msg = UserSendMessageData {
        r#type: ChatMessageType::Text,
        client_id: 1,
        chat_id: 1,
        timestamp: 1,
        serialized_content: "content".to_string(),
    };
    let request = ClientToServerMessage::SendMessage(msg);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_solve_request_1<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserSendRequestData {
        message: "message".to_string(),
        content: UserRequsetContent::MakeFriend { receiver_id: 1 },
        client_id: 1,
    };
    database::write_user_request(2, data, &UserRequestHandler::One(1)).await?;
    let data = UserSolveRequestData {
        req_id: 1,
        answer: UserRequestState::Approved,
    };
    let request = ClientToServerMessage::SolveRequest(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_solve_request_2<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserSolveRequestData {
        req_id: 2,
        answer: UserRequestState::Approved,
    };
    let request = ClientToServerMessage::SolveRequest(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_solve_request_3<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserSolveRequestData {
        req_id: 3,
        answer: UserRequestState::Approved,
    };
    let request = ClientToServerMessage::SolveRequest(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_create_group_chat<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserCreateGroupChatData {
        name: "name".to_string(),
        avater_hash: "avater_hash".to_string(),
    };

    let request = ClientToServerMessage::CreateGroupChat(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_send_message_2<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    database::add_user_to_chat(2, 2).await?;
    let msg = UserSendMessageData {
        r#type: ChatMessageType::MentionText,
        client_id: 1,
        chat_id: 2,
        timestamp: 1,
        serialized_content: "content".to_string(),
    };

    let request = ClientToServerMessage::SendMessage(msg);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_get_user_info<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::GetUserInfo(1);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_get_messages<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserGetMessagesData {
        chat_id: 1,
        start_id: 1,
        end_id: None,
    };
    let request = ClientToServerMessage::GetMessages(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_get_chat_info<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::GetChatInfo(1);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_set_user_setting<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::SetUserSetting("setting".to_string());
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_update_user_info_1<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserUpdateData::UserName {
        new_name: "new_name".to_string(),
    };
    let request = ClientToServerMessage::UpdateUserInfo(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_update_user_info_2<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserUpdateData::AvaterHash {
        new_hash: "new_avater".to_string(),
    };
    let request = ClientToServerMessage::UpdateUserInfo(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_update_user_info_3<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserUpdateData::Password {
        new_password: "6B86B273FF34FCE19D6B804EFF5A3F5747ADA4EAA22F1D49C01E52DDB7875B4B"
            .to_string(),
        email_code: 123456,
    };
    let request = ClientToServerMessage::UpdateUserInfo(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_unfriend<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::Unfriend(2);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_quit_group_chat<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::QuitGroupChat(2);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_set_user_already_read_1<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let msg = UserSendMessageData {
        r#type: ChatMessageType::Text,
        client_id: 1,
        chat_id: 1,
        timestamp: 1,
        serialized_content: "content".to_string(),
    };
    send_message(2, msg).await;
    let data = UserSetAlreadyReadData {
        chat_id: 1,
        in_chat_id: 3,
        private: true,
    };
    let request = ClientToServerMessage::SetAlreadyRead(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_set_user_already_read_2<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let msg = UserSendMessageData {
        r#type: ChatMessageType::Text,
        client_id: 1,
        chat_id: 2,
        timestamp: 1,
        serialized_content: "content".to_string(),
    };
    send_message(2, msg).await;
    let data = UserSetAlreadyReadData {
        chat_id: 2,
        in_chat_id: 2,
        private: false,
    };
    let request = ClientToServerMessage::SetAlreadyRead(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_upload_file_req<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserUploadFileRequestData {
        suffix: "suffix".to_string(),
        user_hash: "user_hash".to_string(),
        size: 1,
    };
    let request = ClientToServerMessage::UploadFileRequest(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}
pub async fn test_for_user_session_file_uploaded<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::FileUploaded(1);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_get_file_pub_url<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::GetFileUrl("user_hash".to_string());
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_get_group_users<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::GetGroupUsers(2);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_set_as_admin<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    database::add_user_to_chat(2, 2).await?;
    let data = UserSetGroupAdminData {
        chat_id: 2,
        user_id: 2,
    };
    let request = ClientToServerMessage::SetGroupAdmin(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_revoke_message_1<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserRevokeMessageData {
        chat_id: 2,
        in_chat_id: 2,
        method: UserRevokeMethod::GroupOwner,
    };

    let request = ClientToServerMessage::RevokeMessage(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_revoke_message_2<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let msg = UserSendMessageData {
        r#type: ChatMessageType::Text,
        client_id: 1,
        chat_id: 2,
        timestamp: 1,
        serialized_content: "content".to_string(),
    };
    send_message(1, msg).await;
    let data = UserRevokeMessageData {
        chat_id: 2,
        in_chat_id: 3,
        method: UserRevokeMethod::GroupAdmin,
    };

    let request = ClientToServerMessage::RevokeMessage(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_revoke_message_3<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let msg = UserSendMessageData {
        r#type: ChatMessageType::Text,
        client_id: 1,
        chat_id: 2,
        timestamp: 1,
        serialized_content: "content".to_string(),
    };
    send_message(1, msg).await;
    let data = UserRevokeMessageData {
        chat_id: 2,
        in_chat_id: 4,
        method: UserRevokeMethod::Sender,
    };

    let request = ClientToServerMessage::RevokeMessage(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_owner_transfer<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserGroupOwnerTransferData {
        chat_id: 2,
        user_id: 2,
    };

    let request = ClientToServerMessage::GroupOwnerTransfer(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_group_notice<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserSendGroupNoticeData {
        client_id: 1,
        chat_id: 2,
        notice: "notice".to_string(),
    };
    let request = ClientToServerMessage::SendGroupNotice(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_pull_group_notice<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserPullGroupNoticeData {
        chat_id: 1,
        last_notice_id: 1,
    };
    let request = ClientToServerMessage::PullGroupNotice(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_update_group_info_1<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserUpdateGroupData {
        chat_id: 2,
        content: UserUpdateGroupContent::GroupName {
            new_name: "new_name".to_string(),
        },
    };
    let request = ClientToServerMessage::UpdateGroupInfo(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_update_group_info_2<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserUpdateGroupData {
        chat_id: 2,
        content: UserUpdateGroupContent::Avater {
            new_avater: "new_avater".to_string(),
        },
    };
    let request = ClientToServerMessage::UpdateGroupInfo(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_remove_member<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserRegisterData {
        user_name: "name".to_string(),
        password: "6B86B273FF34FCE19D6B804EFF5A3F5747ADA4EAA22F1D49C01E52DDB7875B4B".to_string(),
        email_code: 123456,
        email: "e@f.com".to_string(),
    };
    database::user_register(data).await;
    database::add_user_to_chat(2, 3).await?;
    let data = UserRemoveGroupMemberData {
        chat_id: 2,
        user_id: 3,
    };
    let request = ClientToServerMessage::RemoveGroupMember(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_unset_group_admin<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserUnsetGroupAdminData {
        chat_id: 1,
        user_id: 2,
    };
    let request = ClientToServerMessage::UnsetGroupAdmin(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_get_group_owner<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::GetGroupOwner(2);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_get_group_admin<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::GetGroupAdmin(2);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_media_call<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserMediaCallData {
        friend_id: 2,
        call_type: UserMediaCallType::Video,
        serialized_offer: "offer".to_string(),
    };
    let request = ClientToServerMessage::MediaCall(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_media_call_answer_ice_candidate_stop<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserMediaCallAnswer {
        friend_id: 2,
        accept: true,
        serialized_answer: None,
    };
    let request = ClientToServerMessage::MediaCallAnswer(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    let data = UserMediaIceCandidate {
        friend_id: 2,
        serialized_candidate: "candidate".to_string(),
    };
    let request = ClientToServerMessage::MediaIceCandidate(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;

    let data = UserMediaCallStop {
        friend_id: 2,
        reason: UserMediaCallStopReason::Network,
    };
    let request = ClientToServerMessage::MediaCallStop(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_get_user_id<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::GetUserID("name".to_string());
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_get_user_read_in_private<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::GetUserReadInPrivate(1);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_get_user_read_in_group<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserGetUserReadInGroupData {
        chat_id: 2,
        in_chat_id: 2,
    };
    let request = ClientToServerMessage::GetUserReadInGroup(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_user_apply_for_token<Stream>(
    socket: &mut WebSocket<Stream>,
) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::ApplyForToken;
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_logoff<Stream>(socket: &mut WebSocket<Stream>) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let request = ClientToServerMessage::LogOff(123456);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn test_for_user_session_pull<Stream>(socket: &mut WebSocket<Stream>) -> Result<(), ()>
where
    Stream: std::io::Read + std::io::Write,
{
    let data = UserPullData {
        last_request_id: 1,
        notice_timestamp: 1,
    };
    let request = ClientToServerMessage::Pull(data);
    socket
        .write_message(Message::Text(encode(
            serde_json::to_string(&request).unwrap(),
        )))
        .map_err(|_| ())?;
    return Ok(());
}
