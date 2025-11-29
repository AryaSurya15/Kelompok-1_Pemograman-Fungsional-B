use crate::book::Book;

/// Mode pencarian yang didukung.
#[derive(Debug, Clone, Copy)]
pub enum SearchMode {
    Title,
    Author,
    Category,
}

impl SearchMode {
    /// Konversi dari string query (?mode=title/author/category) ke enum.
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "title" => Some(Self::Title),
            "author" => Some(Self::Author),
            "category" => Some(Self::Category),
            _ => None,
        }
    }
}

/// Pure function: tidak mengubah input, tidak mengakses IO.
/// Hanya mem-filter slice books berdasarkan mode & query.
pub fn search_books(books: &[Book], mode: SearchMode, query: &str) -> Vec<Book> {
    let q = query.to_lowercase();

    books
        .iter()
        .filter(|book| {
            let field = match mode {
                SearchMode::Title => &book.title,
                SearchMode::Author => &book.author,
                SearchMode::Category => &book.category,
            };

            field.to_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}
