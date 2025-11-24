use std::{env};

use dotenv::dotenv;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub mod schema;

#[allow(non_snake_case)]
pub async fn init_db() -> anyhow::Result<Pool<Postgres>> {
    dotenv().ok();
    
    let DATABASE_URL = env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&DATABASE_URL).await?;


    Ok(pool)
}