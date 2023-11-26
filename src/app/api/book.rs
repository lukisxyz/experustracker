use std::collections::HashMap;

use cookie::{time::Duration, Cookie};
use http_body_util::BodyExt;
use hyper::{body::Incoming, header::SET_COOKIE, Request, Response, StatusCode};
use sqlx::{PgPool, Row};
use ulid::Ulid;

use crate::{
    app::web::handler::HandlerResult,
    database::model::book::{AccountBook, Book},
    utils::{serve_empty, serve_full},
};

use super::{get_book_default_id, get_session_account_id};

static EMAILS_MISSING: &[u8] =
    b"missing field: email, example: 'example2@gmail.com,example1@gg.com'";
static NAME_MISSING: &[u8] = b"missing field: name";
static DESC_MISSING: &[u8] = b"missing field: description";

pub async fn create_book(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    match get_session_account_id(&req, &pool).await {
        Some(v) => {
            let body = req.collect().await?.to_bytes();
            let params = form_urlencoded::parse(body.as_ref())
                .into_owned()
                .collect::<HashMap<String, String>>();
            let name = if let Some(e) = params.get("name") {
                e
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(serve_full(NAME_MISSING))
                    .unwrap());
            };

            let description = if let Some(e) = params.get("description") {
                e
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(serve_full(DESC_MISSING))
                    .unwrap());
            };

            let new_book = Book::new(&name, &description);

            let mut tx = pool.begin().await.unwrap();

            match sqlx::query(
                "INSERT INTO books (id, name, description) VALUES ($1, $2, $3) RETURNING *;",
            )
            .bind(new_book.id.to_bytes())
            .bind(new_book.name)
            .bind(new_book.description)
            .execute(&mut *tx)
            .await
            {
                Ok(_) => {
                    match sqlx::query(
                        "INSERT INTO account_books (account_id, book_id) VALUES ($1, $2)",
                    )
                    .bind(v.to_bytes())
                    .bind(new_book.id.to_bytes())
                    .execute(&mut *tx)
                    .await
                    {
                        Ok(_) => {
                            tx.commit().await.unwrap();
                            let mut c = Cookie::new("book", new_book.id.to_string());
                            c.set_http_only(true);
                            c.set_max_age(Duration::days(30 * 12));
                            c.set_path("/");
                            c.set_secure(true);
                            return Ok(Response::builder()
                                .status(StatusCode::CREATED)
                                .header("HX-Trigger", "createbookSuccess")
                                .header(SET_COOKIE, c.to_string())
                                .body(serve_full("Success create a book"))
                                .unwrap());
                        }
                        Err(err) => {
                            tx.rollback().await.unwrap();
                            return Ok(Response::builder()
                                .status(StatusCode::UNPROCESSABLE_ENTITY)
                                .body(serve_full(err.to_string()))
                                .unwrap());
                        }
                    }
                }
                Err(err) => {
                    tx.rollback().await.unwrap();
                    return Ok(Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(serve_full(err.to_string()))
                        .unwrap());
                }
            }
        }
        None => Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(serve_empty())
            .unwrap()),
    }
}

pub async fn add_book_owner(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    match get_session_account_id(&req, &pool).await {
        Some(_) => {
            let book_id: Ulid;
            {
                let header = req.headers();
                let id = get_book_default_id(&header);
                book_id = id.await.unwrap();
            }
            let body = req.collect().await?.to_bytes();
            let params = form_urlencoded::parse(body.as_ref())
                .into_owned()
                .collect::<HashMap<String, String>>();
            let emails = if let Some(e) = params.get("people") {
                e
            } else {
                return Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(serve_full(EMAILS_MISSING))
                    .unwrap());
            };
            let mut tx = pool.begin().await.unwrap();

            let parts: Vec<&str> = emails.split(',').collect();

            match sqlx::query("SELECT id FROM accounts WHERE email = ANY($1)")
                .bind(&parts)
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
                    return Ok(Response::builder()
                        .status(StatusCode::CREATED)
                        .header("HX-Trigger", "addBookOwnerSuccess")
                        .body(serve_full("Success create a book"))
                        .unwrap());
                }
                Err(err) => {
                    tx.rollback().await.unwrap();
                    return Ok(Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(serve_full(err.to_string()))
                        .unwrap());
                }
            }
        }
        None => Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(serve_empty())
            .unwrap()),
    }
}