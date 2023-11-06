use chrono::Utc;
use once_cell::sync::Lazy;
use s3::{error::S3Error, serde_types::HeadObjectResult, Bucket, Region};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    config::{
        datatype::{Timestamp, UserID},
        Config,
    },
    database,
};

// TODO : 图像和文件分Bucket
/// `BUCKET`用来连接oss的客户端
pub static BUCKET: Lazy<Bucket> = Lazy::new(|| create_bucket());

/// `ObjectUploadRequest`文件上传申请数据类型
#[derive(Debug, Serialize, Deserialize)]
pub struct ObjectUploadRequest {
    pub user_id: UserID,
    pub user_hash: String,
    pub file_size: u64,
    pub path: String,
}

/// `PresignUrl`预签名url数据类型
#[derive(Debug, Serialize, Deserialize)]
pub struct PresignUrl {
    pub path: String,
    pub url: String,
    pub expire: Timestamp,
}

fn create_bucket() -> Bucket {
    let s3_config = &Config::get().s3_oss;
    return Bucket::new(
        &s3_config.bucket_name,
        Region::Custom {
            region: s3_config.region.clone(),
            endpoint: s3_config.endpoint.clone(),
        },
        s3::creds::Credentials {
            access_key: Some(s3_config.access_key.clone()),
            secret_key: Some(s3_config.secret_key.clone()),
            security_token: None,
            session_token: None,
            expiration: None,
        },
    )
    .expect("创建S3 Bucket失败，请检查OSS配置")
    .with_path_style();
}

fn get_presign_put_url(suffix: String, expire: u32) -> Result<PresignUrl, S3Error> {
    let mut path = "/".to_string();
    path.push_str(&Uuid::new_v4().simple().to_string());
    path.push_str(&suffix);

    let url = BUCKET.presign_put(&path, expire, None)?;

    return Ok(PresignUrl {
        path,
        url,
        expire: (Utc::now().timestamp_millis() + (expire * 1000) as i64) as Timestamp,
    });
}

/// `get_presign_put_file_url`获取上传文件的预签名url
pub fn get_presign_put_file_url(suffix: String) -> Result<PresignUrl, S3Error> {
    return get_presign_put_url(suffix, Config::get().s3_oss.presign_put_file_expire);
}

/// `get_presign_put_image_url`获取上传图片的预签名url
pub fn get_presign_put_image_url(suffix: String) -> Result<PresignUrl, S3Error> {
    return get_presign_put_url(suffix, Config::get().s3_oss.presign_put_file_expire);
}

/// `get_object_stat`获取桶内对象状态
pub async fn get_object_stat(path: &String) -> Result<HeadObjectResult, ()> {
    // TODO : 连接池化
    match BUCKET.head_object(path).await {
        Ok((result, _u16)) => {
            if result.e_tag.is_none() || result.content_length.is_none() {
                return Err(());
            }
            return Ok(result);
        }
        Err(_) => Err(()),
    }
}

/// `create_pub_url`生成公共url
pub async fn create_pub_url(hash: &String, path: String, expire: u32) -> Result<PresignUrl, ()> {
    match BUCKET.presign_get(&path, expire, None) {
        Ok(url) => {
            let presign_url = PresignUrl {
                path,
                url,
                expire: Utc::now().timestamp_millis() as Timestamp,
            };
            database::write_file_public_url(&hash, &presign_url).await?;
            return Ok(presign_url);
        }
        Err(_) => Err(()),
    }
}

/// `get_public_url_and_auto_renew`获取公共url并自动更新
pub async fn get_public_url_and_auto_renew(hash: &String) -> Result<Option<String>, ()> {
    if let Some((pub_url, _)) = database::get_file_public_url(hash).await? {
        if pub_url.expire < Utc::now().timestamp_millis() as Timestamp {
            let new_pub_url =
                create_pub_url(hash, pub_url.path, Config::get().s3_oss.presign_get_expire).await?;
            return Ok(Some(new_pub_url.url));
        }
        return Ok(Some(pub_url.url));
    } else {
        return Ok(None);
    }
}
