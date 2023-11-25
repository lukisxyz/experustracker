use std::convert::Infallible;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Error, Method, Request, Response, StatusCode};
use sqlx::PgPool;

use crate::app::handlers::account::{validate_email, validate_password};
use crate::utils;

pub async fn api_router(
    req: Request<Incoming>,
    pool: PgPool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    let (method, path) = (req.method(), req.uri().path());
    match (method, path) {
        (&Method::POST, "/api/register/validate-email") => validate_email(req, pool).await,
        (&Method::POST, "/api/register/validate-password") => validate_password(req).await,
        _ => {
            let mut not_found = Response::new(utils::serve_empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
