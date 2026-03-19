use std::{collections::HashMap, str::FromStr};
use bigdecimal::BigDecimal;
use common::{AcknowledgementEvent, BalanceEvent, CreateOrderArgs, EngineIx, InsertTradeArgs, OnRampArgs, Order, OrderEvent, Orderbook, Side, Status, TradeEvent, UserBalance};
use db::{create_order, create_user_balance, get_all_user_balance, get_open_orders, get_order_by_order_id};
use event_bus::producer::EventBusProducer;
use redis_service::RedisConnection;
use sqlx::{Pool, Postgres};
use tokio::{sync::mpsc::{self, Receiver, Sender}};
use uuid::Uuid;

use crate::{OrderBookTrait, UserBalanceTrait};


pub struct Engine {
    orderbook: Orderbook,
    balances: HashMap<Uuid, UserBalance>,
    balance_tx: Sender<BalanceEvent>,
    trade_tx: Sender<TradeEvent>,
    order_tx: Sender<OrderEvent>,
    pool: Pool<Postgres>,
    engine_rx: Receiver<EngineIx>,
    event_producer: EventBusProducer
}

impl Engine {
    pub fn default(balance_tx: Sender<BalanceEvent>, trade_tx: Sender<TradeEvent>,
        order_tx: Sender<OrderEvent>, pool: Pool<Postgres>, engine_rx: Receiver<EngineIx>, redis_conn: RedisConnection) -> Self {
        let event_producer = EventBusProducer::new(redis_conn.connection_manger);
        Self { 
            orderbook: Orderbook::default(), 
            balances: HashMap::new(),
            balance_tx: balance_tx,
            trade_tx: trade_tx,
            order_tx: order_tx,
            pool: pool,
            engine_rx: engine_rx,
            event_producer: event_producer
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
                        EngineIx::CancelOrder { order_id } => {
                            self.cancel_order(order_id).await;
                        },
                        EngineIx::OnRamp(args) => {
                            self.onramp(args).await;
                        }
                        EngineIx::State(tx) => {
                            let _ = tx.send(AcknowledgementEvent::State((self.orderbook.clone(), self.balances.clone()))).await;
                        },
                        EngineIx::Shutdown(tx) => {
                            let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<AcknowledgementEvent>(3);
                            
                            self.balance_tx.send(BalanceEvent::Shutdown(shutdown_tx.clone())).await.unwrap();
                            self.trade_tx.send(TradeEvent::Shutdown(shutdown_tx.clone())).await.unwrap();
                            self.order_tx.send(OrderEvent::Shutdown(shutdown_tx.clone())).await.unwrap();

                            let mut remaining = 3;

                            while remaining > 0 {
                                if let Some(command) = shutdown_rx.recv().await {
                                    match command {
                                        AcknowledgementEvent::Shutdown => {
                                            remaining = remaining - 1;
                                        },
                                        _ => {}
                                    }
                                }
                            };

                            let _ = tx.send(AcknowledgementEvent::Shutdown).await;

                            println!("Shutting Down Engine");
                            break;
                        },
                        EngineIx::Flush(tx) => {
                            let (flush_tx, mut flush_rx) = mpsc::channel::<AcknowledgementEvent>(3);
                            
                            self.balance_tx.send(BalanceEvent::Flush(flush_tx.clone())).await.unwrap();
                            self.trade_tx.send(TradeEvent::Flush(flush_tx.clone())).await.unwrap();
                            self.order_tx.send(OrderEvent::Flush(flush_tx.clone())).await.unwrap();

                            let mut remaining = 3;

                            while remaining > 0 {
                                if let Some(command) = flush_rx.recv().await {
                                    match command {
                                        AcknowledgementEvent::Flush => {
                                            remaining = remaining - 1;                         
                                        },
                                        _ => {}
                                    }
                                }
                            };

                            let _ = tx.send(AcknowledgementEvent::Flush).await;
                        },
                        EngineIx::Debug => {
                            println!("Orderbook:");
                            println!("{:#?}", self.orderbook);
                            println!("Balances:");
                            println!("{:#?}", self.balances);
                            println!("-------------------------------------------------");
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
            let user_balance = create_user_balance(&self.pool, args.user_id, BigDecimal::from(0)).await.unwrap();
            self.balances.insert(args.user_id, user_balance);
        }

        {
            let user_balance = self.balances.get_mut(&args.user_id).unwrap();
            
            //lock funds
            user_balance.lock_funds(&args).unwrap();
        };
        

        //determine maker_book and taker_book
        let (maker_book, _taker_book) = self.orderbook.determine_maker_taker_book(args.side);

        let mut base_qty_remaining = args.base_qty.clone();

        //create user's order in db first synchronously
        let mut user_order = create_order(&self.pool, &args).await.unwrap();

        //for side = ask => maker_book = self.bids, so iterate in reverse direction for that
        let maker_book_iter: Box<dyn Iterator<Item = (&BigDecimal, &mut Vec<Order>)>> = match args.side {
            Side::Bid => Box::new(maker_book.iter_mut()),
            Side::Ask => Box::new(maker_book.iter_mut().rev())
        };

        for (price, orders) in maker_book_iter {
            if base_qty_remaining.eq(&BigDecimal::from(0)) {
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
                    price > args.limit_price.as_ref().unwrap()
                }
                Side::Ask => {
                    price < args.limit_price.as_ref().unwrap()
                }
            };

            if crossed {
                break;
            }

            for (_index, order) in orders.iter_mut().enumerate() {
                if base_qty_remaining.eq(&BigDecimal::from(0)) {
                    break;
                }

                let order_price = order.price.as_ref().unwrap().clone();

                let qty_left = &order.quantity - &order.filled_quantity;

                let mut trade_qty = qty_left.clone().min(base_qty_remaining.clone());
                
                
                if args.side == Side::Bid {
                    let user_balance = self.balances.get(&args.user_id).unwrap();
                    if user_balance.locked_quote_qty < order_price {
                        break;
                    }

                    let mut quote_qty_to_pay = &order_price * &trade_qty; 
                    quote_qty_to_pay = quote_qty_to_pay / BigDecimal::from_str("1000000000").unwrap();
                    if quote_qty_to_pay > user_balance.locked_quote_qty {
                        trade_qty = &user_balance.locked_quote_qty / &order_price;
                    }
                    trade_qty = trade_qty.with_scale_round(0, bigdecimal::RoundingMode::Floor);
                }

                base_qty_remaining -= &trade_qty;
                order.filled_quantity += &trade_qty;
                user_order.filled_quantity += &trade_qty;   

                //update maker balance and emit balance event for maker
                {
                    let maker_balance = self.balances.get_mut(&order.user_id).unwrap();
                    maker_balance.update_balance(order.side, &order_price, &trade_qty).unwrap();
                    self.event_producer.publish_balance_event(maker_balance.clone()).await.unwrap();
                }

                //update user's balance
                {
                    let user_balance = self.balances.get_mut(&args.user_id).unwrap();
                    user_balance.update_balance(args.side, &order_price, &trade_qty).unwrap();
                }  

            
                /////emit events for maker
                //trade event
                let (buy_order_id, sell_order_id) = Engine::determine_order_ids_for_trade_event(args.side, user_order.id, order.id).unwrap();
                
                let inser_trade_args = InsertTradeArgs {
                    buy_order_id: buy_order_id,
                    sell_order_id: sell_order_id,
                    price: order_price.clone(),
                    quantity: trade_qty.clone()
                };
                self.event_producer.publish_trade_event(inser_trade_args).await.unwrap();

                if order.filled_quantity.eq(&order.quantity) {
                    order.status = Status::Close;
                }

                //order event
                self.event_producer.publish_order_event(order.clone()).await.unwrap();

            }

            //remove all in-memory orders which are completely filled
            orders.retain(|order| order.filled_quantity < order.quantity);
        }

        //if quote_qty_remaining > 0 add user order in taker book
        if base_qty_remaining > BigDecimal::from(0) {
            self.orderbook.add_order(user_order.clone()).unwrap();
        } else {
            user_order.status = Status::Close;
        }
    
        ////emit event for user
        //balance event
        let user_balance = self.balances.get_mut(&args.user_id).unwrap();
        self.event_producer.publish_balance_event(user_balance.clone()).await.unwrap();
        
        //order event
        self.event_producer.publish_order_event(user_order.clone()).await.unwrap();
    }

