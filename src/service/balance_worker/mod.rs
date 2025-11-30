use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;

use crate::{db::update_user_balance, service::UserBalance};


pub struct BalanceWorker {
    pool: Pool<Postgres>,
    balance_rx: Receiver<BalanceEvent>
}

impl BalanceWorker {
    pub fn default(pool: Pool<Postgres>, balance_rx: Receiver<BalanceEvent>) -> Self {
        Self { 
            pool: pool, 
            balance_rx: balance_rx 
        }
    }

    pub async fn run(&mut self) {
        loop {
            if let Some(cmd) = self.balance_rx.recv().await {
                match cmd {
                    BalanceEvent::UpdateBalance(args) => {
                        update_user_balance(&self.pool, args).await.unwrap()
                    }
                }
            }
        }
    }
}


pub enum BalanceEvent {
    UpdateBalance(UserBalance)
}