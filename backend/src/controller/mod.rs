use actix_web::{HttpResponse, get, post, web};
use common::{CreateOrderArgs, OnRampArgs, SignUp};
use serde_json::json;
use uuid::Uuid;
use crate::{AppData, service::{cancel_order, create_order_in_engine, create_user_in_db, onramp}};



#[get("/signup")]
pub async fn signup(data: web::Data<AppData>, body: web::Json<SignUp>) -> HttpResponse {
    match create_user_in_db(&data.pool.clone(), body.0).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}

#[post("/order/create")]
pub async fn create_order(data: web::Data<AppData>, body: web::Json<CreateOrderArgs>) -> HttpResponse {
    let result = create_order_in_engine(data.engine_tx.clone(), body.0).await;

    match result {
        Ok(()) => return HttpResponse::Ok().json(json!({
            "message": "Order submitted successfully"
        })),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };
}

#[post("/order/cancel/{id}")]
pub async fn cancel_user_order(data: web::Data<AppData>, path: web::Path<Uuid>) -> HttpResponse {
    let order_id = path.into_inner();

    let result = cancel_order(data.engine_tx.clone(), order_id).await;

    match result {
        Ok(()) => return HttpResponse::Ok().json(json!({
            "message": "Order submitted for cancelling successfully"
        })),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };
}

#[post("/balance/onramp")]
pub async fn onramp_balance(data: web::Data<AppData>, body: web::Json<OnRampArgs>) -> HttpResponse {
    let result = onramp(data.engine_tx.clone(), body.0).await;

    match result {
        Ok(()) => return HttpResponse::Ok().json(json!({
            "message": "Onramp request submitted successfully"
        })),
        Err(e) => return HttpResponse::InternalServerError().body(e.to_string())
    };
}

#[get("/debug")]
pub async fn debug_engine(data: web::Data<AppData>) -> HttpResponse {
    let _ = data.engine_tx.send(common::EngineIx::Debug).await;
    return HttpResponse::Ok().finish();
}