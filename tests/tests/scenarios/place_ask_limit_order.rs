use std::str::FromStr;

use bigdecimal::BigDecimal;
use common::{CreateOrderArgs, OrderType, Side, Status};

use crate::TestHarness;



#[tokio::test]
pub async fn place_ask_limit_order() {
    let harness = TestHarness::start().await;

    let email = String::from("user@gmail.com");
    let pass = String::from("password");
    let user = harness.create_user_in_db(&email, &pass).await;

    let limit_price = BigDecimal::from_str("120000000").unwrap();
    let base_qty = BigDecimal::from_str("2000000000").unwrap();
    let quote_qty = BigDecimal::from_str("0").unwrap();

    let args = CreateOrderArgs {
        order_type: OrderType::Limit,
        side: Side::Ask,
        user_id: user.id,
        limit_price: Some(limit_price) ,
        base_qty: base_qty,
        quote_qty: quote_qty
    }; 

    harness.create_order(args.clone()).await.unwrap();

    harness.flush().await.unwrap();

    let order = harness.get_db_order_by_user_id(user.id).await;

    assert_eq!(order.filled_quantity, BigDecimal::from_str("0").unwrap());
    assert_eq!(order.quantity, args.base_qty);
    assert_eq!(order.order_type, args.order_type);
    assert_eq!(order.price.as_ref().unwrap(), args.limit_price.as_ref().unwrap());
    assert_eq!(order.side, args.side);
    assert_eq!(order.status, Status::Open);

    harness.shutdown().await.unwrap();
}