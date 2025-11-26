use std::collections::{HashMap};
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;
use uuid::Uuid;

use crate::service::{Order, Orderbook, Side, UserBalance, get_all_user_balance, get_orders};

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
                    match cmd {
                        EngineIx::CreateLimitOrder(args) => {
                            
                            //user existence check
                            if self.balances.get(&args.user_id).is_none() {
                                eprintln!("User does not exist");
                                continue;
                            }

                            let user_balance = self.balances.get_mut(&args.user_id).unwrap();
                            let deposit_amount;

                            //lock funds
                            match args.side {
                                Side::Bid => {
                                    let free_quote_qty = user_balance.free_quote_qty.clone();
                                    let quote_qty_to_lock = free_quote_qty.clone().min(args.quote_qty.clone());   
                                    deposit_amount = &args.quote_qty - quote_qty_to_lock;
                                    user_balance.credit_locked_quote_qty(&deposit_amount).unwrap();
                                    user_balance.lock_free_quote_qty(&free_quote_qty).unwrap();
                                }
                                Side::Ask => {
                                    let free_base_qty = user_balance.free_base_qty.clone();
                                    let base_qty_to_lock = free_base_qty.clone().min(args.base_qty.clone());   
                                    deposit_amount = &args.base_qty - base_qty_to_lock;
                                    user_balance.credit_locked_base_qty(&deposit_amount).unwrap();
                                    user_balance.lock_free_base_qty(&free_base_qty).unwrap();                      
                                }
                            }

                            //determine maker_book taker_book
                            let (maker_book, taker_book) = match args.side {
                                Side::Bid => {
                                    (&mut self.orderbook.asks, &mut self.orderbook.bids)
                                }
                                Side::Ask => {
                                    (&mut self.orderbook.bids, &mut self.orderbook.asks)
                                }
                            };

                            let mut quote_qty_remaining = args.base_qty;
                            let maker_book_len = maker_book.len();

                            for (price, orders) in maker_book.iter_mut() {
                                if quote_qty_remaining.eq(&BigDecimal::from(0)) {
                                    break;
                                }

                                let crossed = match args.side {
                                    Side::Bid => {
                                        *price > args.limit_price
                                    }
                                    Side::Ask => {
                                        *price < args.limit_price
                                    }
                                };

                                if crossed {
                                    break;
                                }

                                for order in orders {
                                    if quote_qty_remaining.eq(&BigDecimal::from(0)) {
                                        break;
                                    }

                                    let trade_qty = order.price.clone().min(args.limit_price.clone());

                                    quote_qty_remaining -= &trade_qty;
                                    order.filled_quantity += &trade_qty;

                                    //remove maker_order if filled qty == qty

                                    //update maker balance

                                    //update taker balance

                                    ////emit events
                                    //ws
                                    //balance
                                    //trade

                                }
                            }

                            //if quote_qty_remaining != 0 add order in taker book
                            //emit event
                            //ws
                            
                        }
                        EngineIx::CreateMarketOrder { key } => {
                            
                        }
                        EngineIx::CancelOrder { key } => {

                        }
                    }
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

pub enum EngineIx {
    CreateLimitOrder(CreateLimitOrderArgs),
    CreateMarketOrder {
        key: String,
    },
    CancelOrder {
        key: String
    }
}

#[derive(Serialize, Deserialize)]
pub struct CreateLimitOrderArgs {
    pub side: Side,
    pub user_id: Uuid,
    pub limit_price: BigDecimal,
    pub base_qty: BigDecimal,
    pub quote_qty: BigDecimal
}
