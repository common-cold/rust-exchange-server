use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{OrderType, Side};

#[derive(Serialize, Deserialize)]
pub struct SignUp {
    pub email: String,
    pub password: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateOrderArgs {
    pub order_type: OrderType,
    pub side: Side,
    pub user_id: Uuid,
    pub limit_price: BigDecimal,
    pub base_qty: BigDecimal,
    pub quote_qty: BigDecimal
}

#[derive(Serialize, Deserialize)]
pub struct InsertTradeArgs {
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: BigDecimal,
    pub quantity: BigDecimal
}

#[derive(Debug)]
pub enum EngineIx {
    CreateLimitOrder(CreateOrderArgs),
    CreateMarketOrder(CreateOrderArgs),
    CancelOrder {
        key: String
    },
    Shutdown
}