use crate::app::api::account::{create_account, validate_email, validate_password};
use crate::app::api::book::{add_book_owner, create_book, delete_book, edit_book};
use crate::app::api::category::{create_category, delete_category, edit_category};
use crate::app::api::record::{create_record, delete_record, edit_record};
use crate::app::api::session::{login_account, logout_account};
use crate::app::middlewares::session::auth_middleware;
use crate::utils::serve_empty;
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
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
        (&Method::DELETE, "/api/book") => auth_middleware(req, pool, delete_book).await,
        (&Method::POST, "/api/book/add-owner") => auth_middleware(req, pool, add_book_owner).await,

        (&Method::POST, "/api/category") => auth_middleware(req, pool, create_category).await,
        (&Method::DELETE, "/api/category") => auth_middleware(req, pool, delete_category).await,
        (&Method::PATCH, "/api/category") => auth_middleware(req, pool, edit_category).await,
        (&Method::POST, "/api/record") => create_record(req, pool).await,
        (&Method::PATCH, "/api/record") => edit_record(req, pool).await,
        (&Method::DELETE, "/api/record") => delete_record(req, pool).await,
        _ => {
            let mut not_found = Response::new(serve_empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
