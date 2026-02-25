use common::TradeEvent;
use db::create_trade;
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;

pub struct TradeWorker {
    pool: Pool<Postgres>,
    trade_rx: Receiver<TradeEvent>
}

impl TradeWorker {
    pub fn default(pool: Pool<Postgres>, trade_rx: Receiver<TradeEvent>) -> Self {
        Self { 
            pool: pool, 
            trade_rx: trade_rx
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
                        let _ = engine_flush_ctx.send(common::AcknowledgementEvent::Flush).await;
                    }
                }
            }
        }
    }
}