use common::{DbTrade, InsertTradeArgs, Trade};
use sqlx::{Pool, Postgres};
use uuid::Uuid;


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

pub async fn get_all_trades(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Trade>> {
    let db_trades = sqlx::query_as!(
        DbTrade,
        r#"
            SELECT 
                id,
                buy_order_id,
                sell_order_id,
                price,
                quantity,
                created_at
            FROM trades    
        "#
    ).fetch_all(pool)
    .await?;

    let trades = db_trades.iter().map(|t| Trade {
        id: t.id,
        buy_order_id: t.buy_order_id,
        sell_order_id: t.sell_order_id,
        price: t.price.clone(),
        quantity: t.quantity.clone(),
        created_at: t.created_at.timestamp_millis(),
    }).collect::<Vec<Trade>>();

    Ok(trades)
}

pub async fn get_trades_by_buy_and_sell_order_id(pool: &Pool<Postgres>, buy_order_id: Uuid, sell_order_id: Uuid) -> anyhow::Result<Vec<Trade>> {
    let db_trades = sqlx::query_as!(
        DbTrade,
        r#"
            SELECT 
                id,
                buy_order_id,
                sell_order_id,
                price,
                quantity,
                created_at
            FROM trades 
            WHERE buy_order_id = $1
            AND sell_order_id = $2
            ORDER BY created_at ASC   
        "#,
        buy_order_id,
        sell_order_id
    ).fetch_all(pool)
    .await?;

    let trades = db_trades.iter().map(|t| Trade {
        id: t.id,
        buy_order_id: t.buy_order_id,
        sell_order_id: t.sell_order_id,
        price: t.price.clone(),
        quantity: t.quantity.clone(),
        created_at: t.created_at.timestamp_millis(),
    }).collect::<Vec<Trade>>();

    Ok(trades)
}