    pub async fn execute_market_order(&mut self, args: CreateOrderArgs) {
        if self.balances.get(&args.user_id).is_none() {
            let user_balance = create_user_balance(&self.pool, args.user_id, BigDecimal::from(0)).await.unwrap();
            self.balances.insert(args.user_id, user_balance);
        }

        {
            let user_balance = self.balances.get_mut(&args.user_id).unwrap();
            
            //lock funds
            user_balance.lock_funds(&args).unwrap();
        };
        

        //determine maker_book and taker_book
        let (maker_book, _taker_book) = self.orderbook.determine_maker_taker_book(args.side);


        let mut base_qty_remaining = args.base_qty.clone();

        //create user's order in db first, synchronously
        let mut user_order = create_order(&self.pool, &args).await.unwrap();

        //for side = ask => maker_book = self.bids, so iterate in reverse direction for that
        let maker_book_iter: Box<dyn Iterator<Item = (&BigDecimal, &mut Vec<Order>)>> = match args.side {
            Side::Bid => Box::new(maker_book.iter_mut()),
            Side::Ask => Box::new(maker_book.iter_mut().rev())
        };
        
        for (price, orders) in maker_book_iter {
            if base_qty_remaining.eq(&BigDecimal::from(0)) {
                break;
            }
            
            if args.side == Side::Bid {
                let user_balance = self.balances.get(&args.user_id).unwrap();
                if user_balance.locked_quote_qty < *price {
                    break;
                }  
            }
            
            for (_index, order) in orders.iter_mut().enumerate() {
                if base_qty_remaining.eq(&BigDecimal::from(0)) {
                    break;
                }

                let order_price = order.price.as_ref().unwrap().clone();
                
                let qty_left = &order.quantity - &order.filled_quantity;

                let mut trade_qty = qty_left.clone().min(base_qty_remaining.clone());


                if args.side == Side::Bid {
                    let user_balance = self.balances.get(&args.user_id).unwrap();
                    if user_balance.locked_quote_qty < order_price {
                        break;
                    }

                    let mut quote_qty_to_pay = &order_price * &trade_qty; 
                    quote_qty_to_pay = quote_qty_to_pay / BigDecimal::from_str("1000000000").unwrap();
                    if quote_qty_to_pay > user_balance.locked_quote_qty {
                        trade_qty = &user_balance.locked_quote_qty / &order_price;
                    }
                    trade_qty = trade_qty.with_scale_round(0, bigdecimal::RoundingMode::Floor);
                }


                base_qty_remaining -= &trade_qty;
                order.filled_quantity += &trade_qty;
                user_order.filled_quantity += &trade_qty;   

                //update maker balance and emit balance event for maker
                {
                    let maker_balance = self.balances.get_mut(&order.user_id).unwrap();
                    maker_balance.update_balance(order.side, &order_price, &trade_qty).unwrap();
                    self.event_producer.publish_balance_event(maker_balance.clone()).await.unwrap();
                }

                //update user's balance
                {
                    let user_balance = self.balances.get_mut(&args.user_id).unwrap();
                    user_balance.update_balance(args.side, &order_price, &trade_qty).unwrap();
                }  

            
                /////emit events for maker
                //trade event
                let (buy_order_id, sell_order_id) = Engine::determine_order_ids_for_trade_event(args.side, user_order.id, order.id).unwrap();
                
                let trade_args = InsertTradeArgs {
                    buy_order_id: buy_order_id,
                    sell_order_id: sell_order_id,
                    price: order_price.clone(),
                    quantity: trade_qty.clone()
                };

                self.event_producer.publish_trade_event(trade_args).await.unwrap();

                //close maker_order if filled qty == qty
                if order.filled_quantity.eq(&order.quantity) {
                    order.status = Status::Close;
                }

                //order event
                self.event_producer.publish_order_event(order.clone()).await.unwrap();


            }

            //remove all in-memory orders which are completely filled
            orders.retain(|order| order.filled_quantity < order.quantity);
        }

        //close this user order
        user_order.status = Status::Close;
        
        ////emit event
        //balance event
        let user_balance = self.balances.get_mut(&args.user_id).unwrap();
        self.event_producer.publish_balance_event(user_balance.clone()).await.unwrap();
        
        //order event
        self.event_producer.publish_order_event(user_order.clone()).await.unwrap();
    }

