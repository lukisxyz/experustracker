use std::convert::Infallible;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Error, Method, Request, Response, StatusCode};
use sqlx::PgPool;

use crate::app::web::handler::{
    image, index_page, login_page, not_found_page, protected_page, registration_page,
    string_handler,
};
use crate::utils::serve_empty;

pub async fn web_routes(
    method: &Method,
    path: &str,
    _req: Request<Incoming>,
    _pool: PgPool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    match (method, path) {
        (&Method::GET, "/") | (&Method::GET, "/index.html") => index_page().await,
        (&Method::GET, "/register") | (&Method::GET, "/register.html") => registration_page().await,
        (&Method::GET, "/login") | (&Method::GET, "/login.html") => login_page().await,
        (&Method::GET, "/protected") | (&Method::GET, "/protected.html") => {
            protected_page(_req, _pool).await
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
