use std::convert::Infallible;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::header::LOCATION;
use hyper::{Error, Method, Request, Response, StatusCode};
use sqlx::PgPool;
use ulid::Ulid;

use crate::app::web::book::{
    add_book_owner_page, add_new_book_page, book_lists_page, edit_book_page, get_book_by_id,
};
use crate::app::web::category::{
    add_new_category_page, category_lists_page, edit_category_page, get_category_by_id,
};
use crate::app::web::handler::{
    dashboard_page, image, index_page, login_page, not_found_page, registration_page,
    string_handler,
};
use crate::app::web::record::{
    add_new_record_page, edit_record_page, get_record_by_id, record_lists_page,
};
use crate::utils::serve_empty;

pub async fn web_routes(
    method: &Method,
    path: &str,
    req: Request<Incoming>,
    pool: PgPool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    match (method, path) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => index_page().await,
        (&Method::GET, "/register") | (&Method::GET, "/register.html") => registration_page().await,
        (&Method::GET, "/login") | (&Method::GET, "/login.html") => login_page().await,
        (&Method::GET, "/book") => book_lists_page(req, pool).await,
        (&Method::GET, "/book/create") => add_new_book_page(req, pool).await,
        (&Method::GET, path) if path.starts_with("/book/edit/") => {
            let id_str = &path[11..];
            let id = Ulid::from_string(id_str).unwrap();
            if let Some(book) = get_book_by_id(id, pool.clone()).await {
                edit_book_page(req, pool, book).await
            } else {
                Ok(Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header(LOCATION, "/book")
                    .body(serve_empty())
                    .unwrap())
            }
        }
        (&Method::GET, "/category") => category_lists_page(req, pool).await,
        (&Method::GET, "/category/create") => add_new_category_page(req, pool).await,
        (&Method::GET, path) if path.starts_with("/category/edit/") => {
            let id_str = &path[15..];
            let id = Ulid::from_string(id_str).unwrap();
            if let Some(category) = get_category_by_id(id, pool.clone()).await {
                edit_category_page(req, pool, category).await
            } else {
                Ok(Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header(LOCATION, "/book")
                    .body(serve_empty())
                    .unwrap())
            }
        }
        (&Method::GET, "/book/add-owner") | (&Method::GET, "/book/add-book-owner.html") => {
            add_book_owner_page(req, pool).await
        }

        (&Method::GET, "/record") => record_lists_page(req, pool).await,
        (&Method::GET, "/record/create") => add_new_record_page(req, pool).await,
        (&Method::GET, path) if path.starts_with("/record/edit/") => {
            let id_str = &path[13..];
            let id = Ulid::from_string(id_str).unwrap();
            if let Some(record) = get_record_by_id(id, pool.clone()).await {
                edit_record_page(req, pool, record).await
            } else {
                Ok(Response::builder()
                    .status(StatusCode::TEMPORARY_REDIRECT)
                    .header(LOCATION, "/book")
                    .body(serve_empty())
                    .unwrap())
            }
        }
        (&Method::GET, "/dashboard") | (&Method::GET, "/dashboard.html") => {
            dashboard_page(req, pool).await
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
                    _ => not_found_page().await,
                }
            } else {
                not_found_page().await
            }
        }
        _ => {
            let mut not_found = Response::new(serve_empty());
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}
