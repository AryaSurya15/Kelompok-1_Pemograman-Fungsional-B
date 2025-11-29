use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::NaiveDateTime;

/// Baris peminjaman di tabel `loans`.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Loan {
    pub id: i32,
    pub book_id: i32,
    pub member_id: i32,
    pub borrowed_at: NaiveDateTime,
    pub due_at: NaiveDateTime,
    pub returned_at: Option<NaiveDateTime>,
}

/// Payload untuk membuat peminjaman baru.
/// Kita kirim tanggal jatuh tempo sebagai string "YYYY-MM-DD" dari frontend.
#[derive(Debug, Clone, Deserialize)]
pub struct NewLoan {
    pub book_id: i32,
    pub member_id: i32,
    pub due_date: String, // contoh: "2025-12-01"
}