    pub async fn cancel_order(&mut self, order_id: Uuid) {
        let order_option = get_order_by_order_id(&self.pool, order_id).await.unwrap();
        if order_option.is_none() {
            return;
        }
        let mut order = order_option.unwrap();

        if order.status != Status::Open {
            return;
        }

        //unlock balance
        let user_balance_option = self.balances.get_mut(&order.user_id);
        if user_balance_option.is_none() {
            return;
        }
        let user_balance = user_balance_option.unwrap();
        if order.side == Side::Bid {
            let mut amount = (&order.quantity - &order.filled_quantity) * order.price.as_ref().unwrap();
            amount = amount / BigDecimal::from_str("1000000000").unwrap();
            user_balance.free_quote_qty += &amount;
            user_balance.locked_quote_qty -= &amount;
        } else {
            let amount = &order.quantity - &order.filled_quantity;
            user_balance.free_base_qty += &amount;
            user_balance.locked_base_qty -= &amount;
        }

        //update balance in db
        self.event_producer.publish_balance_event(user_balance.clone()).await.unwrap();

        //remove order from in memory orderbook
        if order.side == Side::Bid {
            let bids_option = self.orderbook.bids.get_mut(order.price.as_ref().unwrap());
            if bids_option.is_some() {
                let bids = bids_option.unwrap();
                bids.retain(|o| o.id != order.id);
                if bids.len() == 0 {
                    self.orderbook.bids.remove(order.price.as_ref().unwrap());
                }
            }
        } else {
            let asks_option = self.orderbook.asks.get_mut(order.price.as_ref().unwrap());
            if asks_option.is_some() {
                let asks = asks_option.unwrap();
                asks.retain(|o| o.id != order.id);
                if asks.len() == 0 {
                    self.orderbook.asks.remove(order.price.as_ref().unwrap());
                }
            }
        }

        //update order in db
        order.status = Status::Cancelled;
        self.event_producer.publish_order_event(order.clone()).await.unwrap();
    }

    pub async fn onramp(&mut self, args: OnRampArgs) {
        if args.usdc_conversion_rate.is_none() {
            return;
        }

        let mut usdc_amount_in_base_units = &args.amount / args.usdc_conversion_rate.as_ref().unwrap();
            usdc_amount_in_base_units = usdc_amount_in_base_units.with_scale_round(6, bigdecimal::RoundingMode::Down);
            usdc_amount_in_base_units = usdc_amount_in_base_units * BigDecimal::from_str("1000000").unwrap();
        
        if self.balances.get(&args.user_id).is_none() {
            let user_balance = create_user_balance(&self.pool, args.user_id, usdc_amount_in_base_units).await.unwrap();
            self.balances.insert(args.user_id, user_balance);
            return;
        }

        let user_balance = self.balances.get_mut(&args.user_id).unwrap();
        user_balance.free_quote_qty += usdc_amount_in_base_units;

        self.event_producer.publish_balance_event(user_balance.clone()).await.unwrap();
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
}