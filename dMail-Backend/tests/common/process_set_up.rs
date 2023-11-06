use lazy_static::lazy_static;
use serde_json;
use std::env::consts::EXE_EXTENSION;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Once;

use super::errors;
use dMail::config::config::{
    DatabaseConfig, EmailConfig, ProtocolConfig, S3Config, SafetyConfig, TlsConfig, UserConfig,
};
use dMail::config::Config;
use errors as ERRORS;

lazy_static! {
    pub static ref EXE_PATH: PathBuf = build("dMail");
}

pub fn build(name: &str) -> PathBuf {
    build_config();
    static CARGO_BUILD_ONCE: Once = Once::new();
    CARGO_BUILD_ONCE.call_once(|| {
        let mut build_command = Command::new("cargo");
        build_command.args(&["build", "--quiet"]);
        if !cfg!(debug_assertions) {
            build_command.arg("--release");
        }
        let build_status = build_command.status().unwrap();
        assert!(
            build_status.success(),
            "Cargo failed to build associated binaries."
        );
    });
    let flavor = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    Path::new("target")
        .join(flavor)
        .join(name)
        .with_extension(EXE_EXTENSION)
}

pub fn build_config() {
    let core_num = num_cpus::get();
    let config = Config {
        server_worker_num: core_num / 2,
        http_worker_num: core_num,
        safety: SafetyConfig {
            max_msg_length: 500,
            max_notice_length: 500,
        },
        protocol: ProtocolConfig {
            max_messages_num_in_one_chat_when_pulling: 20,
            max_messages_num_in_one_chat_when_getting: 30,
            worker_send_messages_member_num_threshold: 5,
        },
        email: EmailConfig {
            enable: false,
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
            token_expire_time: 604800,
            sender_revoke_expire: 180,
        },
        database: DatabaseConfig {
            address: "redis://localhost:6379/".to_string(),
            pool_max_open: 16,
            pool_max_idle: 8,
            pool_timeout: 1,
            pool_expire: 60,
        },
        tls: TlsConfig {
            enable: false,
            private_key_file: "private.pem".to_string(),
            cert_chain_file: "cert.pem".to_string(),
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
    };
    let json = serde_json::to_string_pretty(&config).unwrap();
    let mut path = std::env::current_dir().expect("");
    path.push("config");
    path.push("config.json");
    let mut file = File::create(path).expect(ERRORS::CREATE_CONFIG_FAILED);

    file.write_fmt(format_args!("{}", json))
        .expect(ERRORS::CREATE_CONFIG_FAILED);
}
