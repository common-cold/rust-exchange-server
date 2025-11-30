use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::db::create_trade;

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
                    }
                }
            }
        }
    }
}

pub enum TradeEvent {
    InsertTrade(InsertTradeArgs)
}

#[derive(Serialize, Deserialize)]
pub struct InsertTradeArgs {
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: BigDecimal,
    pub quantity: BigDecimal
}