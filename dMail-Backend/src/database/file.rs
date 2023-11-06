use crate::{
    config::datatype::{SerializedFilePubUrl, UploadId},
    oss::{ObjectUploadRequest, PresignUrl},
};

use super::redis;

/// 获取文件的公共url
pub async fn get_file_public_url(
    hash: &String,
) -> Result<Option<(PresignUrl, SerializedFilePubUrl)>, ()> {
    return redis::get_file_public_url(hash).await;
}

/// 写入文件上传请求，获取upload_id
pub async fn write_upload_request(req: ObjectUploadRequest) -> Result<UploadId, ()> {
    return redis::write_upload_request(req).await;
}

/// 根据upload_id获取文件上传请求
pub async fn get_upload_request(upload_id: UploadId) -> Result<Option<ObjectUploadRequest>, ()> {
    return redis::get_upload_request(upload_id).await;
}

/// 写入文件的公开url
pub async fn write_file_public_url(hash: &String, pub_url: &PresignUrl) -> Result<(), ()> {
    return redis::write_file_public_url(hash, pub_url).await;
}

/// 获取文件的公开url
pub async fn get_file_url(hash: &String) -> Result<String, ()> {
    match redis::get_file_url(hash).await {
        Ok(presign_url) => match serde_json::from_str::<PresignUrl>(&presign_url) {
            Ok(presign_url) => Ok(presign_url.url),
            Err(_) => Err(()),
        },
        Err(_) => Err(()),
    }
}
