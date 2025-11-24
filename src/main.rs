use actix_web::{App, HttpServer, web};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::{self, Receiver, Sender};

use crate::{db::init_db, routes::signup, service::{Engine, EngineIx}};

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
    
    let (tx, mut rx) = mpsc::channel::<EngineIx>(32);
    let db = init_db().await?;

    let app_data  = AppData {
        pool: db.clone(),
        engine_tx: tx
    };
    
    std::thread::spawn(move || {
        let mut engine = Engine::default();
        engine.run(db, &mut rx);
    });
    

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
