use backend::create_order_in_engine;
use common::{CreateOrderArgs, DbUser, EngineIx};
use db::{create_user};
use sqlx::{PgConnection, Pool, Postgres, pool::PoolConnection, postgres::PgPoolOptions};
use tokio::sync::mpsc::{self, Sender};
use runtime::AppRuntime;
use uuid::Uuid;
use std::env;
use dotenv::dotenv;

mod scenarios;


pub struct TestHarness {
   pub engine_tx: Sender<EngineIx>,
   pub db: Pool<Postgres>,
   schema: String
}

impl TestHarness {
    pub async fn start() -> Self {
        dotenv().ok();
        let DATABASE_URL = env::var("DATABASE_URL").unwrap();

        let schema = format!("test_{}", Uuid::new_v4().simple());
        // bootstrap pool (no search_path)
        let bootstrap_db = PgPoolOptions::new()
            .max_connections(1)
            .connect(&DATABASE_URL)
            .await.unwrap();

        let mut conn = bootstrap_db.acquire().await.unwrap();

        // create schema
        TestHarness::create_schema(&mut *conn, &schema).await.unwrap();
        let db = TestHarness::init_db_test(schema.clone()).await.unwrap();
        
        // let schema = TestHarness::create_schema(&mut *conn, &schema).await.unwrap();
        
        // TestHarness::set_search_path(&mut *conn, &schema).await.unwrap();
        
        sqlx::migrate!("../db/migrations")
        .run(&db)
        .await
        .unwrap();

        let app_runtime = AppRuntime::run(db.clone());
        Self {
          engine_tx: app_runtime.engine_tx,
          db: db,
          schema: schema
        }
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        //stop engine and workers
        self.engine_tx.send(EngineIx::Shutdown).await.unwrap();
        
        //drop db
        TestHarness::drop_schema(&self.db, &self.schema).await.unwrap();

        Ok(())
    }


    pub async fn create_user(&self, email: &String, pass: &String) -> DbUser {
        create_user(&self.db, &email, &pass).await.unwrap()
    }

    pub async fn create_order(&self, args: CreateOrderArgs) -> anyhow::Result<()> {
        create_order_in_engine(self.engine_tx.clone(), args).await
    }

    pub async fn init_db_test(schema: String) -> anyhow::Result<Pool<Postgres>> {
        dotenv().ok();
        let DATABASE_URL = env::var("DATABASE_URL")?;

        let db = PgPoolOptions::new()
        .max_connections(10)
        .after_connect(move |conn, _| {
            let schema = schema.clone();
            Box::pin(async move {
                sqlx::query(&format!("SET search_path TO {}, public", schema))
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&DATABASE_URL)
        .await?;

        Ok(db)
    }

    pub async fn create_schema(pool: &mut PgConnection, schema: &String) -> anyhow::Result<()> {
        
        sqlx::query(
            &format!(r#"CREATE SCHEMA "{}""#, schema)
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }

    pub async fn set_search_path(pool: &mut PgConnection, schema: &str) -> anyhow::Result<()> {
        sqlx::query(
            &format!(r#"SET search_path TO "{}", public"#, schema)
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn drop_schema(pool: &Pool<Postgres>, schema: &str) -> anyhow::Result<()> {
        sqlx::query(
            &format!(r#"DROP SCHEMA "{}" CASCADE"#, schema)
        )
        .execute(pool)
        .await?;    

        Ok(())
    }
}

