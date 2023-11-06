use std::time::Duration;

use mobc::{Connection, Pool};
use mobc_redis::{redis, RedisConnectionManager};
use once_cell::sync::OnceCell;
use redis::AsyncCommands;

use crate::config::Config;

pub type MobcPool = Pool<RedisConnectionManager>;
pub type MobcCon = Connection<RedisConnectionManager>;

pub static POOL: OnceCell<MobcPool> = OnceCell::new();

pub async fn connect_database() -> Result<(), ()> {
    let database_config = &Config::get().database;
    let client = redis::Client::open(&*database_config.address).map_err(|_| ())?;
    let manager = RedisConnectionManager::new(client);

    let pool = mobc::Pool::builder()
        .get_timeout(Some(Duration::from_secs(
            database_config.pool_timeout as u64,
        )))
        .max_open(database_config.pool_max_open as u64)
        .max_idle(database_config.pool_max_idle as u64)
        .max_lifetime(Some(Duration::from_secs(
            database_config.pool_expire as u64,
        )))
        .build(manager);

    pool.get().await.unwrap().set::<_, _, ()>("test", 1);

    POOL.set(pool).ok();

    return Ok(());
}

pub async fn get_con() -> Result<MobcCon, ()> {
    return POOL.get().unwrap().get().await.map_err(|_| ());
}
