use std::convert::Infallible;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::header::LOCATION;
use hyper::{Error, Method, Request, Response, StatusCode};
use sqlx::PgPool;
use ulid::Ulid;

use crate::app::web::book::get_books_by_id;
use crate::app::web::handler::{
    add_book_owner_page, add_new_book_page, book_lists_page, dashboard_page, edit_book_page, image,
    index_page, login_page, not_found_page, registration_page, string_handler,
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
            if let Some(book) = get_books_by_id(id, pool.clone()).await {
                edit_book_page(req, pool, book).await
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
