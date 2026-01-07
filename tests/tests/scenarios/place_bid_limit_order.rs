use std::str::FromStr;
use bigdecimal::BigDecimal;
use common::{CreateOrderArgs, OrderType, Side};

use crate::TestHarness;


#[tokio::test]
pub async fn place_bid_limitorder() {
    let harness = TestHarness::start().await;

    let email = String::from("user@gmail.com");
    let pass = String::from("password");
    let user = harness.create_user(&email, &pass).await;

    let limit_price = BigDecimal::from_str("125000000").unwrap();
    let base_qty = BigDecimal::from_str("630000000").unwrap();
    let quote_qty = BigDecimal::from_str("5000000000").unwrap();

    let args = CreateOrderArgs {
        order_type: OrderType::Limit,
        side: Side::Bid,
        user_id: user.id,
        limit_price: limit_price ,
        base_qty: base_qty,
        quote_qty: quote_qty
    }; 

    let _ = harness.create_order(args).await;

    harness.shutdown().await.unwrap();
}