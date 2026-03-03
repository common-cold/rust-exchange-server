use common::BalanceEvent;
use db::update_user_balance;
use event_bus::get_balance_stream_length;
use redis::aio::ConnectionManager;
use redis_service::RedisConnection;
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;


pub struct BalanceWorker {
    pool: Pool<Postgres>,
    balance_rx: Receiver<BalanceEvent>,
    redis_conn: ConnectionManager
}

impl BalanceWorker {
    pub fn default(pool: Pool<Postgres>, balance_rx: Receiver<BalanceEvent>, redis: RedisConnection) -> Self {
        Self { 
            pool: pool, 
            balance_rx: balance_rx ,
            redis_conn: redis.connection_manger
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(cmd) = self.balance_rx.recv().await {
                match cmd {
                    BalanceEvent::UpdateBalance(args) => {
                        update_user_balance(&self.pool, args).await.unwrap()
                    },
                    BalanceEvent::Shutdown(engine_shutdown_tx) => {
                        let _ = engine_shutdown_tx.send(common::AcknowledgementEvent::Shutdown).await;
                        println!("Shutting Down Balance Worker");
                        break;
                    },
                    BalanceEvent::Flush(engine_flush_tx) => {
                        loop {
                            match get_balance_stream_length(&mut self.redis_conn).await {
                                Ok(size) => {
                                    if size == 0 {
                                        break;
                                    }
                                },
                                Err(e) => {
                                    println!("Balance worker stream length error: {}", e.to_string());
                                    continue;
                                }
                            }
                        }
                        let _ = engine_flush_tx.send(common::AcknowledgementEvent::Flush).await;
                    }
                }
            }
        }
    }
}
