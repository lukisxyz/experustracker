use std::collections::HashMap;

use http_body_util::BodyExt;
use hyper::{body::Incoming, header::LOCATION, Request, Response, StatusCode};
use sqlx::PgPool;

use crate::{
    app::web::handler::HandlerResult,
    database::model::book::Book,
    utils::{serve_empty, serve_full},
};

use super::get_session_account_id;

static NAME_MISSING: &[u8] = b"Missing field: Name";

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

            let new_book = Book::new(&name);

            let mut tx = pool.begin().await.unwrap();

            match sqlx::query("INSERT INTO books (id, name) VALUES ($1, $2) RETURNING *;")
                .bind(new_book.id.to_bytes())
                .bind(new_book.name)
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
                            return Ok(Response::builder()
                                .status(StatusCode::CREATED)
                                .header("HX-Trigger", "createbookSuccess")
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
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "/login")
            .body(serve_empty())
            .unwrap()),
    }
}
