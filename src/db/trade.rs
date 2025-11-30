use sqlx::{Pool, Postgres};

use crate::{db::schema::DbTrade, service::{InsertTradeArgs, Trade}};


pub async fn create_trade(pool: &Pool<Postgres>, insert_trade_args: InsertTradeArgs) -> anyhow::Result<Trade> {
    let db_trade = sqlx::query_as!(
        DbTrade,
        r#"
        INSERT INTO trades (
            buy_order_id,
            sell_order_id,
            price,
            quantity
        )
        VALUES (
            $1,
            $2,
            $3,
            $4
        )
        RETURNING
            id,
            buy_order_id,
            sell_order_id,
            price,
            quantity,
            created_at
        "#,
        insert_trade_args.buy_order_id,
        insert_trade_args.sell_order_id,
        insert_trade_args.price,
        insert_trade_args.quantity
    )
    .fetch_one(pool)
    .await?;

    let trade  = Trade { 
        id: db_trade.id,
        buy_order_id: db_trade.buy_order_id,
        sell_order_id: db_trade.sell_order_id,
        price: db_trade.price,
        quantity: db_trade.quantity,
        created_at: db_trade.created_at.timestamp_millis()
    };

    Ok(trade)
}