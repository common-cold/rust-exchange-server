use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::service::{OrderType, Side, Status};


#[derive(Debug, Serialize, Deserialize)]
pub struct DbUser {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<Utc>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbOrder {
    pub id: Uuid,
    pub user_id: Uuid,
    pub order_type: OrderType,
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub filled_quantity: BigDecimal,
    pub side: Side,
    pub status: Status,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbUserBalance {
    pub id: Uuid,
    pub user_id: Uuid,

    pub free_base_qty: BigDecimal,
    pub free_quote_qty: BigDecimal,

    pub locked_base_qty: BigDecimal,
    pub locked_quote_qty: BigDecimal,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DbTrade {
    pub id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub created_at: DateTime<Utc>
}