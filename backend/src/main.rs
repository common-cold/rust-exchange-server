use actix_web::{App, HttpServer, web};
use common::{EngineIx};
use db::init_db;
use runtime::AppRuntime;
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::{Sender};

use crate::controller::{cancel_user_order, create_order, debug_engine, onramp_balance, signup};

mod controller;
mod service;

#[derive(Clone)]
pub struct AppData {
    pub pool: Pool<Postgres>,
    pub engine_tx: Sender<EngineIx>
}


#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    
    let db = init_db().await?;
    
    let app_runtime = AppRuntime::run(db.clone());
    
    let app_data  = AppData {
        pool: db.clone(),
        engine_tx: app_runtime.engine_tx
    };

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_data.clone()))
            .service(signup)
            .service(create_order)
            .service(cancel_user_order)
            .service(onramp_balance)
            .service(debug_engine)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
