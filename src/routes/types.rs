use serde::{Deserialize};

#[derive(Deserialize)]
pub struct SignUp {
    pub email: String,
    pub password: String
}