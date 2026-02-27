use std::str::FromStr;

use bigdecimal::BigDecimal;
use common::{CreateOrderArgs, OrderType, Side, Status};

use crate::TestHarness;



#[tokio::test]
pub async fn place_bid_then_ask_then_cancel_bid() {
    let harness = TestHarness::start().await;

    //user1-bid
    let email = String::from("user1_bid@gmail.com");
    let pass = String::from("password");
    let user1_bid = harness.create_user_in_db(&email, &pass).await;
    let limit_price1_bid = BigDecimal::from_str("120000000").unwrap();
    let base_qty1_bid = BigDecimal::from_str("10000000000").unwrap();
    let quote_qty1_bid = BigDecimal::from_str("1200000000").unwrap();

    //user1-ask
    let email1_ask = String::from("user1_ask@gmail.com");
    let pass1_ask = String::from("password");
    let user1_ask = harness.create_user_in_db(&email1_ask, &pass1_ask).await;
    let limit_price1_ask = BigDecimal::from_str("115000000").unwrap();
    let base_qty1_ask = BigDecimal::from_str("2000000000").unwrap();
    let quote_qty1_ask = BigDecimal::from_str("0").unwrap();

    //user2-ask
    let email2_ask = String::from("user2_ask@gmail.com");
    let pass2_ask = String::from("password");
    let user2_ask = harness.create_user_in_db(&email2_ask, &pass2_ask).await;
    let limit_price2_ask = BigDecimal::from_str("120000000").unwrap();
    let base_qty2_ask = BigDecimal::from_str("4000000000").unwrap();
    let quote_qty2_ask = BigDecimal::from_str("0").unwrap();

    
    let args1_bid = CreateOrderArgs {
        order_type: OrderType::Limit,
        side: Side::Bid,
        user_id: user1_bid.id,
        limit_price: Some(limit_price1_bid),
        base_qty: base_qty1_bid,
        quote_qty: quote_qty1_bid
    }; 
    let args1_ask = CreateOrderArgs {
        order_type: OrderType::Limit,
        side: Side::Ask,
        user_id: user1_ask.id,
        limit_price: Some(limit_price1_ask),
        base_qty: base_qty1_ask,
        quote_qty: quote_qty1_ask
    }; 
    let args2_ask = CreateOrderArgs {
        order_type: OrderType::Limit,
        side: Side::Ask,
        user_id: user2_ask.id,
        limit_price: Some(limit_price2_ask),
        base_qty: base_qty2_ask,
        quote_qty: quote_qty2_ask
    }; 

    
    harness.create_order(args1_bid.clone()).await.unwrap();
    harness.create_order(args1_ask.clone()).await.unwrap();    
    harness.create_order(args2_ask.clone()).await.unwrap();

    harness.flush().await.unwrap();


    let mut order1_bid = harness.get_db_order_by_user_id(user1_bid.id).await;
    let order1_ask = harness.get_db_order_by_user_id(user1_ask.id).await;
    let order2_ask = harness.get_db_order_by_user_id(user2_ask.id).await;
    


    //order checks
    assert_eq!(order1_bid.filled_quantity, BigDecimal::from_str("6000000000").unwrap());
    assert_eq!(order1_bid.quantity, args1_bid.base_qty);
    assert_eq!(order1_bid.order_type, args1_bid.order_type);
    assert_eq!(order1_bid.price.as_ref().unwrap(), args1_bid.limit_price.as_ref().unwrap());
    assert_eq!(order1_bid.side, args1_bid.side);
    assert_eq!(order1_bid.status, Status::Open);

    assert_eq!(order1_ask.filled_quantity, BigDecimal::from_str("2000000000").unwrap());
    assert_eq!(order1_ask.quantity, args1_ask.base_qty);
    assert_eq!(order1_ask.order_type, args1_ask.order_type);
    assert_eq!(order1_ask.price.as_ref().unwrap(), args1_ask.limit_price.as_ref().unwrap());
    assert_eq!(order1_ask.side, args1_ask.side);
    assert_eq!(order1_ask.status, Status::Close);

    assert_eq!(order2_ask.filled_quantity, BigDecimal::from_str("4000000000").unwrap());
    assert_eq!(order2_ask.quantity, args2_ask.base_qty);
    assert_eq!(order2_ask.order_type, args2_ask.order_type);
    assert_eq!(order2_ask.price.as_ref().unwrap(), args2_ask.limit_price.as_ref().unwrap());
    assert_eq!(order2_ask.side, args2_ask.side);
    assert_eq!(order2_ask.status, Status::Close);

    


    //balance checks
    let mut user1_bid_balance = harness.get_balance_from_db(user1_bid.id).await;
    let user1_ask_balance = harness.get_balance_from_db(user1_ask.id).await;
    let user2_ask_balance = harness.get_balance_from_db(user2_ask.id).await;

    let free_quote_qty = BigDecimal::from_str("0").unwrap();
    let locked_quote_qty = BigDecimal::from_str("480000000").unwrap();

    assert_eq!(user1_bid_balance.user_id, user1_bid.id);
    assert_eq!(user1_bid_balance.free_base_qty, BigDecimal::from_str("6000000000").unwrap());
    assert_eq!(user1_bid_balance.free_quote_qty, free_quote_qty);
    assert_eq!(user1_bid_balance.locked_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(user1_bid_balance.locked_quote_qty, locked_quote_qty);

    assert_eq!(user1_ask_balance.user_id, user1_ask.id);
    assert_eq!(user1_ask_balance.free_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(user1_ask_balance.free_quote_qty, BigDecimal::from_str("240000000").unwrap());
    assert_eq!(user1_ask_balance.locked_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(user1_ask_balance.locked_quote_qty, BigDecimal::from_str("0").unwrap());

    assert_eq!(user2_ask_balance.user_id, user2_ask.id);
    assert_eq!(user2_ask_balance.free_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(user2_ask_balance.free_quote_qty, BigDecimal::from_str("480000000").unwrap());
    assert_eq!(user2_ask_balance.locked_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(user2_ask_balance.locked_quote_qty, BigDecimal::from_str("0").unwrap());

    

    //trade checks
    let mut trades = harness.get_db_trades().await;
    assert_eq!(trades.len(), 2);

    trades = harness.get_db_trades_by_buy_sell_order_id(order1_bid.id, order1_ask.id).await;
    let bid1_ask1_trade = trades.first().unwrap();
    assert_eq!(bid1_ask1_trade.buy_order_id, order1_bid.id);
    assert_eq!(bid1_ask1_trade.sell_order_id, order1_ask.id);
    assert_eq!(&bid1_ask1_trade.price, order1_bid.price.as_ref().unwrap());
    assert_eq!(bid1_ask1_trade.quantity, BigDecimal::from_str("2000000000").unwrap());

    trades = harness.get_db_trades_by_buy_sell_order_id(order1_bid.id, order2_ask.id).await;
    let bid1_ask2_trade = trades.first().unwrap();
    assert_eq!(bid1_ask2_trade.buy_order_id, order1_bid.id);
    assert_eq!(bid1_ask2_trade.sell_order_id, order2_ask.id);
    assert_eq!(&bid1_ask2_trade.price, order1_bid.price.as_ref().unwrap());
    assert_eq!(bid1_ask2_trade.quantity, BigDecimal::from_str("4000000000").unwrap());
    

    ////in memory state check
    let (in_memory_orderbook, in_memory_balances) = harness.get_engine_state().await.unwrap();
    
    ////in memory orderbook check
    //ask
    assert_eq!(in_memory_orderbook.asks.len(), 0);
    //bid
    let (_price, bid_list) = in_memory_orderbook.bids.get_key_value(args1_bid.limit_price.as_ref().unwrap()).unwrap();
    assert_eq!(bid_list.len(), 1);
    let bid = &bid_list[0];
    assert_eq!(bid.filled_quantity, BigDecimal::from_str("6000000000").unwrap());
    assert_eq!(bid.id, order1_bid.id);
    assert_eq!(bid.order_type, OrderType::Limit);
    assert_eq!(bid.price.as_ref().unwrap(), &BigDecimal::from_str("120000000").unwrap());
    assert_eq!(bid.quantity, BigDecimal::from_str("10000000000").unwrap());
    assert_eq!(bid.side, Side::Bid);
    assert_eq!(bid.status, Status::Open);
    assert_eq!(bid.user_id, user1_bid.id);


    ////in memory balance check
    assert_eq!(in_memory_balances.len(), 3);
    let mut in_memory_user1_bid_balance = in_memory_balances.get(&user1_bid.id).unwrap();
    let in_memory_user1_ask_balance = in_memory_balances.get(&user1_ask.id).unwrap();
    let in_memory_user2_ask_balance = in_memory_balances.get(&user2_ask.id).unwrap();

    assert_eq!(in_memory_user1_bid_balance.user_id, user1_bid.id);
    assert_eq!(in_memory_user1_bid_balance.free_base_qty, BigDecimal::from_str("6000000000").unwrap());
    assert_eq!(in_memory_user1_bid_balance.free_quote_qty, free_quote_qty);
    assert_eq!(in_memory_user1_bid_balance.locked_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(in_memory_user1_bid_balance.locked_quote_qty, locked_quote_qty);

    assert_eq!(in_memory_user1_ask_balance.user_id, user1_ask.id);
    assert_eq!(in_memory_user1_ask_balance.free_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(in_memory_user1_ask_balance.free_quote_qty, BigDecimal::from_str("240000000").unwrap());
    assert_eq!(in_memory_user1_ask_balance.locked_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(in_memory_user1_ask_balance.locked_quote_qty, BigDecimal::from_str("0").unwrap());

    assert_eq!(in_memory_user2_ask_balance.user_id, user2_ask.id);
    assert_eq!(in_memory_user2_ask_balance.free_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(in_memory_user2_ask_balance.free_quote_qty, BigDecimal::from_str("480000000").unwrap());
    assert_eq!(in_memory_user2_ask_balance.locked_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(in_memory_user2_ask_balance.locked_quote_qty, BigDecimal::from_str("0").unwrap());

    
    //cancel order
    harness.cancel_order(order1_bid.id).await;
    harness.flush().await.unwrap();

    order1_bid = harness.get_db_order_by_user_id(user1_bid.id).await;


    //order checks
    assert_eq!(order1_bid.filled_quantity, BigDecimal::from_str("6000000000").unwrap());
    assert_eq!(order1_bid.quantity, args1_bid.base_qty);
    assert_eq!(order1_bid.order_type, args1_bid.order_type);
    assert_eq!(order1_bid.price.as_ref().unwrap(), args1_bid.limit_price.as_ref().unwrap());
    assert_eq!(order1_bid.side, args1_bid.side);
    assert_eq!(order1_bid.status, Status::Cancelled);


    //balance checks
    user1_bid_balance = harness.get_balance_from_db(user1_bid.id).await;

    let quote_qty_to_unlock = (&order1_bid.quantity - &order1_bid.filled_quantity) * order1_bid.price.as_ref().unwrap();
    let new_free_quote_qty = &free_quote_qty + &quote_qty_to_unlock;
    let new_locked_quote_qty = &locked_quote_qty - &quote_qty_to_unlock;

    assert_eq!(user1_bid_balance.user_id, user1_bid.id);
    assert_eq!(user1_bid_balance.free_base_qty, BigDecimal::from_str("6000000000").unwrap());
    assert_eq!(user1_bid_balance.free_quote_qty, new_free_quote_qty);
    assert_eq!(user1_bid_balance.locked_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(user1_bid_balance.locked_quote_qty, new_locked_quote_qty);


    ////in memory state check
    let (_in_memory_orderbook, in_memory_balances) = harness.get_engine_state().await.unwrap();

    //in memory balance check
    assert_eq!(in_memory_balances.len(), 3);
    in_memory_user1_bid_balance = in_memory_balances.get(&user1_bid.id).unwrap();

    assert_eq!(in_memory_user1_bid_balance.user_id, user1_bid.id);
    assert_eq!(in_memory_user1_bid_balance.free_base_qty, BigDecimal::from_str("6000000000").unwrap());
    assert_eq!(in_memory_user1_bid_balance.free_quote_qty, new_free_quote_qty);
    assert_eq!(in_memory_user1_bid_balance.locked_base_qty, BigDecimal::from_str("0").unwrap());
    assert_eq!(in_memory_user1_bid_balance.locked_quote_qty, new_locked_quote_qty);

    harness.shutdown().await.unwrap();
}