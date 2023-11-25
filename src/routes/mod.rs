use std::convert::Infallible;

use http_body_util::combinators::BoxBody;
use hyper::body::{Bytes, Incoming};
use hyper::{Error, Request, Response};
use sqlx::PgPool;

use self::api::api_routes;
use self::web::web_routes;

pub mod api;
pub mod web;

pub async fn router(
    req: Request<Incoming>,
    pool: PgPool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    let (method, path) = (req.method().clone(), req.uri().path().to_string());
    if path.starts_with("/api/") {
        api_routes(&method, &path, req, pool).await
    } else {
        web_routes(&method, &path, req, pool).await
    }
}
