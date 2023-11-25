use cookie::Cookie;
use hyper::{
    body::Incoming,
    header::{COOKIE, LOCATION},
    Request, Response, StatusCode,
};
use sqlx::PgPool;

use crate::utils::serve_empty;

use self::handler::HandlerResult;

pub mod handler;
pub mod templates;

pub async fn middleware_auth(
    req: Request<Incoming>,
    pool: PgPool,
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
            .fetch_one(&pool)
            .await
            {
                Ok(_) => f,
                Err(_) => Ok(Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header(LOCATION, "/login")
                    .body(serve_empty())
                    .unwrap()),
            }
        }
        None => todo!(),
    }
}
