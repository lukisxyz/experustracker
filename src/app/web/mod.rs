use cookie::Cookie;
use hyper::{
    body::Incoming,
    header::{COOKIE, LOCATION},
    Request, Response, StatusCode,
};
use sqlx::{postgres::PgRow, FromRow, PgPool, Postgres, Row};

use crate::{database::model::session::Session, utils::serve_empty};

use self::handler::HandlerResult;

pub mod book;
pub mod handler;
pub mod templates;

pub async fn middleware_auth(
    req: &Request<Incoming>,
    pool: &PgPool,
    f: HandlerResult,
) -> HandlerResult {
    let headers = req.headers();
    match headers.get(COOKIE) {
        Some(v) => {
            let c = Cookie::parse(v.to_str().unwrap()).unwrap();
            match sqlx::query(
                "
                SELECT *
                FROM sessions
                WHERE token = $1
                AND status = TRUE 
                AND (expire_at > CURRENT_TIMESTAMP OR expire_at IS NULL);
            ",
            )
            .bind(c.value())
            .fetch_one(pool)
            .await
            {
                Ok(s) => match Session::from_row(&s) {
                    Ok(_) => f,
                    Err(_) => Ok(Response::builder()
                        .status(StatusCode::TEMPORARY_REDIRECT)
                        .header(LOCATION, "/login")
                        .body(serve_empty())
                        .unwrap()),
                },
                Err(_) => Ok(Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header(LOCATION, "/login")
                    .body(serve_empty())
                    .unwrap()),
            }
        }
        None => Ok(Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "/login")
            .body(serve_empty())
            .unwrap()),
    }
}

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

pub async fn check_atleast_one_book(
    req: &Request<Incoming>,
    pool: &PgPool,
    f: HandlerResult,
) -> HandlerResult {
    let headers = req.headers();
    match headers.get(COOKIE) {
        Some(v) => {
            let c = Cookie::parse(v.to_str().unwrap()).unwrap();
            match sqlx::query(
                "
                SELECT *
                FROM sessions
                WHERE token = $1
                AND status = TRUE 
                AND (expire_at > CURRENT_TIMESTAMP OR expire_at IS NULL);
            ",
            )
            .bind(c.value())
            .fetch_one(pool)
            .await
            {
                Ok(s) => match Session::from_row(&s) {
                    Ok(session) => {
                        match sqlx::query_as::<Postgres, AccountBookCount>(
                            "
                            SELECT a.id AS account_id, COUNT(b.id) AS book_count
                            FROM accounts a
                            LEFT JOIN account_books ab ON a.id = ab.account_id
                            LEFT JOIN books b ON ab.book_id = b.id
                            WHERE a.id = $1
                            GROUP BY a.id;
                        ",
                        )
                        .bind(session.user_id.to_bytes())
                        .fetch_one(pool)
                        .await
                        {
                            Ok(b) => {
                                if b.book_count > 0 {
                                    f
                                } else {
                                    Ok(Response::builder()
                                        .status(StatusCode::SEE_OTHER)
                                        .header(LOCATION, "/book/create")
                                        .body(serve_empty())
                                        .unwrap())
                                }
                            }
                            Err(_) => Ok(Response::builder()
                                .status(StatusCode::SEE_OTHER)
                                .header(LOCATION, "/book/create")
                                .body(serve_empty())
                                .unwrap()),
                        }
                    }
                    Err(_) => Ok(Response::builder()
                        .status(StatusCode::TEMPORARY_REDIRECT)
                        .header(LOCATION, "/login")
                        .body(serve_empty())
                        .unwrap()),
                },
                Err(_) => Ok(Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header(LOCATION, "/login")
                    .body(serve_empty())
                    .unwrap()),
            }
        }
        None => Ok(Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "/login")
            .body(serve_empty())
            .unwrap()),
    }
}
