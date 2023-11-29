use crate::{app::web::common::HandlerResult, utils::serve_empty};
use hyper::{body::Incoming, header::LOCATION, Request, Response, StatusCode};
use sqlx::PgPool;
use std::future::Future;
use ulid::Ulid;

pub async fn id_params_middleware<F, Fut>(
    req: Request<Incoming>,
    pool: PgPool,
    cursor: usize,
    fail: String,
    path: String,
    next: F,
) -> HandlerResult
where
    F: Fn(Request<Incoming>, PgPool, Ulid) -> Fut,
    Fut: Future<Output = HandlerResult>,
{
    let id_str = &path[cursor..];
    match Ulid::from_string(id_str) {
        Ok(id) => next(req, pool, id).await,
        Err(_) => Ok(Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, fail)
            .body(serve_empty())
            .unwrap()),
    }
}
