use std::{env::{self}, str::FromStr};

use anyhow::anyhow;
use bigdecimal::{BigDecimal};
use common::{CreateOrderArgs, Currency, DbUser, EngineIx, ExchangeRateApiDto, OnRampArgs, OrderType, SignUp};
use db::create_user;
use dotenv::dotenv;
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Sender;
use uuid::Uuid;

pub async fn create_user_in_db(db: &Pool<Postgres>, args: SignUp) -> anyhow::Result<DbUser> {
    create_user(db, &args.email, &args.password).await
}

pub async fn create_order_in_engine(engine_tx: Sender<EngineIx>, args: CreateOrderArgs) -> anyhow::Result<()> {
    let result = match args.order_type {
        OrderType::Limit => engine_tx.send(EngineIx::CreateLimitOrder(args)).await,
        OrderType::Market => engine_tx.send(EngineIx::CreateMarketOrder(args)).await  
    };

    result.map_err(|e| anyhow!("{:?}", e))
}

pub async fn cancel_order(engine_tx: Sender<EngineIx>, order_id: Uuid) -> anyhow::Result<()> {
    let result = engine_tx.send(
        EngineIx::CancelOrder { 
            order_id: order_id 
        }).await;
    result.map_err(|e| anyhow!("{:?}", e))
}

pub async fn onramp(engine_tx: Sender<EngineIx>, mut args: OnRampArgs) -> anyhow::Result<()> {
    let rate = fetch_conversion_rate(args.currency).await?;

    args.usdc_conversion_rate = Some(BigDecimal::from_str(&rate.to_string()).unwrap());

    let result = engine_tx.send(EngineIx::OnRamp(args)).await;

    result.map_err(|e| anyhow!("{:?}", e))
}

pub async fn fetch_conversion_rate(currency: Currency) -> anyhow::Result<f64> {
    dotenv().ok();
    let url = env::var("EXCHANGE_RATE_API_URL")?;
    let response = reqwest::get(url)
        .await?
        .json::<ExchangeRateApiDto>()
        .await?;

    let rate_option = match currency {
        Currency::INR => response.usdc.get("inr"),
        Currency::EUR => response.usdc.get("eur"),
        Currency::USD => response.usdc.get("usd")
    };
    
    if rate_option.is_none() {
        return Err(anyhow!("{:#?} mapping does not exist in api", currency));
    }

    Ok(rate_option.unwrap().clone())
}