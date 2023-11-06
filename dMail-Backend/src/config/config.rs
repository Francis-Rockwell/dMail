/*! 配置文件解析 */

use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    process::exit,
};

use once_cell::sync::{Lazy, OnceCell};
use regex::Regex;
use serde::{Deserialize, Serialize};

static CONFIG: OnceCell<Config> = OnceCell::new();

/** `PWD_PATTERN` 从配置文件中生成的密码正则匹配式
 */
pub static PWD_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(&Config::get().user.password_check).unwrap());

/** `Config` 配置信息的数据类型
 */
#[derive(Serialize, Debug, Deserialize)]
pub struct Config {
    pub server_worker_num: usize,
    pub http_worker_num: usize,
    pub tls: TlsConfig,
    pub safety: SafetyConfig,
    pub protocol: ProtocolConfig,
    pub email: EmailConfig,
    pub user: UserConfig,
    pub database: DatabaseConfig,
    pub s3_oss: S3Config,
}

/** `TlsConfig` tls有关的配置信息的数据类型
 */
#[derive(Serialize, Debug, Deserialize)]
pub struct TlsConfig {
    pub enable: bool,
    pub private_key_file: String,
    pub cert_chain_file: String,
}

/** `ProtocolConfig` 服务端与客户端之间通信协议的配置信息的数据类型
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct ProtocolConfig {
    /// 当用户登录拉取所有未读消息时，单个聊天能发送的最大消息数量
    pub max_messages_num_in_one_chat_when_pulling: u8,
    /// 当用户使用get_messages_in_chat接口时，单次能发送的最大消息数量
    pub max_messages_num_in_one_chat_when_getting: u8,
    /// 当群聊人数超过这一阈值时，会发送到server_worker进行信息发送
    pub worker_send_messages_member_num_threshold: u8,
}

/** `SafetyConfig` 安全性保证的配置信息的数据类型
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct SafetyConfig {
    pub max_msg_length: u16,
    pub max_notice_length: u16,
}

/** `EmailConfig` 邮件配置信息的数据类型
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct EmailConfig {
    pub enable: bool,
    pub relay: String,
    pub relay_user_name: String,
    pub relay_password: String,
    pub from_name: String,
    pub from: String,
    pub connection_pool_size: u32,
    pub cool_down_sec: i32,
    pub valid_time_sec: i32,
    pub email_code_len: u32,
}

/** `UserConfig` 用户客户端的配置信息的数据类型
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct UserConfig {
    pub token_expire_time: u32,
    pub max_user_name_length: u32,
    pub heart_beat_time: u32,
    pub password_check: String,
    pub sender_revoke_expire: u32,
}

/** `DatabaseConfig` 数据库的配置信息的数据类型
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub address: String,
    pub pool_max_open: usize,
    pub pool_max_idle: usize,
    pub pool_timeout: usize,
    pub pool_expire: usize,
}

/** `S3Config` oss配置信息的数据类型
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct S3Config {
    pub enable: bool,
    pub bucket_name: String,
    pub region: String,
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,

    pub presign_put_file_expire: u32,
    pub presign_put_image_expire: u32,
    pub presign_get_expire: u32,
}

impl Config {
    /** `init` 试图读取配置文件，生成Config
     */
    pub fn init() {
        match Self::try_read_from_file() {
            Ok(config) => {
                CONFIG.set(config).unwrap();
            }
            Err(_) => {
                println!("未找到配置文件，创建默认配置文件");

                let path = Config::get_config_path().expect("获得配置文件路径失败");

                let config = Config::default();
                let json = serde_json::to_string_pretty(&config).unwrap();

                let mut file = File::create(path).expect("创建配置文件失败");

                file.write_fmt(format_args!("{}", json))
                    .expect("写入配置文件失败");

                println!("默认配置文件写入到{}，请配置数据库与邮件服务", json);
                exit(0);
            }
        }
    }

    /** `get` 调用已初始化的CONFIG
     */
    pub fn get() -> &'static Config {
        return CONFIG.get().expect("Config未初始化");
    }

    fn default() -> Config {
        let core_num = num_cpus::get();
        Config {
            server_worker_num: core_num / 2,
            http_worker_num: core_num,
            tls: TlsConfig {
                enable: false,
                private_key_file: "private.pem".to_string(),
                cert_chain_file: "cert.pem".to_string(),
            },
            safety: SafetyConfig {
                max_msg_length: 500,
                max_notice_length: 500,
            },
            protocol: ProtocolConfig {
                max_messages_num_in_one_chat_when_pulling: 4,
                max_messages_num_in_one_chat_when_getting: 30,
                worker_send_messages_member_num_threshold: 5,
            },
            email: EmailConfig {
                enable: true,
                relay: "smtp.example.com".to_string(),
                relay_user_name: "your_user_name".to_string(),
                relay_password: "your_password".to_string(),
                from: "nobody@domain.tld".to_string(),
                from_name: "nobody".to_string(),
                connection_pool_size: ((core_num as f32 + 4.0) / 4.0) as u32,
                cool_down_sec: 30,
                valid_time_sec: 60,
                email_code_len: 6,
            },
            user: UserConfig {
                max_user_name_length: 32,
                heart_beat_time: 5,
                password_check: "^[a-fA-F0-9]{64}$".to_string(),
                sender_revoke_expire: 180,
                token_expire_time: 604800,
            },
            database: DatabaseConfig {
                address: "redis://127.0.0.1:6379/".to_string(),
                pool_max_open: 16,
                pool_max_idle: 8,
                pool_timeout: 1,
                pool_expire: 60,
            },
            s3_oss: S3Config {
                enable: true,
                region: "zh-east-1".to_string(),
                endpoint: "http://localhost:9000".to_owned(),
                bucket_name: "dMail".to_string(),
                access_key: "YOUR_ACCESS_KEY".to_string(),
                secret_key: "YOUR_SECRET_KEY".to_string(),
                presign_put_file_expire: 3600,
                presign_put_image_expire: 120,
                presign_get_expire: 3600 * 24 * 7,
            },
        }
    }

    fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut path = std::env::current_dir()?;
        path.push("config/config.json");
        return Ok(path);
    }

    fn try_read_from_file() -> Result<Config, Box<dyn std::error::Error>> {
        let path = Self::get_config_path()?;

        let mut file = File::open(path)?;

        let mut json = String::new();
        file.read_to_string(&mut json).expect("配置文件读取失败");

        let obj = serde_json::from_str(&json).expect("配置文件序列化失败，请检查格式");

        return Ok(obj);
    }
}
