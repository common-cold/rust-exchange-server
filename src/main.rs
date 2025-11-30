use actix_web::{App, HttpServer, web};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::{self, Sender};

use crate::{db::init_db, routes::signup, service::{BalanceEvent, BalanceWorker, Engine, EngineIx, OrderEvent, OrderWorker, TradeEvent, TradeWorker}};

pub mod db;
pub mod routes;
pub mod service;


#[derive(Clone)]
pub struct AppData {
    pub pool: Pool<Postgres>,
    pub engine_tx: Sender<EngineIx>
}


#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    
    let db = init_db().await?;
    
    let balance_db = db.clone();
    let trade_db = db.clone();
    let order_db = db.clone();
    let engine_db = db.clone();

    let (balance_tx, balance_rx) = mpsc::channel::<BalanceEvent>(100);
    let (trade_tx, trade_rx) = mpsc::channel::<TradeEvent>(100);
    let (order_tx, order_rx) = mpsc::channel::<OrderEvent>(100);
    let (engine_tx, engine_rx) = mpsc::channel::<EngineIx>(100);
    
    tokio::spawn(async move {
        let mut balance_worker = BalanceWorker::default(balance_db, balance_rx);
        balance_worker.run();
    });

    tokio::spawn(async move {
        let mut trade_worker = TradeWorker::default(trade_db, trade_rx);
        trade_worker.run();
    });

    tokio::spawn(async move {
        let mut order_worker = OrderWorker::default(order_db, order_rx);
        order_worker.run();
    });

    std::thread::spawn(move || {
        let mut engine = Engine::default(balance_tx, trade_tx, order_tx, engine_db, engine_rx);
        engine.run();
    });
    
    let app_data  = AppData {
        pool: db.clone(),
        engine_tx: engine_tx
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .service(signup)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
