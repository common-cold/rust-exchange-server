use backend::{cancel_order, create_order_in_engine, fetch_conversion_rate, onramp};
use bigdecimal::BigDecimal;
use common::{AcknowledgementEvent, CreateOrderArgs, Currency, DbUser, EngineIx, OnRampArgs, Order, Orderbook, Trade, UserBalance};
use db::{create_user, get_all_trades, get_order_by_user_id, get_trades_by_buy_and_sell_order_id, get_user_balance};
use event_bus::consumer::EventBusConsumer;
use redis::{aio::ConnectionManager};
use redis_service::RedisConnection;
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};
use testcontainers::{ContainerAsync, GenericImage, runners::AsyncRunner};
use testcontainers_modules::postgres::Postgres as Pg;
use tokio::{sync::mpsc::{self, Sender}, time::sleep};
use runtime::AppRuntime;
use uuid::Uuid;
use std::{collections::HashMap, str::FromStr, time::Duration};

mod scenarios;

pub struct TestHarness {
   pub engine_tx: Sender<EngineIx>,
   pub db: Pool<Postgres>,
   pub postgres_testcontainer: ContainerAsync<Pg>,
   pub redis_testcontainer: ContainerAsync<GenericImage>
}

impl TestHarness {
    pub async fn start() -> Self {
        let (db, node) = TestHarness::init_postgres_testcontainer().await.unwrap();
        let (redis, redis_testcontainer) = TestHarness::init_redis_testcontainer().await.unwrap();
        let runtime_redis = redis.clone();
        
        EventBusConsumer::run(db.clone(), redis).await.unwrap();
        
        let app_runtime = AppRuntime::run(db.clone(), runtime_redis);
        
        Self {
          engine_tx: app_runtime.engine_tx,
          db: db,
          postgres_testcontainer: node,
          redis_testcontainer: redis_testcontainer
        }
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        let (harness_tx, mut harness_rx) = mpsc::channel::<AcknowledgementEvent>(1);

        //stop engine and workers
        self.engine_tx.send(EngineIx::Shutdown(harness_tx)).await.unwrap();

        //wait for ack
        loop {
            if let Some(cmd) = harness_rx.recv().await {
                match cmd {
                    AcknowledgementEvent::Shutdown => {
                        break;
                    },
                    _ => {}
                }
            }
        }

        Ok(())
    }

    pub async fn flush(&self) -> anyhow::Result<()> {
        let (harness_tx, mut harness_rx) = mpsc::channel::<AcknowledgementEvent>(1);

        //send Flush command
        self.engine_tx.send(EngineIx::Flush(harness_tx)).await.unwrap();

        //wait for ack
        loop {
            if let Some(cmd) = harness_rx.recv().await {
                match cmd {
                    AcknowledgementEvent::Flush => {
                        break;
                    },
                    _ => {}
                }
            }
        }
        
        Ok(())
    }

    pub async fn get_engine_state(&self) -> anyhow::Result<(Orderbook, HashMap<Uuid, UserBalance>)> {
        let (harness_tx, mut harness_rx) = mpsc::channel::<AcknowledgementEvent>(1);

        //send State command
        self.engine_tx.send(EngineIx::State(harness_tx)).await.unwrap();

        //wait for ack
        loop {
            if let Some(cmd) = harness_rx.recv().await {
                match cmd {
                    AcknowledgementEvent::State((orderbook, balances)) => {
                        return Ok((orderbook, balances));
                    },
                    _ => {}
                }
            }
        }
    }
 
    pub async fn init_postgres_testcontainer() -> anyhow::Result<(Pool<Postgres>, ContainerAsync<Pg>)> {
        let postgres_image = Pg::default();
        let node = postgres_image.start().await?;

        let host = node.get_host().await?;
        let port = node.get_host_port_ipv4(5432).await?;
        let connection_string = &format!(
            "postgres://postgres:postgres@{}:{}/postgres?sslmode=disable",
            host,
            port
        );

        let pool = loop {
            match PgPoolOptions::new()
                .max_connections(10)
                .connect(&connection_string).await {
                Ok(pool) => break pool,
                Err(_) => sleep(Duration::from_millis(500)).await,
            }
        };
        
        sqlx::migrate!("../db/test-migrations")
        .run(&pool)
        .await
        .unwrap();

        Ok((pool, node))
    }

    pub async fn init_redis_testcontainer() -> anyhow::Result<(RedisConnection, ContainerAsync<GenericImage>)>{
        let container = GenericImage::new("redis", "7.2.4")
            .with_exposed_port(testcontainers::core::ContainerPort::Tcp(6379))
            .start()
            .await?;
        let host = container.get_host().await?;
        let host_port = container.get_host_port_ipv4(6379).await?;

        let url = format!("redis://{host}:{host_port}");
        let client = redis::Client::open(url)?;
        let connection_manager=  ConnectionManager::new(client.clone()).await?;

        let redis_connection = RedisConnection {
            client: client,
            connection_manger: connection_manager
        };

        Ok((redis_connection, container))
    }
    

    pub async fn create_user_in_db(&self, email: &String, pass: &String) -> DbUser {
        create_user(&self.db, &email, &pass).await.unwrap()
    }

    pub async fn create_order(&self, args: CreateOrderArgs) -> anyhow::Result<()> {
        create_order_in_engine(self.engine_tx.clone(), args).await
    }

    pub async fn get_db_order_by_user_id(&self, id: Uuid) -> Order {
        get_order_by_user_id(&self.db, id).await.unwrap()
    }

    pub async fn get_db_trades(&self) -> Vec<Trade> {
        get_all_trades(&self.db).await.unwrap()
    }

    pub async fn get_db_trades_by_buy_sell_order_id(&self, buy_order_id: Uuid, sell_order_id: Uuid) -> Vec<Trade> {
        get_trades_by_buy_and_sell_order_id(&self.db, buy_order_id, sell_order_id).await.unwrap()
    }

    pub async fn get_balance_from_db(&self, user_id: Uuid) -> UserBalance {
        get_user_balance(&self.db, user_id).await.unwrap()
    }

    pub async fn cancel_order(&self, order_id: Uuid) {
        cancel_order(self.engine_tx.clone(), order_id).await.unwrap()
    }

    pub async fn onramp_balance(&self, args: OnRampArgs) {
        onramp(self.engine_tx.clone(), args).await.unwrap();
    }

    pub async fn get_conversion_rate(&self, currency: Currency) -> f64 {
        fetch_conversion_rate(currency).await.unwrap()
    }

    pub fn calculate_usdc_base_units(&self, amount: BigDecimal, conversion_rate: BigDecimal) -> BigDecimal {
        let mut usdc_amount_in_base_units = &amount / conversion_rate;
        usdc_amount_in_base_units = usdc_amount_in_base_units.with_scale_round(6, bigdecimal::RoundingMode::Down);
        usdc_amount_in_base_units = usdc_amount_in_base_units * BigDecimal::from_str("1000000").unwrap();
        return usdc_amount_in_base_units;
    }
       
}


impl Drop for TestHarness {
    fn drop(&mut self) {
        let (harness_tx, _) = mpsc::channel::<AcknowledgementEvent>(1);
        let _ = self.engine_tx.try_send(EngineIx::Shutdown(harness_tx));
    }
}
