use std::env;

use redis::{Client, aio::ConnectionManager};

#[derive(Clone)]
pub struct RedisConnection {
    pub client: Client,
    pub connection_manger: ConnectionManager
}

impl RedisConnection {
    pub async fn new() -> anyhow::Result<Self> {
        let redis_url = env::var("REDIS_URL")?;
        let client = redis::Client::open(redis_url)?;
        let connection_manager = ConnectionManager::new(client.clone()).await?;
        Ok(Self {
            client: client,
            connection_manger: connection_manager
        })
    }
}