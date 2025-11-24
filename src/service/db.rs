use sqlx::{Pool, Postgres};

use crate::{db::schema::{DbOrder, DbUser, DbUserBalance}, service::{Order, Side, UserBalance, balance}};

pub async fn create_user(pool: &Pool<Postgres>, email: &str, password: &str) -> anyhow::Result<DbUser> {
    let user = sqlx::query_as!(
        DbUser,
        r#"
        INSERT INTO users (email, password_hash)
        VALUES ($1, $2)
        RETURNING id, email, password_hash, created_at
        "#,
        email,
        password
    )
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn get_orders(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Order>> {
    let db_orders = sqlx::query_as!(
        DbOrder,
        r#"
        SELECT 
            id,
            user_id,
            price,
            quantity,
            filled_quantity,
            side AS "side: Side",
            created_at, 
            updated_at
        FROM orders
        ORDER BY created_at ASC
        "#
    ).fetch_all(pool)
    .await?;

    let mut orders: Vec<Order> = Vec::new();
    for order in db_orders.iter() {
        let item = Order {
            id: order.id,
            user_id: order.user_id,
            price: order.price.clone(),
            quantity: order.quantity.clone(),
            filled_quantity: order.filled_quantity.clone(),
            side: order.side,
            created_at: order.created_at.timestamp_millis(),
            updated_at: order.updated_at.timestamp_millis()
        };
        orders.push(item);
    } 

    Ok(orders)
}

pub async fn get_all_user_balance(pool: &Pool<Postgres>) -> anyhow::Result<Vec<UserBalance>> {
    let db_balances = sqlx::query_as!(
        DbUserBalance,
        r#"
        SELECT 
            id,
            user_id,
            free_base_qty,
            free_quote_qty,
            locked_base_qty,
            locked_quote_qty,
            updated_at
        FROM user_balance
        "#
    ).fetch_all(pool)
    .await?;

    let mut balances: Vec<UserBalance> = Vec::new();
    for balance in db_balances.iter() {
        let item = UserBalance {
            id: balance.id,
            user_id: balance.user_id,
            free_base_qty: balance.free_base_qty.clone(),
            free_quote_qty: balance.free_quote_qty.clone(),
            locked_base_qty: balance.locked_base_qty.clone(),
            locked_quote_qty: balance.locked_quote_qty.clone()
        };
        balances.push(item);
    } 

    Ok(balances)
}