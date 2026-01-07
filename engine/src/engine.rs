use std::{collections::HashMap, net::Shutdown};
use bigdecimal::BigDecimal;
use common::{BalanceEvent, CreateOrderArgs, EngineIx, InsertTradeArgs, Order, OrderEvent, Orderbook, Side, Status, TradeEvent, UserBalance};
use db::{create_order, create_user_balance, get_all_user_balance, get_open_orders};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::{Receiver, Sender};
use uuid::Uuid;

use crate::{OrderBookTrait, UserBalanceTrait};


pub struct Engine {
    orderbook: Orderbook,
    balances: HashMap<Uuid, UserBalance>,
    balance_tx: Sender<BalanceEvent>,
    trade_tx: Sender<TradeEvent>,
    order_tx: Sender<OrderEvent>,
    pool: Pool<Postgres>,
    engine_rx: Receiver<EngineIx>
}

impl Engine {
    pub fn default(balance_tx: Sender<BalanceEvent>, trade_tx: Sender<TradeEvent>,
        order_tx: Sender<OrderEvent>, pool: Pool<Postgres>, engine_rx: Receiver<EngineIx>) -> Self {
        Self { 
            orderbook: Orderbook::default(), 
            balances: HashMap::new(),
            balance_tx: balance_tx,
            trade_tx: trade_tx,
            order_tx: order_tx,
            pool: pool,
            engine_rx: engine_rx
        }
    }

    pub fn run(&mut self) {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build().unwrap();


        rt.block_on(async move {
            if let Err(e) = self.init_engine().await {
                eprintln!("Error Occurred, Shutting Down: {}", e.to_string());
                return;
            }

            loop {
                if let Some(cmd) = self.engine_rx.recv().await {
                    match cmd {
                        EngineIx::CreateLimitOrder(args) => {
                            self.execute_limit_order(args).await;
                        }
                        EngineIx::CreateMarketOrder (args) => {
                            self.execute_market_order(args).await;
                        }
                        EngineIx::CancelOrder { key } => {

                        },
                        EngineIx::Shutdown => {
                            self.balance_tx.send(BalanceEvent::Shutdown).await.unwrap();
                            self.trade_tx.send(TradeEvent::Shutdown).await.unwrap();
                            self.order_tx.send(OrderEvent::Shutdown).await.unwrap();

                            println!("Shutting Down Engine");
                            break;
                        }
                    }
                }
            }
        });
        
    }

    async fn init_engine(&mut self) -> anyhow::Result<()> {
        //load db orderbook
        let orders = get_open_orders(&self.pool).await?;

        //load db user balances
        let balances = get_all_user_balance(&self.pool).await?;

        //construct in memory orderbook and user balances
        self.orderbook = Orderbook::init_orderbook(orders)?;
        
        self.balances = UserBalance::init_user_balances(balances)?;

        Ok(())
    }

