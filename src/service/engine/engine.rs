use std::collections::{HashMap};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::{service::{Orderbook, UserBalance, get_all_user_balance, get_orders}};

pub enum EngineIx {
    CreateLimitOrder {
        key: String,
    },
    CreateMarketOrder {
        key: String,
    },
    CancelOrder {
        key: String
    }
}

pub struct Engine {
    orderbook: Orderbook,
    balances: HashMap<Uuid, UserBalance>
}

impl Engine {
    pub fn default() -> Self {
        Self { 
            orderbook: Orderbook::default(), 
            balances: HashMap::new() 
        }
    }

    pub fn run(&mut self, pool: Pool<Postgres>, rx: &mut Receiver<EngineIx>) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build().unwrap();


        rt.block_on(async move {
            if self.init_engine(&pool).await.is_err() {
                eprintln!("Error Occurred, Shutting Down");
                return;
            }

            loop {
                if let Some(cmd) = rx.recv().await {
                    
                }
            }
        });


        //construct in memory orderbook, user balances

        //start loop 
    }

    async fn init_engine(&mut self, pool: &Pool<Postgres>) -> anyhow::Result<()> {
        //load db orderbook
        let orders = get_orders(pool).await?;

        //load db user balances
        let balances = get_all_user_balance(pool).await?;

        //construct in memory orderbook, user balances
        self.orderbook = Orderbook::init_orderbook(orders)?;
        
        self.balances = UserBalance::init_user_balances(balances)?;

        Ok(())
    }


}
