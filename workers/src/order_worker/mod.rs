use common::OrderEvent;
use db::update_order;
use event_bus::get_order_stream_length;
use redis::aio::ConnectionManager;
use redis_service::RedisConnection;
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;


pub struct OrderWorker {
    pool: Pool<Postgres>,
    order_rx: Receiver<OrderEvent>,
    redis_conn: ConnectionManager
}

impl OrderWorker {
    pub fn default(pool: Pool<Postgres>, order_rx: Receiver<OrderEvent>, redis: RedisConnection) -> Self {
        Self { 
            pool: pool, 
            order_rx: order_rx ,
            redis_conn: redis.connection_manger
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(cmd) = self.order_rx.recv().await {
                match cmd {
                    OrderEvent::UpdateOrder(args) => {
                        update_order(&self.pool, args).await.unwrap()
                    },
                    OrderEvent::Shutdown(engine_shutdown_ctx) => {
                        let _ = engine_shutdown_ctx.send(common::AcknowledgementEvent::Shutdown).await;
                        println!("Shutting Down Order Worker");
                        break;
                    },
                    OrderEvent::Flush(engine_flush_ctx) => {
                        loop {
                            match get_order_stream_length(&mut self.redis_conn).await {
                                Ok(size) => {
                                    if size == 0 {
                                        break;
                                    }
                                },
                                Err(e) => {
                                    println!("Order worker stream length error: {}", e.to_string());
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