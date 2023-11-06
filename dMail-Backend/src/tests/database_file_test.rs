use crate::{
    database,
    oss::{ObjectUploadRequest, PresignUrl},
};

pub async fn test_for_write_upload_request() -> Result<(), ()> {
    let req = ObjectUploadRequest {
        user_id: 1,
        user_hash: "user_hash".to_string(),
        file_size: 1,
        path: "path".to_string(),
    };
    match database::write_upload_request(req).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_upload_request"),
    }
}

pub async fn test_for_get_upload_request() -> Result<(), ()> {
    match database::get_upload_request(1).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_upload_request"),
    }
}

pub async fn test_for_write_file_public_url() -> Result<(), ()> {
    let pub_url = PresignUrl {
        path: "path".to_string(),
        url: "url".to_string(),
        expire: 1,
    };
    match database::write_file_public_url(&"hash".to_string(), &pub_url).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("write_file_public_url"),
    }
}

pub async fn test_for_get_file_public_url() -> Result<(), ()> {
    match database::get_file_public_url(&"hash".to_string()).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_file_public_url"),
    }
}

pub async fn test_for_get_file_url() -> Result<(), ()> {
    match database::get_file_url(&"hash".to_string()).await {
        Ok(_) => Ok(()),
        Err(_) => panic!("get_file_url"),
    }
}
