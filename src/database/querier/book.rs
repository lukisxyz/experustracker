use crate::database::model::book::{AccountBook, Book};
use sqlx::{postgres::PgRow, FromRow, PgPool, Postgres, Row};
use sqlx_core::error::BoxDynError;
use ulid::Ulid;

#[derive(Debug)]
struct AccountBookCount {
    book_count: i64,
}

impl<'r> sqlx::FromRow<'r, PgRow> for AccountBookCount {
    fn from_row(row: &'r PgRow) -> Result<Self, sqlx::Error> {
        Ok(Self {
            book_count: row.try_get("book_count")?,
        })
    }
}

pub async fn get_by_account_id(id: Ulid, pool: PgPool) -> Vec<Book> {
    match sqlx::query(
        "SELECT b.*
        FROM books b
        JOIN account_books ab ON b.id = ab.book_id
        WHERE ab.account_id = $1 AND ab.deleted_at IS NULL AND b.deleted_at IS NULL
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
        Err(_) => [].to_vec(),
    }
}

pub async fn get_by_id(pool: PgPool, id: Ulid) -> Option<Book> {
    match sqlx::query(
        "SELECT *
        FROM books
        WHERE id = $1 AND deleted_at IS NULL;
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

pub async fn get_count(pool: &PgPool, id: Ulid) -> i64 {
    match sqlx::query_as::<Postgres, AccountBookCount>(
        "
        SELECT COUNT(book_id) as book_count
        FROM account_books
        WHERE account_id = $1;
    ",
    )
    .bind(id.to_bytes())
    .fetch_one(pool)
    .await
    {
        Ok(b) => b.book_count,
        Err(_) => 0,
    }
}

pub async fn save(pool: &PgPool, account_id: Ulid, b: Book) -> Result<(), BoxDynError> {
    let mut tx = pool.begin().await.unwrap();
    match sqlx::query("INSERT INTO books (id, name, description) VALUES ($1, $2, $3) RETURNING *;")
        .bind(b.id.to_bytes())
        .bind(b.name)
        .bind(b.description)
        .execute(&mut *tx)
        .await
    {
        Ok(_) => {
            match sqlx::query("INSERT INTO account_books (account_id, book_id) VALUES ($1, $2)")
                .bind(account_id.to_bytes())
                .bind(b.id.to_bytes())
                .execute(&mut *tx)
                .await
            {
                Ok(_) => {
                    tx.commit().await.unwrap();
                    Ok(())
                }
                Err(e) => {
                    tx.rollback().await.unwrap();
                    Err(Box::new(e))
                }
            }
        }
        Err(e) => {
            tx.rollback().await.unwrap();
            Err(Box::new(e))
        }
    }
}

pub async fn edit(
    pool: &PgPool,
    name: String,
    desc: String,
    book_id: Ulid,
) -> Result<(), BoxDynError> {
    let mut tx = pool.begin().await.unwrap();
    match sqlx::query(
        "UPDATE books
        SET name = $2, description = $3, updated_at = CURRENT_TIMESTAMP
        WHERE id = $1",
    )
    .bind(book_id.to_bytes())
    .bind(name)
    .bind(desc)
    .execute(&mut *tx)
    .await
    {
        Ok(_) => {
            tx.commit().await.unwrap();
            Ok(())
        }
        Err(e) => {
            tx.rollback().await.unwrap();
            Err(Box::new(e))
        }
    }
}

pub async fn add_owner_by_email(
    pool: &PgPool,
    book_id: Ulid,
    email: Vec<&str>,
) -> Result<(), BoxDynError> {
    let mut tx = pool.begin().await.unwrap();
    match sqlx::query("SELECT id FROM accounts WHERE email = ANY($1)")
        .bind(&email)
        .fetch_all(&mut *tx)
        .await
    {
        Ok(accounts) => {
            let mut datas: Vec<AccountBook> = Vec::new();
            for account in accounts {
                let id: [u8; 16] = account.get("id");
                let id_account = Ulid::from_bytes(id);
                datas.push(AccountBook {
                    account_id: id_account,
                    book_id,
                });
            }
            for record in datas {
                sqlx::query(
                            "INSERT INTO account_books (account_id, book_id) VALUES ($1, $2) ON CONFLICT (account_id, book_id) DO NOTHING"
                        ).bind(&record.account_id.to_bytes())
                        .bind(&record.book_id.to_bytes())
                        .execute(&mut *tx)
                        .await.unwrap();
            }
            tx.commit().await.unwrap();
            Ok(())
        }
        Err(err) => {
            tx.rollback().await.unwrap();
            Err(Box::new(err))
        }
    }
}

pub async fn delete(pool: &PgPool, book_id: Ulid, account_id: Ulid) -> Result<(), BoxDynError> {
    let mut tx = pool.begin().await.unwrap();
    match sqlx::query(
        "UPDATE books
        SET deleted_at = CURRENT_TIMESTAMP
        WHERE id = $1",
    )
    .bind(book_id.to_bytes())
    .execute(&mut *tx)
    .await
    {
        Ok(_) => {
            match sqlx::query(
                "UPDATE account_books
                SET deleted_at = CURRENT_TIMESTAMP
                WHERE account_id = $1
                    AND book_id = $2",
            )
            .bind(account_id.to_bytes())
            .bind(book_id.to_bytes())
            .execute(&mut *tx)
            .await
            {
                Ok(_) => {
                    tx.commit().await.unwrap();
                    Ok(())
                }
                Err(err) => {
                    tx.rollback().await.unwrap();
                    Err(Box::new(err))
                }
            }
        }
        Err(err) => {
            tx.rollback().await.unwrap();
            Err(Box::new(err))
        }
    }
}
