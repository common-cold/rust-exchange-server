use std::collections::BTreeMap;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::DbOrder;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq)]
#[sqlx(type_name = "varchar")]
#[sqlx(rename_all = "PascalCase")]
pub enum Side {
    Bid,
    Ask,
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq)]
#[sqlx(type_name = "varchar")]
#[sqlx(rename_all = "PascalCase")]
pub enum Status {
    Open,
    Close,
    Cancelled
}

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq)]
#[sqlx(type_name = "varchar")]
#[sqlx(rename_all = "PascalCase")]
pub enum OrderType {
    Limit,
    Market,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct Order {
    pub id: Uuid,
    pub user_id: Uuid,
    pub order_type: OrderType,
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub filled_quantity: BigDecimal,
    pub side: Side,
    pub status: Status,
    pub created_at: i64,
    pub updated_at: i64
}

#[derive(Serialize, Deserialize)]
pub struct Orderbook {
    pub bids: BTreeMap<BigDecimal, Vec<Order>>,
    pub asks: BTreeMap<BigDecimal, Vec<Order>>,
}

impl Orderbook {
    pub fn default() -> Self {
        Self { 
            bids: BTreeMap::new(), 
            asks: BTreeMap::new()
        }
    }

    pub fn init_orderbook(orders: Vec<Order>) -> anyhow::Result<Orderbook> {
        let mut bids: BTreeMap<BigDecimal, Vec<Order>> = BTreeMap::new();
        let mut asks: BTreeMap<BigDecimal, Vec<Order>> = BTreeMap::new();

        for order in orders.iter() {
            match order.side {
                Side::Bid => {
                    match bids.get_mut(&order.price) {
                        None => {
                            let mut order_list = Vec::<Order>::new();
                            order_list.push(order.clone());
                            bids.insert(order.price.clone(), order_list);
                        }
                        Some(order_list) => {
                            let index = order_list.partition_point(
                                |o| o.created_at <= order.created_at
                            );
                            order_list.insert(index, order.clone());
                        }
                    }
                }    
                Side::Ask => {
                    match asks.get_mut(&order.price) {
                        None => {
                            let mut order_list = Vec::<Order>::new();
                            order_list.push(order.clone());
                            asks.insert(order.price.clone(), order_list);
                        }
                        Some(order_list) => {
                            let index = order_list.partition_point(
                                |o| o.created_at >= order.created_at
                            );
                            order_list.insert(index, order.clone());
                        }
                    }
                }
            }
        }

        let orderbook = Orderbook {
            bids: bids,
            asks: asks
        };

        Ok(orderbook)
    }

    pub fn add_order(&mut self, order: Order) -> anyhow::Result<()> {
        match order.side {
            Side::Bid => {
                match self.bids.get_mut(&order.price) {
                    None => {
                        let mut order_list = Vec::<Order>::new();
                        order_list.push(order.clone());
                        self.bids.insert(order.price.clone(), order_list);
                    }
                    Some(order_list) => {
                        let index = order_list.partition_point(
                            |o| o.created_at <= order.created_at
                        );
                        order_list.insert(index, order.clone());
                    }
                }
            }    
            Side::Ask => {
                match self.asks.get_mut(&order.price) {
                    None => {
                        let mut order_list = Vec::<Order>::new();
                        order_list.push(order.clone());
                        self.asks.insert(order.price.clone(), order_list);
                    }
                    Some(order_list) => {
                        let index = order_list.partition_point(
                            |o| o.created_at >= order.created_at
                        );
                        order_list.insert(index, order.clone());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn convert_db_order(db_order: &DbOrder) -> anyhow::Result<Order> {
        let order = Order {
            id: db_order.id,
            user_id: db_order.user_id,
            order_type: db_order.order_type,
            price: db_order.price.clone(),
            quantity: db_order.quantity.clone(),
            filled_quantity: db_order.filled_quantity.clone(),
            side: db_order.side,
            status: db_order.status,
            created_at: db_order.created_at.timestamp_millis(),
            updated_at: db_order.updated_at.timestamp_millis()
        };

        Ok(order)
    }

    pub fn remove_order(&mut self, order_id: Uuid, side: Side, price: &BigDecimal) -> anyhow::Result<()> {
        let book = match side {
            Side::Bid => {
                &mut self.bids
            }

            Side::Ask => {
                &mut self.asks
            }
        };

        match book.get_mut(price) {
            None => {
                return Err(anyhow::anyhow!("Orders at price: {:?} does not exist", price));
            }

            Some(order_list) => {
                match order_list.binary_search_by(
                    |order| order.price.cmp(price)
                ) {
                    //index must be 0
                    Ok(index) => {
                        let order = &order_list[index];
                        if order.id == order_id {
                            order_list.remove(index);
                        } else {
                            return Err(anyhow::anyhow!("Order does not exist"));
                        }
                    }
                    Err(_) => {
                        return Err(anyhow::anyhow!("Order does not exist"));
                    }
                }   
            }
        }
        Ok(())
    }

    pub fn determine_maker_taker_book(&mut self, side: Side) -> (&mut BTreeMap<BigDecimal, Vec<Order>>, &mut BTreeMap<BigDecimal, Vec<Order>>) {
        match side {
            Side::Bid => {
                (&mut self.asks, &mut self.bids)
            }
            Side::Ask => {
                (&mut self.bids, &mut self.asks)
            }
        }    
    }
}