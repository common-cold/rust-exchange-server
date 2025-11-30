use bigdecimal::BigDecimal;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{db::schema::DbUserBalance, service::UserBalance};

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
            created_at,
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


pub async fn create_user_balance(pool: &Pool<Postgres>, user_id: Uuid) -> anyhow::Result<UserBalance> {
    let db_balance = sqlx::query_as!(
        DbUserBalance,
        r#"
        INSERT INTO user_balance (
            user_id,
            free_base_qty,
            free_quote_qty,
            locked_base_qty,
            locked_quote_qty
        )
        VALUES (
            $1,
            0,
            0,
            0,
            0
        )
        RETURNING
            id,
            user_id,
            free_base_qty,
            free_quote_qty,
            locked_base_qty,
            locked_quote_qty,
            created_at,
            updated_at
        "#,
        user_id
    )
    .fetch_one(pool)
    .await?;

    let user_balance  = UserBalance { 
        id: db_balance.id, 
        user_id, 
        free_base_qty: BigDecimal::from(0), 
        free_quote_qty: BigDecimal::from(0), 
        locked_base_qty: BigDecimal::from(0), 
        locked_quote_qty: BigDecimal::from(0) 
    };

    Ok(user_balance)
}

pub async fn update_user_balance(pool: &Pool<Postgres>, updated_balance: UserBalance) -> anyhow::Result<()> {
    let db_balance = sqlx::query_as!(
        DbUserBalance,
        r#"
        UPDATE user_balance
        SET
            free_base_qty = $1,
            free_quote_qty = $2,
            locked_base_qty = $3,
            locked_quote_qty = $4,
            updated_at     = NOW()
        WHERE user_id = $5
        "#,
        updated_balance.free_base_qty,
        updated_balance.free_quote_qty,
        updated_balance.locked_base_qty,
        updated_balance.locked_quote_qty,
        updated_balance.user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(())
} 