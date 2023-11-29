use crate::app::middlewares::params::id_params_middleware;
use crate::app::middlewares::session::auth_middleware;
use crate::app::web::book::{page_book_add_owner, page_book_create, page_book_edit, page_books};
use crate::app::web::category::{page_categories, page_category_create, page_category_edit};
use crate::app::web::common::{
    image, page_dashboard, page_index, page_not_found, page_signin, page_signup, string_handler,
};
use crate::app::web::record::{page_record_create, page_record_edit, page_records};
use crate::utils::serve_empty;
use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Error, Method, Request, Response, StatusCode};
use sqlx::PgPool;
use std::convert::Infallible;
use ulid::Ulid;

pub async fn web_routes(
    method: &Method,
    path: &str,
    req: Request<Incoming>,
    pool: PgPool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    match (method, path) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => page_index().await,
        (&Method::GET, "/register") | (&Method::GET, "/register.html") => page_signup().await,
        (&Method::GET, "/login") | (&Method::GET, "/login.html") => page_signin().await,

        // book routes
        (&Method::GET, "/book") => auth_middleware(req, pool, page_books).await,
        (&Method::GET, "/book/create") => auth_middleware(req, pool, page_book_create).await,
        (&Method::GET, path) if path.starts_with("/book/edit/") => {
            let p = path;
            let run = move |req: Request<Incoming>, pool: PgPool, _: Ulid| async move {
                id_params_middleware(
                    req,
                    pool,
                    11,
                    "/book".to_string(),
                    p.to_owned(),
                    page_book_edit,
                )
                .await
            };

            auth_middleware(req, pool, run).await
        }
        (&Method::GET, "/book/add-owner") | (&Method::GET, "/book/add-book-owner.html") => {
            page_book_add_owner(req, pool).await
        }

        // category routes
        (&Method::GET, "/category") => auth_middleware(req, pool, page_categories).await,
        (&Method::GET, "/category/create") => {
            auth_middleware(req, pool, page_category_create).await
        }
        (&Method::GET, path) if path.starts_with("/category/edit/") => {
            let p = path;
            let run = move |req: Request<Incoming>, pool: PgPool, _: Ulid| async move {
                id_params_middleware(
                    req,
                    pool,
                    15,
                    "/category".to_string(),
                    p.to_owned(),
                    page_category_edit,
                )
                .await
            };

            auth_middleware(req, pool, run).await
        }

        (&Method::GET, "/record") => auth_middleware(req, pool, page_records).await,
        (&Method::GET, "/record/create") => auth_middleware(req, pool, page_record_create).await,
        (&Method::GET, path) if path.starts_with("/record/edit/") => {
            let p = path;
            let run = move |req: Request<Incoming>, pool: PgPool, _: Ulid| async move {
                id_params_middleware(
                    req,
                    pool,
                    15,
                    "/category".to_string(),
                    p.to_owned(),
                    page_record_edit,
                )
                .await
            };

            auth_middleware(req, pool, run).await
        }
        (&Method::GET, "/dashboard") | (&Method::GET, "/dashboard.html") => {
            page_dashboard(req, pool).await
        }
        (&Method::GET, "/main.css") => {
            string_handler(include_str!("../assets/main.css"), "text/css", None).await
        }
        (&Method::GET, "/manifest.json") => {
            string_handler(include_str!("../assets/manifest.json"), "text/json", None).await
        }
        (&Method::GET, "/htmx.min.js") => {
            string_handler(include_str!("../assets/htmx.min.js"), "text", None).await
        }
        (&Method::GET, "/robots.txt") => {
            string_handler(include_str!("../assets/robots.txt"), "text", None).await
        }
        (&Method::GET, path_str) => {
            // Otherwise...
            // is it an image?
            if let Some(ext) = path_str.split('.').nth(1) {
                match ext {
                    "ico" | "svg" => image(path).await,
                    _ => page_not_found().await,
                }
            } else {
                page_not_found().await
            }
        }
        _ => {
            let mut not_found = Response::new(serve_empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
