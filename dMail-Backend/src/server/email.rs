/*!
 邮箱相关的函数
*/
use std::time::Duration;

use actix::clock::Instant;
use lettre::{
    message::Mailbox,
    transport::smtp::{
        authentication::{Credentials, Mechanism},
        PoolConfig,
    },
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use log::info;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tokio::time;

use crate::config::{
    datatype::{EmailCodeValue, Timestamp},
    Config,
};

use super::server_state::{email_sender, users_email_codes};

pub type EmailSender = AsyncSmtpTransport<Tokio1Executor>;

static SERVER_MAILBOX: OnceCell<Mailbox> = OnceCell::new();

/** `EmailCode` 邮箱验证码数据类型
*/
#[derive(Debug)]
pub struct EmailCode {
    pub value: EmailCodeValue,
    pub timestamp: Timestamp,
}

/** `start_sender` 启动邮件客户端
*/
pub fn start_sender() -> EmailSender {
    let email_config = &Config::get().email;

    info!("正在启动邮件客户端");

    SERVER_MAILBOX
        .set(Mailbox {
            name: Some(email_config.from_name.clone()),
            email: email_config
                .from
                .parse()
                .expect("Config-Email-From 解析失败，请输入正确的邮件地址"),
        })
        .ok();
    let sender: AsyncSmtpTransport<Tokio1Executor> =
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&email_config.relay)
            .expect("连接邮件中转服务器失败")
            .credentials(Credentials::new(
                email_config.relay_user_name.clone(),
                email_config.relay_password.clone(),
            ))
            .authentication(vec![Mechanism::Login])
            .pool_config(PoolConfig::new().max_size(email_config.connection_pool_size))
            .build();
    return sender;
}

/** `test_sender` 测试与服务器连接
*/
pub async fn test_sender() {
    info!("正在进行邮件服务器连接测试");

    let deadline = Instant::now() + Duration::from_secs(5);

    let test = time::timeout_at(deadline, email_sender.test_connection())
        .await
        .expect("连接测试超时，请检查邮件配置")
        .expect("连接测试失败，请检查邮件配置");

    assert!(test, "连接测试失败，请检查邮件配置");
    info!("连接测试成功");
}

/** `SendEmailCodeError` 发送邮箱验证码错误的数据类型
*/
#[derive(Serialize, Deserialize, Debug)]
pub enum SendEmailCodeError {
    AddressParseFailed,
    EmailBuildFailed,
    SendFailed,
}

/** `send_email_code` 发送验证码
*/
pub async fn send_email_code(
    user_name: Option<String>,
    receiver: &String,
    code: EmailCodeValue,
) -> Result<(), SendEmailCodeError> {
    if Config::get().email.enable == false {
        return Ok(());
    }

    // let email_config = &Config::get().email;
    // TODO : 邮件美化

    // 为什么这里Mailbox传入会夺取所有权()，发一次复制一个Mailbox么
    let email = Message::builder()
        .from(SERVER_MAILBOX.get().unwrap().clone())
        .to(Mailbox::new(
            user_name,
            receiver
                .parse()
                .map_err(|_| SendEmailCodeError::AddressParseFailed)?,
        ))
        .subject("Email Code")
        .body(format!(
            "您的验证码为：{}, 请在{}s内完成验证",
            code,
            Config::get().email.valid_time_sec
        ))
        .map_err(|_| SendEmailCodeError::EmailBuildFailed)?;

    email_sender
        .send(email)
        .await
        .map_err(|_| SendEmailCodeError::SendFailed)?;

    users_email_codes.insert(
        receiver.clone(),
        EmailCode {
            value: code,
            timestamp: chrono::Utc::now().timestamp_millis() as u64,
        },
    );
    return Ok(());
}

/** `send_random_code` 封装`send_email_code`，发送随机的验证码
*/
pub async fn send_random_code(
    user_name: Option<String>,
    receiver: &String,
) -> Result<(), SendEmailCodeError> {
    let code: EmailCodeValue = rand::random::<u32>() % 1000000;
    return send_email_code(user_name, receiver, code).await;
}

/** `check_and_consume_email_code` 校验验证码，从验证码表中删除对应的验证码项
*/
pub fn check_and_consume_email_code(email: &String, input_code: EmailCodeValue) -> bool {
    if !Config::get().email.enable {
        return true;
    }
    {
        let code = match users_email_codes.get(email) {
            Some(code) => code,
            None => return false,
        };

        if chrono::Utc::now().timestamp_millis() as u64 - code.timestamp
            > (Config::get().email.valid_time_sec * 1000) as u64
        {
            return false;
        }

        if code.value != input_code {
            return false;
        }
    }

    users_email_codes.remove(email);

    return true;
}
