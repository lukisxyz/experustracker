use std::convert::Infallible;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Error, Method, Request, Response, StatusCode};
use sqlx::PgPool;

use crate::app::api::account::{create_new_account, validate_email, validate_password};
use crate::app::api::session::{login_account, logout_account};
use crate::utils;

pub async fn api_routes(
    method: &Method,
    path: &str,
    req: Request<Incoming>,
    pool: PgPool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    match (method, path) {
        (&Method::POST, "/api/logout") => logout_account(req, pool).await,
        (&Method::POST, "/api/login") => login_account(req, pool).await,
        (&Method::POST, "/api/register") => create_new_account(req, pool).await,
        (&Method::POST, "/api/register/validate-email") => validate_email(req, pool).await,
        (&Method::POST, "/api/register/validate-password") => validate_password(req).await,
        _ => {
            let mut not_found = Response::new(utils::serve_empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
