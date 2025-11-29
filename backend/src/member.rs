use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

/// Satu anggota perpustakaan (sesuai tabel `members`).
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Member {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub joined_at: NaiveDateTime,
}

/// Payload untuk membuat anggota baru.
#[derive(Debug, Clone, Deserialize)]
pub struct NewMember {
    pub name: String,
    pub email: String,
}
