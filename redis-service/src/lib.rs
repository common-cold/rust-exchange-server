use std::env;

use redis::aio::ConnectionManager;


pub struct RedisConnection {
    pub connection_manger: ConnectionManager
}

impl RedisConnection {
    pub async fn new() -> anyhow::Result<Self> {
        let redis_url = env::var("REDIS_URL")?;
        let client = redis::Client::open(redis_url)?;
        let connection_manager = ConnectionManager::new(client).await?;
        Ok(Self {
            connection_manger: connection_manager
        })
    }
}