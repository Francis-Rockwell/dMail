use redis::AsyncCommands;

use crate::{
    config::datatype::{SerializedFilePubUrl, UploadId},
    oss::{ObjectUploadRequest, PresignUrl},
};

use super::{
    get_con,
    path::{FILE_UPLOAD_HASH, FILE_URL, LAST_UPLOAD_REQ_ID},
};

pub async fn get_file_public_url(
    hash: &String,
) -> Result<Option<(PresignUrl, SerializedFilePubUrl)>, ()> {
    let mut con = get_con().await?;

    let serialized_opt: Option<String> = con.hget(FILE_URL, hash).await.map_err(|_| ())?;

    return Ok(serialized_opt.map(|serialized| {
        (
            serde_json::from_str::<PresignUrl>(&serialized).unwrap(),
            serialized,
        )
    }));
}

pub async fn write_file_public_url(hash: &String, pub_url: &PresignUrl) -> Result<(), ()> {
    let serialized = serde_json::to_string(pub_url).unwrap();

    let mut con = get_con().await?;
    con.hset(FILE_URL, hash, &serialized)
        .await
        .map_err(|_| ())?;
    return Ok(());
}

pub async fn write_upload_request(req: ObjectUploadRequest) -> Result<UploadId, ()> {
    let mut con = get_con().await?;

    let upload_id: UploadId = con.incr(LAST_UPLOAD_REQ_ID, 1).await.map_err(|_| ())?;

    con.hset(
        FILE_UPLOAD_HASH,
        upload_id,
        serde_json::to_string(&req).unwrap(),
    )
    .await
    .map_err(|_| ())?;

    return Ok(upload_id);
}

pub async fn get_upload_request(upload_id: UploadId) -> Result<Option<ObjectUploadRequest>, ()> {
    let mut con = get_con().await?;

    let serialized_opt: Option<String> = con
        .hget(FILE_UPLOAD_HASH, upload_id)
        .await
        .map_err(|_| ())?;

    return Ok(serialized_opt.map(|serialized| serde_json::from_str(&serialized).unwrap()));
}

pub async fn get_file_url(hash: &String) -> Result<SerializedFilePubUrl, ()> {
    let mut con = get_con().await?;
    con.hget(FILE_URL, hash).await.map_err(|_| ())
}
