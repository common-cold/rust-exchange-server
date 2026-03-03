use redis::{AsyncTypedCommands, aio::ConnectionManager};

pub const BALANCE_STREAM_NAME: &str = "engine:balance";
pub const ORDER_STREAM_NAME: &str = "engine:order";
pub const TRADE_STREAM_NAME: &str = "engine:trade";

pub const BALANCE_DLQ_STREAM_NAME: &str = "engine:balance_dlq";
pub const ORDER_DLQ_STREAM_NAME: &str = "engine:order_dlq";
pub const TRADE_DLQ_STREAM_NAME: &str = "engine:trade_dlq";

pub async fn get_balance_stream_length(redis_conn: &mut ConnectionManager) -> anyhow::Result<usize> {
    let res = redis_conn.xlen(BALANCE_STREAM_NAME).await?;
    Ok(res)
}

pub async fn get_order_stream_length(redis_conn: &mut ConnectionManager) -> anyhow::Result<usize> {
    let res = redis_conn.xlen(ORDER_STREAM_NAME).await?;
    Ok(res)
}

pub async fn get_trade_stream_length(redis_conn: &mut ConnectionManager) -> anyhow::Result<usize> {
    let res = redis_conn.xlen(TRADE_STREAM_NAME).await?;
    Ok(res)
}