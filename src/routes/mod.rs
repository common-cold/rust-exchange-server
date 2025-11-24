use actix_web::{HttpResponse, web, get};
use crate::{AppData, service::create_user};

pub mod types;
pub use types::*;


#[get("/signup")]
pub async fn signup(data: web::Data<AppData>, body: web::Json<SignUp>) -> HttpResponse {
    match create_user(&data.pool.clone(), &body.email, &body.password).await {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string())
    }
}