    pub async fn execute_limit_order(&mut self, args: CreateOrderArgs) {
        if self.balances.get(&args.user_id).is_none() {
            let user_balance = create_user_balance(&self.pool, args.user_id).await.unwrap();
            self.balances.insert(args.user_id, user_balance);
        }

        {
            let user_balance = self.balances.get_mut(&args.user_id).unwrap();
            
            //lock funds
            user_balance.lock_funds(&args).unwrap();
        };
        

        //determine maker_book and taker_book
        let (maker_book, _taker_book) = self.orderbook.determine_maker_taker_book(args.side);

        let mut quote_qty_remaining = args.base_qty.clone();

        //create user's order in db first
        let mut user_order = create_order(&self.pool, &args).await.unwrap();

        //for side = ask => maker_book = self.bids, so iterate in reverse direction for that
        let maker_book_iter: Box<dyn Iterator<Item = (&BigDecimal, &mut Vec<Order>)>> = match args.side {
            Side::Bid => Box::new(maker_book.iter_mut()),
            Side::Ask => Box::new(maker_book.iter_mut().rev())
        };


        for (price, orders) in maker_book_iter {
            if quote_qty_remaining.eq(&BigDecimal::from(0)) {
                break;
            }

            if args.side == Side::Bid {
                let user_balance = self.balances.get(&args.user_id).unwrap();
                if user_balance.locked_quote_qty < *price {
                    break;
                }  
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

            for (_index, order) in orders.iter_mut().enumerate() {
                
                let qty_left = &order.quantity - &order.filled_quantity;

                let mut trade_qty = qty_left.clone().min(quote_qty_remaining.clone());
                
                if quote_qty_remaining.eq(&BigDecimal::from(0)) {
                    break;
                }

                if args.side == Side::Bid {
                    let user_balance = self.balances.get(&args.user_id).unwrap();

                    if user_balance.locked_quote_qty < order.price {
                        break;
                    }

                    let quote_qty_to_pay = &order.price * &trade_qty; 
                    if quote_qty_to_pay > user_balance.locked_quote_qty {
                        trade_qty = &user_balance.locked_quote_qty / &order.price;
                    }
                    trade_qty = trade_qty.with_scale_round(0, bigdecimal::RoundingMode::Floor);
                }
                

                quote_qty_remaining -= &trade_qty;
                order.filled_quantity += &trade_qty;
                user_order.filled_quantity += &trade_qty;   

                //update maker balance and emit balance event for maker
                {
                    let maker_balance = self.balances.get_mut(&order.user_id).unwrap();
                    maker_balance.update_balance(order.side, &order.price, &trade_qty).unwrap();
                    self.balance_tx.send(BalanceEvent::UpdateBalance(maker_balance.clone())).await.unwrap();
                }

                //update user's balance
                {
                    let user_balance = self.balances.get_mut(&args.user_id).unwrap();
                    user_balance.update_balance(args.side, &order.price, &trade_qty).unwrap();
                }  

            
                /////emit events for maker
                //ws

                //trade event
                let (buy_order_id, sell_order_id) = Engine::determine_order_ids_for_trade_event(args.side, user_order.id, order.id).unwrap();
                
                self.trade_tx.send(TradeEvent::InsertTrade(InsertTradeArgs {
                    buy_order_id: buy_order_id,
                    sell_order_id: sell_order_id,
                    price: order.price.clone(),
                    quantity: trade_qty.clone()
                }))
                .await.unwrap();

                //close maker_order if filled qty == qty
                if order.filled_quantity.eq(&order.quantity) {
                    order.status = Status::Close;
                }

                //order event
                self.order_tx.send(OrderEvent::UpdateOrder(order.clone())).await.unwrap();

            }

            //remove all in-memory orders which are completely filled
            orders.retain(|order| order.filled_quantity < order.quantity);
        }

        //if quote_qty_remaining > 0 add user order in taker book
        if quote_qty_remaining > BigDecimal::from(0) {
            self.orderbook.add_order(user_order.clone()).unwrap();
        } else {
            user_order.status = Status::Close;
        }
    
        ////emit event for user
        //ws

        //balance event
        let user_balance = self.balances.get_mut(&args.user_id).unwrap();
        self.balance_tx.send(BalanceEvent::UpdateBalance(user_balance.clone())).await.unwrap();
        
        //order event
        self.order_tx.send(OrderEvent::UpdateOrder(user_order.clone())).await.unwrap();

        println!("Asks = {:?}", self.orderbook.asks);
        println!("Bids = {:?}", self.orderbook.bids);
        println!("Balances = {:?}", self.balances);
    }


    pub async fn execute_market_order(&mut self, args: CreateOrderArgs) {
        if self.balances.get(&args.user_id).is_none() {
            let user_balance = create_user_balance(&self.pool, args.user_id).await.unwrap();
            self.balances.insert(args.user_id, user_balance);
        }

        {
            let user_balance = self.balances.get_mut(&args.user_id).unwrap();
            
            //lock funds
            user_balance.lock_funds(&args).unwrap();
        };
        

        //determine maker_book and taker_book
        let (maker_book, _taker_book) = self.orderbook.determine_maker_taker_book(args.side);

        //for side = Bid this will be 0 by default
        let mut quote_qty_remaining = args.base_qty.clone();

        //create user's order in db first
        let mut user_order = create_order(&self.pool, &args).await.unwrap();

        //for side = ask => maker_book = self.bids, so iterate in reverse direction for that
        let maker_book_iter: Box<dyn Iterator<Item = (&BigDecimal, &mut Vec<Order>)>> = match args.side {
            Side::Bid => Box::new(maker_book.iter_mut()),
            Side::Ask => Box::new(maker_book.iter_mut().rev())
        };
        
        for (price, orders) in maker_book_iter {
            
            if args.side == Side::Ask {
                if quote_qty_remaining.eq(&BigDecimal::from(0)) {
                    break;
                }
            } else {
                let user_balance = self.balances.get(&args.user_id).unwrap();
                if user_balance.locked_quote_qty < *price {
                    break;
                }  
            }
        

            for (_index, order) in orders.iter_mut().enumerate() {
                
                let mut trade_qty = &order.quantity - &order.filled_quantity;


                if args.side == Side::Ask {
                    if quote_qty_remaining.eq(&BigDecimal::from(0)) {
                        break;
                    }
                } else {
                    let user_balance = self.balances.get(&args.user_id).unwrap();

                    if user_balance.locked_quote_qty < order.price {
                        break;
                    }

                    let quote_qty_to_pay = &order.price * &trade_qty; 
                    if quote_qty_to_pay > user_balance.locked_quote_qty {
                        trade_qty = &user_balance.locked_quote_qty / &order.price;
                    }
                    trade_qty = trade_qty.with_scale_round(0, bigdecimal::RoundingMode::Floor);
                }


                if args.side == Side::Ask {
                    quote_qty_remaining -= &trade_qty;
                }
                order.filled_quantity += &trade_qty;
                user_order.filled_quantity += &trade_qty;   

                //update maker balance and emit balance event for maker
                {
                    let maker_balance = self.balances.get_mut(&order.user_id).unwrap();
                    maker_balance.update_balance(order.side, &order.price, &trade_qty).unwrap();
                    self.balance_tx.send(BalanceEvent::UpdateBalance(maker_balance.clone())).await.unwrap();
                }

                //update user's balance
                {
                    let user_balance = self.balances.get_mut(&args.user_id).unwrap();
                    user_balance.update_balance(args.side, &order.price, &trade_qty).unwrap();
                }  

            
                /////emit events for maker
                //ws

                //trade event
                let (buy_order_id, sell_order_id) = Engine::determine_order_ids_for_trade_event(args.side, user_order.id, order.id).unwrap();
                
                self.trade_tx.send(TradeEvent::InsertTrade(InsertTradeArgs {
                    buy_order_id: buy_order_id,
                    sell_order_id: sell_order_id,
                    price: order.price.clone(),
                    quantity: trade_qty.clone()
                }))
                .await.unwrap();

                //close maker_order if filled qty == qty
                if order.filled_quantity.eq(&order.quantity) {
                    order.status = Status::Close;
                }

                //order event
                self.order_tx.send(OrderEvent::UpdateOrder(order.clone())).await.unwrap();

            }

            //remove all in-memory orders which are completely filled
            orders.retain(|order| order.filled_quantity < order.quantity);
        }

        //close this user order
        user_order.status = Status::Close;
        
        ////emit event
        //ws

        //balance event
        let user_balance = self.balances.get_mut(&args.user_id).unwrap();
        self.balance_tx.send(BalanceEvent::UpdateBalance(user_balance.clone())).await.unwrap();
        
        //order event
        self.order_tx.send(OrderEvent::UpdateOrder(user_order.clone())).await.unwrap();

        println!("Asks = {:?}", self.orderbook.asks);
        println!("Bids = {:?}", self.orderbook.bids);
        println!("Balances = {:?}", self.balances);
    }


    pub fn determine_order_ids_for_trade_event(side: Side, user_order_id: Uuid, 
            matching_order_id: Uuid) -> anyhow::Result<(Uuid, Uuid)> {
        Ok(match side {
            Side::Bid => {
                (user_order_id, matching_order_id)
            }
            Side::Ask => {
                (matching_order_id, user_order_id)
            }
        })
    }

    pub fn is_balance_exhausted(&self, user_id: &Uuid, price: &BigDecimal) -> bool {
        let user_balance = self.balances.get(user_id).unwrap();
        return *price > user_balance.locked_quote_qty;
    }

    


}