use sqlx::{Pool, Postgres};

use crate::db::schema::DbUser;

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