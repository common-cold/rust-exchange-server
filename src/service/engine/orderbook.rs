use std::collections::BTreeMap;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, sqlx::Type, Clone, Copy, PartialEq)]
#[sqlx(type_name = "varchar")]
#[sqlx(rename_all = "PascalCase")]
pub enum Side {
    Bid,
    Ask,
}

    #[derive(sqlx::FromRow, Serialize, Deserialize, Clone)]
    pub struct Order {
        pub id: Uuid,
        pub user_id: Uuid,
        pub price: BigDecimal,
        pub quantity: BigDecimal,
        pub filled_quantity: BigDecimal,
        pub side: Side,
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
}