use cookie::Cookie;
use hyper::{body::Incoming, header::COOKIE, HeaderMap, Request};
use sqlx::{FromRow, PgPool};
use ulid::Ulid;

use crate::database::model::session::Session;

pub mod account;
pub mod book;
pub mod category;
pub mod session;

pub async fn get_session_account_id(req: &Request<Incoming>, pool: &PgPool) -> Option<Ulid> {
    let headers = req.headers();
    match headers.get(COOKIE) {
        Some(v) => {
            for cookie in Cookie::split_parse(v.to_str().unwrap()) {
                let cookie = cookie.unwrap();
                if cookie.name() == "session" {
                    return match sqlx::query(
                        "
                            SELECT *
                            FROM sessions
                            WHERE token = $1
                            AND status = TRUE 
                            AND (expire_at > CURRENT_TIMESTAMP OR expire_at IS NULL);
                        ",
                    )
                    .bind(cookie.value())
                    .fetch_one(pool)
                    .await
                    {
                        Ok(s) => match Session::from_row(&s) {
                            Ok(session) => return Some(session.user_id),
                            Err(_) => None,
                        },
                        Err(_) => None,
                    };
                }
            }
            None
        }
        None => None,
    }
}

pub async fn get_book_default_id(h: &HeaderMap) -> Option<Ulid> {
    let cookies = h.get(COOKIE).unwrap();
    for cookie in Cookie::split_parse(cookies.to_str().unwrap()) {
        let cookie = cookie.unwrap();
        if cookie.name() == "book" {
            return Some(Ulid::from_string(cookie.value()).unwrap());
        }
    }
    None
}
