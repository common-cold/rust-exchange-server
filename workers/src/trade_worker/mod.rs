use common::TradeEvent;
use db::create_trade;
use event_bus::get_trade_stream_length;
use redis::aio::ConnectionManager;
use redis_service::RedisConnection;
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;

pub struct TradeWorker {
    pool: Pool<Postgres>,
    trade_rx: Receiver<TradeEvent>,
    redis_conn: ConnectionManager
}

impl TradeWorker {
    pub fn default(pool: Pool<Postgres>, trade_rx: Receiver<TradeEvent>, redis: RedisConnection) -> Self {
        Self { 
            pool: pool, 
            trade_rx: trade_rx,
            redis_conn: redis.connection_manger
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(cmd) = self.trade_rx.recv().await {
                match cmd {
                    TradeEvent::InsertTrade(args) => {
                        create_trade(&self.pool, args).await.unwrap();
                    },
                    TradeEvent::Shutdown(engine_shutdown_ctx) => {
                        let _ = engine_shutdown_ctx.send(common::AcknowledgementEvent::Shutdown).await;
                        println!("Shutting Down Trade Worker");
                        break;
                    },
                    TradeEvent::Flush(engine_flush_ctx) => {
                        loop {
                            match get_trade_stream_length(&mut self.redis_conn).await {
                                Ok(size) => {
                                    if size == 0 {
                                        break;
                                    }
                                },
                                Err(e) => {
                                    println!("Trade worker stream length error: {}", e.to_string());
                                    continue;
                                }
                            }
                        }
                        let _ = engine_flush_ctx.send(common::AcknowledgementEvent::Flush).await;
                    }
                }
            }
        }
    }
}