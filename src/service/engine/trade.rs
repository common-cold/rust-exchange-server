use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(sqlx::FromRow, Serialize, Deserialize, Clone)]
pub struct Trade {
    pub id: Uuid,
    pub buy_order_id: Uuid,
    pub sell_order_id: Uuid,
    pub price: BigDecimal,
    pub quantity: BigDecimal,
    pub created_at: i64
}