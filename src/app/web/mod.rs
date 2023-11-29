use crate::database::model::session::Session;
use cookie::Cookie;
use hyper::{body::Incoming, header::COOKIE, Request};
use sqlx::{FromRow, PgPool};
use ulid::Ulid;
pub mod book;
pub mod category;
pub mod common;
pub mod record;
pub mod templates;

pub async fn middleware_auth(req: &Request<Incoming>, pool: &PgPool) -> Option<Ulid> {
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
                    Ok(session) => Some(session.user_id),
                    Err(_) => None,
                },
                Err(_) => None,
            }
        }
        None => None,
    }
}
