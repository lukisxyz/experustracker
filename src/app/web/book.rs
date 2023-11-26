use sqlx::{FromRow, PgPool};
use ulid::Ulid;

use crate::database::model::book::Book;

pub async fn get_books_by_account_id(id: Ulid, pool: PgPool) -> Vec<Book> {
    match sqlx::query(
        "SELECT b.*
        FROM books b
        JOIN account_books ab ON b.id = ab.book_id
        WHERE ab.account_id = $1
        ORDER BY b.id DESC;
        ",
    )
    .bind(id.to_bytes())
    .fetch_all(&pool)
    .await
    {
        Ok(v) => {
            let mut datas: Vec<Book> = Vec::new();
            for book in v {
                let b = Book::from_row(&book).unwrap();
                datas.push(b)
            }
            return datas;
        }
        Err(_) => todo!(),
    }
}

pub async fn get_books_by_id(id: Ulid, pool: PgPool) -> Option<Book> {
    match sqlx::query(
        "SELECT *
        FROM books
        WHERE id = $1;
        ",
    )
    .bind(id.to_bytes())
    .fetch_one(&pool)
    .await
    {
        Ok(v) => {
            let book = Book::from_row(&v).unwrap();
            Some(book)
        }
        Err(_) => None,
    }
}
