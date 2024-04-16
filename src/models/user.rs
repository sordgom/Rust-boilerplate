use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub password_hash: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserRequest {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Login {
    pub email: String,
    #[serde(default)]
    pub remember_me: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Register {
    pub name: String,
    pub email: String,
}
