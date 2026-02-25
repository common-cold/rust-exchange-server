use anyhow::anyhow;
use common::{CreateOrderArgs, DbUser, EngineIx, OrderType, SignUp};
use db::create_user;
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Sender;

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