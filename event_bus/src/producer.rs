use common::{InsertTradeArgs, Order, UserBalance};
use redis::{AsyncTypedCommands, aio::ConnectionManager};

use crate::{BALANCE_DLQ_STREAM_NAME, BALANCE_STREAM_NAME, ORDER_DLQ_STREAM_NAME, ORDER_STREAM_NAME, TRADE_DLQ_STREAM_NAME, TRADE_STREAM_NAME};

#[derive(Clone)]
pub struct EventBusProducer {
    pub redis_conn: ConnectionManager
}

impl EventBusProducer {
    pub fn new(redis_conn: ConnectionManager) -> Self {
        Self { 
            redis_conn: redis_conn 
        }
    }
    pub async fn publish_balance_event(&self, balance_event: UserBalance) -> anyhow::Result<()> {
        let mut conn = self.redis_conn.clone();
        let string_event = serde_json::to_string(&balance_event)?;
        let _id = conn.xadd(BALANCE_STREAM_NAME, "*", &[("value", &string_event)]).await?.unwrap();
        Ok(())
    }

    pub async fn publish_order_event(&self, order_event: Order) -> anyhow::Result<()> {
        let mut conn = self.redis_conn.clone();
        let string_event = serde_json::to_string(&order_event)?;
        let _id = conn.xadd(ORDER_STREAM_NAME, "*", &[("value", &string_event)]).await?.unwrap();
        Ok(())
    }

    pub async fn publish_trade_event(&self, trade_event: InsertTradeArgs) -> anyhow::Result<()> {
        let mut conn = self.redis_conn.clone();
        let string_event = serde_json::to_string(&trade_event)?;
        let _id = conn.xadd(TRADE_STREAM_NAME, "*", &[("value", &string_event)]).await?.unwrap();
        Ok(())
    }

    pub async fn publish_balance_dlq_event(&self, balance_event: UserBalance) -> anyhow::Result<()> {
        let mut conn = self.redis_conn.clone();
        let string_event = serde_json::to_string(&balance_event)?;
        let _id = conn.xadd(BALANCE_DLQ_STREAM_NAME, "*", &[("value", &string_event)]).await?.unwrap();
        Ok(())
    }

    pub async fn publish_order_dlq_event(&self, order_event: Order) -> anyhow::Result<()> {
        let mut conn = self.redis_conn.clone();
        let string_event = serde_json::to_string(&order_event)?;
        let _id = conn.xadd(ORDER_DLQ_STREAM_NAME, "*", &[("value", &string_event)]).await?.unwrap();
        Ok(())
    }

    pub async fn publish_trade_dlq_event(&self, trade_event: InsertTradeArgs) -> anyhow::Result<()> {
        let mut conn = self.redis_conn.clone();
        let string_event = serde_json::to_string(&trade_event)?;
        let _id = conn.xadd(TRADE_DLQ_STREAM_NAME, "*", &[("value", &string_event)]).await?.unwrap();
        Ok(())
    }
}