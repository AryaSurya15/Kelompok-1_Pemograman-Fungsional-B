mod config;
mod book;
mod search;
mod member;
mod loan;

use axum::{
    extract::{Path, Query, State},
    routing::{delete, get, post},
    Json, Router,
};
use chrono::{NaiveDate, NaiveDateTime, Utc};
use serde::Deserialize;
use sqlx::{MySqlPool, Row};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

use crate::book::{Book, NewBook};
use crate::config::create_pool;
use crate::member::{Member, NewMember};
use crate::loan::{Loan, NewLoan};
use crate::search::{search_books as search_books_fn, SearchMode};

#[derive(Clone)]
struct AppState {
    pool: MySqlPool,
}

async fn health_check() -> &'static str {
    "OK - Sudut Buku backend (DB)"
}

//
// ---------------------- BOOKS ----------------------
//

/// GET /books – ambil semua buku dari tabel `books`.
async fn list_books(State(state): State<AppState>) -> Json<Vec<Book>> {
    let result = sqlx::query_as::<_, Book>(
        "SELECT id, title, author, category, year, total_copies, available_copies FROM books",
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(books) => Json(books),
        Err(e) => {
            eprintln!("DB error on list_books: {e}");
            Json(Vec::new())
        }
    }
}

/// POST /books – insert buku baru ke DB.
async fn create_book(
    State(state): State<AppState>,
    Json(payload): Json<NewBook>,
) -> Json<Book> {
    let result = sqlx::query(
        "INSERT INTO books (title, author, category, year, total_copies, available_copies)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&payload.title)
    .bind(&payload.author)
    .bind(&payload.category)
    .bind(payload.year)
    .bind(payload.total_copies)
    .bind(payload.total_copies) // awalnya stok tersedia = total
    .execute(&state.pool)
    .await;

    match result {
        Ok(res) => {
            let new_id = res.last_insert_id() as i32;
            let fetched = sqlx::query_as::<_, Book>(
                "SELECT id, title, author, category, year, total_copies, available_copies
                 FROM books WHERE id = ?",
            )
            .bind(new_id)
            .fetch_one(&state.pool)
            .await
            .expect("newly inserted book not found");

            Json(fetched)
        }
        Err(e) => {
            eprintln!("DB error on create_book: {e}");
            // fallback minimal
            Json(Book {
                id: -1,
                title: payload.title,
                author: payload.author,
                category: payload.category,
                year: payload.year,
                total_copies: payload.total_copies,
                available_copies: payload.total_copies,
            })
        }
    }
}

/// DELETE /books/:id – hapus baris dari DB.
async fn delete_book(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Json<bool> {
    let result = sqlx::query("DELETE FROM books WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(res) => Json(res.rows_affected() > 0),
        Err(e) => {
            eprintln!("DB error on delete_book: {e}");
            Json(false)
        }
    }
}

//
// ---------------------- SEARCH (PARALLEL) ----------------------
//

/// Query string untuk /search?mode=title&q=rust
#[derive(Deserialize)]
struct SearchParams {
    mode: Option<String>,
    q: String,
}

/// GET /search – ambil semua buku dari DB, lalu FP + parallel search.
async fn search_handler(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Json<Vec<Book>> {
    // 1) Ambil snapshot dari DB (harus konsisten dengan struct Book).
    let books_snapshot = match sqlx::query_as::<_, Book>(
        "SELECT id, title, author, category, year, total_copies, available_copies FROM books",
    )
    .fetch_all(&state.pool)
    .await
    {
        Ok(books) => books,
        Err(e) => {
            eprintln!("DB error on search_handler (load books): {e}");
            Vec::new()
        }
    };

    // 2) Tentukan mode search (default Title).
    let mode = params
        .mode
        .as_deref()
        .and_then(SearchMode::from_str)
        .unwrap_or(SearchMode::Title);

    let query = params.q;

    // 3) Bagi data jadi chunk dan proses paralel.
    let num_cores = num_cpus::get().max(1);
    let len = books_snapshot.len();
    if len == 0 {
        return Json(Vec::new());
    }
    let chunk_size = (len + num_cores - 1) / num_cores;

    let mut tasks = Vec::new();

    for chunk in books_snapshot.chunks(chunk_size) {
        let chunk_vec: Vec<Book> = chunk.to_vec();
        let query_clone = query.clone();
        let mode_copy = mode;

        let handle = tokio::spawn(async move {
            // pure function dari modul search
            search_books_fn(&chunk_vec, mode_copy, &query_clone)
        });

        tasks.push(handle);
    }

    let mut results: Vec<Book> = Vec::new();

    for task in tasks {
        match task.await {
            Ok(mut partial) => results.append(&mut partial),
            Err(e) => eprintln!("Task search gagal: {e}"),
        }
    }

    Json(results)
}

//
// ---------------------- MEMBERS ----------------------
//

/// GET /members – ambil semua anggota.
async fn list_members(State(state): State<AppState>) -> Json<Vec<Member>> {
    let result = sqlx::query_as::<_, Member>(
        "SELECT id, name, email, joined_at FROM members",
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(members) => Json(members),
        Err(e) => {
            eprintln!("DB error on list_members: {e}");
            Json(Vec::new())
        }
    }
}

/// POST /members – buat anggota baru.
async fn create_member(
    State(state): State<AppState>,
    Json(payload): Json<NewMember>,
) -> Json<Member> {
    let result = sqlx::query(
        "INSERT INTO members (name, email) VALUES (?, ?)",
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .execute(&state.pool)
    .await;

    match result {
        Ok(res) => {
            let new_id = res.last_insert_id() as i32;

            // Ambil kembali baris yang baru dibuat untuk mendapatkan joined_at
            let fetched = sqlx::query_as::<_, Member>(
                "SELECT id, name, email, joined_at FROM members WHERE id = ?",
            )
            .bind(new_id)
            .fetch_one(&state.pool)
            .await;

            match fetched {
                Ok(member) => Json(member),
                Err(e) => {
                    eprintln!("DB error on fetch new member: {e}");
                    // fallback kalau gagal fetch – minimal kirim sesuatu
                    Json(Member {
                        id: new_id,
                        name: payload.name,
                        email: payload.email,
                        joined_at: chrono::NaiveDateTime::MIN,
                    })
                }
            }
        }
        Err(e) => {
            eprintln!("DB error on create_member: {e}");
            Json(Member {
                id: -1,
                name: "ERROR".to_string(),
                email: "".to_string(),
                joined_at: chrono::NaiveDateTime::MIN,
            })
        }
    }
}

/// DELETE /members/:id – hapus anggota.
async fn delete_member(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Json<bool> {
    let result = sqlx::query("DELETE FROM members WHERE id = ?")
        .bind(id)
        .execute(&state.pool)
        .await;

    match result {
        Ok(res) => Json(res.rows_affected() > 0),
        Err(e) => {
            eprintln!("DB error on delete_member: {e}");
            Json(false)
        }
    }
}

//
// ---------------------- LOANS ----------------------
//

/// GET /loans – ambil semua peminjaman dari tabel `loans`.
async fn list_loans(State(state): State<AppState>) -> Json<Vec<Loan>> {
    let result = sqlx::query_as::<_, Loan>(
        "SELECT id, book_id, member_id, borrowed_at, due_at, returned_at FROM loans",
    )
    .fetch_all(&state.pool)
    .await;

    match result {
        Ok(loans) => Json(loans),
        Err(e) => {
            eprintln!("DB error on list_loans: {e}");
            Json(Vec::new())
        }
    }
}

/// POST /loans – buat peminjaman baru.
/// Body JSON: { "book_id": 1, "member_id": 1, "due_date": "2025-12-01" }
async fn create_loan(
    State(state): State<AppState>,
    Json(payload): Json<NewLoan>, // book_id, member_id, due_date (YYYY-MM-DD)
) -> Json<Loan> {
    // 0) Parse dan validasi due_date
    let due_date = match NaiveDate::parse_from_str(&payload.due_date, "%Y-%m-%d") {
        Ok(date) => {
            let today = Utc::now().date_naive();
            if date < today {
                eprintln!(
                    "Validation error: due_date {} lebih kecil dari hari ini {}",
                    date, today
                );
                return Json(Loan {
                    id: -1,
                    book_id: payload.book_id,
                    member_id: payload.member_id,
                    borrowed_at: NaiveDateTime::MIN,
                    due_at: NaiveDateTime::MIN,
                    returned_at: None,
                });
            }
            date
        }
        Err(e) => {
            eprintln!(
                "Validation error: gagal parse due_date '{}' : {e}",
                payload.due_date
            );
            return Json(Loan {
                id: -1,
                book_id: payload.book_id,
                member_id: payload.member_id,
                borrowed_at: NaiveDateTime::MIN,
                due_at: NaiveDateTime::MIN,
                returned_at: None,
            });
        }
    };

    let due_at = due_date
        .and_hms_opt(0, 0, 0)
        .unwrap_or(NaiveDateTime::MIN);

    // Mulai transaksi
    let mut tx = state.pool.begin().await.expect("failed to begin tx");

    // 1) Cek stok tersedia
    let row = sqlx::query!(
        "SELECT available_copies FROM books WHERE id = ?",
        payload.book_id
    )
    .fetch_one(&mut *tx)
    .await;

    let available = match row {
        Ok(r) => r.available_copies,
        Err(e) => {
            eprintln!("DB error on select available_copies: {e}");
            tx.rollback().await.ok();
            return Json(Loan {
                id: -1,
                book_id: payload.book_id,
                member_id: payload.member_id,
                borrowed_at: NaiveDateTime::MIN,
                due_at,
                returned_at: None,
            });
        }
    };

    if available <= 0 {
        // stok habis → tolak peminjaman
        tx.rollback().await.ok();
        eprintln!("Stok buku habis untuk book_id={}", payload.book_id);
        return Json(Loan {
            id: -1,
            book_id: payload.book_id,
            member_id: payload.member_id,
            borrowed_at: NaiveDateTime::MIN,
            due_at,
            returned_at: None,
        });
    }

    // 2) Insert ke loans
    let insert_res = sqlx::query(
        "INSERT INTO loans (book_id, member_id, due_at) VALUES (?, ?, ?)",
    )
    .bind(payload.book_id)
    .bind(payload.member_id)
    .bind(due_at)
    .execute(&mut *tx)
    .await
    .expect("failed to insert loan");

    let new_id = insert_res.last_insert_id() as i32;

    // 3) Kurangi stok tersedia
    sqlx::query("UPDATE books SET available_copies = available_copies - 1 WHERE id = ?")
        .bind(payload.book_id)
        .execute(&mut *tx)
        .await
        .expect("failed to update available_copies");

    // 4) Ambil loan yang baru dibuat
    let fetched = sqlx::query_as::<_, Loan>(
        "SELECT id, book_id, member_id, borrowed_at, due_at, returned_at
         FROM loans WHERE id = ?",
    )
    .bind(new_id)
    .fetch_one(&mut *tx)
    .await
    .expect("newly inserted loan not found");

    tx.commit().await.ok();

    Json(fetched)
}

/// POST /loans/:id/return – tandai peminjaman sudah dikembalikan.
async fn return_loan(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Json<bool> {
    let now = Utc::now().naive_utc();

    let mut tx = match state.pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Failed to begin transaction on return_loan: {e}");
            return Json(false);
        }
    };

    // 1. Ambil book_id
    let row = sqlx::query("SELECT book_id FROM loans WHERE id = ?")
        .bind(id)
        .fetch_one(&mut *tx)
        .await;

    let book_id: i32 = match row {
        Ok(r) => r.get("book_id"),
        Err(e) => {
            eprintln!("DB error on select loan book_id: {e}");
            tx.rollback().await.ok();
            return Json(false);
        }
    };

    // 2. Set returned_at
    if let Err(e) = sqlx::query("UPDATE loans SET returned_at = ? WHERE id = ?")
        .bind(now)
        .bind(id)
        .execute(&mut *tx)
        .await
    {
        eprintln!("DB error on update returned_at: {e}");
        tx.rollback().await.ok();
        return Json(false);
    }

    // 3. Tambah stok tersedia
    if let Err(e) = sqlx::query(
        "UPDATE books SET available_copies = available_copies + 1 WHERE id = ?",
    )
    .bind(book_id)
    .execute(&mut *tx)
    .await
    {
        eprintln!("DB error on update available_copies (return): {e}");
        tx.rollback().await.ok();
        return Json(false);
    }

    tx.commit().await.ok();
    Json(true)
}

//
// ---------------------- MAIN ----------------------
//

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let pool = create_pool().await;
    println!("Connected to database");

    let state = AppState { pool };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/books", get(list_books).post(create_book))
        .route("/books/:id", delete(delete_book))
        .route("/members", get(list_members).post(create_member))
        .route("/members/:id", delete(delete_member))
        .route("/loans", get(list_loans).post(create_loan))
        .route("/loans/:id/return", post(return_loan))
        .route("/search", get(search_handler))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("Backend running at http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");

    axum::serve(listener, app)
        .await
        .expect("server error");
}