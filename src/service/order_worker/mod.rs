use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;

use crate::{db::{update_order}, service::{Order}};


pub struct OrderWorker {
    pool: Pool<Postgres>,
    balance_rx: Receiver<OrderEvent>
}

impl OrderWorker {
    pub fn default(pool: Pool<Postgres>, balance_rx: Receiver<OrderEvent>) -> Self {
        Self { 
            pool: pool, 
            balance_rx: balance_rx 
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(cmd) = self.balance_rx.recv().await {
                match cmd {
                    OrderEvent::UpdateOrder(args) => {
                        update_order(&self.pool, args).await.unwrap()
                    }
                }
            }
        }
    }
}


pub enum OrderEvent {
    UpdateOrder(Order)
}