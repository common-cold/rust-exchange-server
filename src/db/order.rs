use anyhow::Ok;
use bigdecimal::BigDecimal;
use sqlx::{Pool, Postgres};

use crate::{db::schema::DbOrder, service::{CreateOrderArgs, Order, OrderType, Orderbook, Side, Status}};

pub async fn get_open_orders(pool: &Pool<Postgres>) -> anyhow::Result<Vec<Order>> {
    let db_orders = sqlx::query_as!(
        DbOrder,
        r#"
        SELECT 
            id,
            user_id,
            order_type AS "order_type: OrderType",
            price,
            quantity,
            filled_quantity,
            side AS "side: Side",
            status AS "status: Status",
            created_at, 
            updated_at
        FROM orders
        WHERE status = 'Open'
        ORDER BY created_at ASC
        "#
    ).fetch_all(pool)
    .await?;

    let mut orders: Vec<Order> = Vec::new();
    for order in db_orders.iter() {
        let item = Orderbook::convert_db_order(order)?;
        orders.push(item);
    } 

    Ok(orders)
}

pub async fn create_order(pool: &Pool<Postgres>, create_order_args: &CreateOrderArgs) -> anyhow::Result<Order> {
    let db_order = sqlx::query_as!(
        DbOrder,
        r#"
        INSERT INTO orders (
            order_type,
            user_id,
            price,
            quantity,
            filled_quantity,
            side,
            status
        )
        VALUES (
            $1,
            $2,
            $3,
            $4,
            $5,
            $6,
            'Open'
        )
        RETURNING 
            id,
            user_id,
            order_type AS "order_type: OrderType",
            price,
            quantity,
            filled_quantity,
            side AS "side: Side",
            status AS "status: Status",
            created_at, 
            updated_at
        "#,
        create_order_args.order_type as OrderType,
        create_order_args.user_id,
        create_order_args.limit_price,
        create_order_args.base_qty,
        BigDecimal::from(0),
        create_order_args.side as Side
    )
    .fetch_one(pool)
    .await?;
    
    let order = Orderbook::convert_db_order(&db_order)?;

    Ok(order)
}

pub async fn update_order(pool: &Pool<Postgres>, updated_order: Order) -> anyhow::Result<()> {
    let db_order = sqlx::query_as!(
        DbOrder,
        r#"
        UPDATE orders
        SET
            filled_quantity = $1,
            status          = $2,
            updated_at      = NOW()
        WHERE id = $3
        "#,
        updated_order.filled_quantity,
        updated_order.status as Status,
        updated_order.id
    )
    .fetch_one(pool)
    .await?;

    Ok(())
}