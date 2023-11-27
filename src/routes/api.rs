use std::convert::Infallible;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::header::LOCATION;
use hyper::{Error, Method, Request, Response, StatusCode};
use sqlx::PgPool;

use crate::app::api::account::{create_new_account, validate_email, validate_password};
use crate::app::api::book::{add_book_owner, create_book, delete_book, edit_book};
use crate::app::api::get_session_account_id;
use crate::app::api::session::{login_account, logout_account};
use crate::utils::{self, serve_empty};

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

        (&Method::POST, "/api/book") => create_book(req, pool).await,
        (&Method::DELETE, "/api/book") => {
            if let Some(account_id) = get_session_account_id(&req, &pool).await {
                delete_book(req, pool, account_id).await
            } else {
                Ok(Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header(LOCATION, "/login")
                    .body(serve_empty())
                    .unwrap())
            }
        }
        (&Method::PATCH, "/api/book") => edit_book(req, pool).await,
        (&Method::POST, "/api/book/add-owner") => add_book_owner(req, pool).await,
        _ => {
            let mut not_found = Response::new(utils::serve_empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
