use std::{collections::BTreeMap};

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

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone, Debug)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Orderbook {
    pub bids: BTreeMap<BigDecimal, Vec<Order>>,
    pub asks: BTreeMap<BigDecimal, Vec<Order>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UserBalance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub free_base_qty: BigDecimal,
    pub free_quote_qty: BigDecimal,
    pub locked_base_qty: BigDecimal,
    pub locked_quote_qty: BigDecimal,
}

#[derive(sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub created_at: i64
}