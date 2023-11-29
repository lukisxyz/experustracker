use crate::app::api::account::{create_account, validate_email, validate_password};
use crate::app::api::book::{add_book_owner, create_book, delete_book, edit_book};
use crate::app::api::category::{create_category, delete_category, edit_category};
use crate::app::api::get_session_account_id;
use crate::app::api::record::{create_record, delete_record, edit_record};
use crate::app::api::session::{login_account, logout_account};
use crate::app::middlewares::session::auth_middleware;
use crate::utils::{self, serve_empty};
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::header::LOCATION;
use hyper::{Error, Method, Request, Response, StatusCode};
use sqlx::PgPool;
use std::convert::Infallible;

pub async fn api_routes(
    method: &Method,
    path: &str,
    req: Request<Incoming>,
    pool: PgPool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    match (method, path) {
        (&Method::POST, "/api/logout") => logout_account(req, pool).await,
        (&Method::POST, "/api/login") => login_account(req, pool).await,
        (&Method::POST, "/api/register") => create_account(req, pool).await,
        (&Method::POST, "/api/register/validate-email") => validate_email(req, pool).await,
        (&Method::POST, "/api/register/validate-password") => validate_password(req).await,
        (&Method::POST, "/api/book") => auth_middleware(req, pool, create_book).await,
        (&Method::PATCH, "/api/book") => auth_middleware(req, pool, edit_book).await,

        //TODO
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
        (&Method::POST, "/api/book/add-owner") => add_book_owner(req, pool).await,
        (&Method::POST, "/api/category") => create_category(req, pool).await,
        (&Method::DELETE, "/api/category") => {
            if let Some(_) = get_session_account_id(&req, &pool).await {
                delete_category(req, pool).await
            } else {
                Ok(Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header(LOCATION, "/login")
                    .body(serve_empty())
                    .unwrap())
            }
        }
        (&Method::PATCH, "/api/category") => edit_category(req, pool).await,
        (&Method::POST, "/api/record") => create_record(req, pool).await,
        (&Method::PATCH, "/api/record") => edit_record(req, pool).await,
        (&Method::DELETE, "/api/record") => delete_record(req, pool).await,
        _ => {
            let mut not_found = Response::new(utils::serve_empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
