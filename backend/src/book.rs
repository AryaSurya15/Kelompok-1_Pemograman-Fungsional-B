use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Book {
    pub id: i32,
    pub title: String,
    pub author: String,
    pub category: String,
    pub year: i32,
    pub total_copies: i32,
    pub available_copies: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NewBook {
    pub title: String,
    pub author: String,
    pub category: String,
    pub year: i32,
    pub total_copies: i32, // input dari user
}
