use std::collections::HashMap;

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{OrderType, Side};

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Currency {
    INR,
    EUR,
    USD
}

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
    pub limit_price: Option<BigDecimal>,
    pub base_qty: BigDecimal,
    pub quote_qty: BigDecimal
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InsertTradeArgs {
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: BigDecimal,
    pub quantity: BigDecimal
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OnRampArgs {
    pub user_id: Uuid,
    pub currency: Currency,
    pub amount: BigDecimal,
    pub usdc_conversion_rate: Option<BigDecimal>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExchangeRateApiDto {
    pub date: String,
    pub usdc: HashMap<String, f64>
}