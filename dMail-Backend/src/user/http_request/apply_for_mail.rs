/*!
 负责向用户发送验证码
*/

use actix_web::{post, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::server::email::send_random_code;

/** `ApplyForEmail` 申请验证码时所用的数据类型
*/
#[derive(Serialize, Deserialize, Debug)]
pub struct ApplyForEmail {
    pub email: String,
}

/** `apply_for_email_code` 申请验证码时调用的函数
*/
#[post("/email/code")]
pub async fn apply_for_email_code(
    json: web::Json<ApplyForEmail>,
    _request: HttpRequest,
) -> impl Responder {
    // TODO : 请求速率限制

    if let Err(err) = send_random_code(None, &json.email).await {
        // TODO : HTTP状态码规范
        return HttpResponse::Ok().json(err);
    }
    return HttpResponse::Ok().into();
}
