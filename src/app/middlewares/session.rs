use crate::{
    app::web::handler::HandlerResult, database::model::session::Session, utils::serve_empty,
};
use cookie::Cookie;
use hyper::{
    body::Incoming,
    header::{COOKIE, LOCATION},
    Request, Response, StatusCode,
};
use sqlx::{FromRow, PgPool};
use std::future::Future;
use ulid::Ulid;

pub async fn auth_middleware<F, Fut>(req: Request<Incoming>, pool: PgPool, next: F) -> HandlerResult
where
    F: Fn(Request<Incoming>, PgPool, Ulid) -> Fut,
    Fut: Future<Output = HandlerResult>,
{
    let headers = req.headers();
    match headers.get(COOKIE) {
        Some(h) => {
            for cookie in Cookie::split_parse(h.to_str().unwrap()) {
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
                    .fetch_one(&pool)
                    .await
                    {
                        Ok(s) => match Session::from_row(&s) {
                            Ok(session) => next(req, pool, session.user_id).await,
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
                    };
                }
            }
            Ok(Response::builder()
                .status(StatusCode::TEMPORARY_REDIRECT)
                .header(LOCATION, "/login")
                .body(serve_empty())
                .unwrap())
        }
        None => Ok(Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "/login")
            .body(serve_empty())
            .unwrap()),
    }
}
