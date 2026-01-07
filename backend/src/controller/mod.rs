use actix_web::{HttpResponse, get, post, web};
use common::{CreateOrderArgs, SignUp};
use serde_json::json;
use crate::{AppData, service::{create_order_in_engine, create_user_in_db}};



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