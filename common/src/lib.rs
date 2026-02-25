pub mod db_schema;
pub use db_schema::*;

pub mod state;
pub use state::*;

pub mod channel_events;
pub use channel_events::*;

pub mod dto;
pub use dto::*;


impl Orderbook {
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
}