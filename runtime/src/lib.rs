use common::{BalanceEvent, EngineIx, OrderEvent, TradeEvent};
use engine::Engine;
use redis_service::RedisConnection;
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::{self, Sender};
use workers::{BalanceWorker, OrderWorker, TradeWorker};

pub struct AppRuntime {
    pub engine_tx: Sender<EngineIx>
}

impl AppRuntime {
    pub fn run(db: Pool<Postgres>, redis: RedisConnection) -> Self {
        let balance_db = db.clone();
        let trade_db = db.clone();
        let order_db = db.clone();
        let engine_db = db.clone();

        let balance_redis = redis.clone();
        let trade_redis = redis.clone();
        let order_redis = redis.clone();
        let engine_redis = redis.clone();

        let (balance_tx, balance_rx) = mpsc::channel::<BalanceEvent>(100);
        let (trade_tx, trade_rx) = mpsc::channel::<TradeEvent>(100);
        let (order_tx, order_rx) = mpsc::channel::<OrderEvent>(100);
        let (engine_tx, engine_rx) = mpsc::channel::<EngineIx>(100);
        
        tokio::spawn(async move {
            let mut balance_worker = BalanceWorker::default(balance_db, balance_rx, balance_redis);
            balance_worker.run().await;
        });

        tokio::spawn(async move {
            let mut trade_worker = TradeWorker::default(trade_db, trade_rx, trade_redis);
            trade_worker.run().await;
        });

        tokio::spawn(async move {
            let mut order_worker = OrderWorker::default(order_db, order_rx, order_redis);
            order_worker.run().await;
        });

        std::thread::spawn(move || {
            let mut engine = Engine::default(balance_tx, trade_tx, order_tx, engine_db, engine_rx, engine_redis);
            engine.run();
        });

        AppRuntime { 
            engine_tx: engine_tx 
        }
    }
}