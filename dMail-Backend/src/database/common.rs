use super::redis;

/// 连接数据库
pub async fn connect_database() {
    redis::connect_database()
        .await
        .expect("Redis 数据库连接失败");
}
