use sqlx::{mysql::MySqlPoolOptions, MySqlPool};
use std::env;

/// Membuat connection pool ke MySQL.
/// Dipanggil sekali di awal aplikasi, lalu disimpan di AppState.
pub async fn create_pool() -> MySqlPool {
    // DATABASE_URL diambil dari .env
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database")
}
