use super::templates::{IndexTemplate, LoginTemplate, NotFoundTemplate, RegisterTemplate};
use crate::{
    app::web::{middleware_auth, templates::DashboardTemplate},
    database::querier::book::get_count,
    utils::serve_empty,
};
use askama::Template;
use flate2::{write::ZlibEncoder, Compression};
use http_body_util::{combinators::BoxBody, BodyExt, Full};
use hyper::{
    body::{Bytes, Incoming},
    header::{CONTENT_ENCODING, CONTENT_TYPE, LOCATION},
    Error, Request, Response, StatusCode,
};
use sqlx::PgPool;
use std::{convert::Infallible, fs::File, io::prelude::*, path::PathBuf};
pub type HandlerResult = Result<Response<BoxBody<Bytes, Infallible>>, Error>;

fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, Infallible> {
    Full::new(chunk.into()).boxed()
}

pub async fn bytes_handler(
    body: &[u8],
    content_type: &str,
    status: Option<StatusCode>,
) -> HandlerResult {
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(body).unwrap();
    let compressed = e.finish().unwrap();
    Ok(Response::builder()
        .status(status.unwrap_or_default())
        .header(CONTENT_TYPE, content_type)
        .header(CONTENT_ENCODING, "deflate")
        .body(full(compressed))
        .unwrap())
}

pub async fn string_handler(
    body: &str,
    content_type: &str,
    status: Option<StatusCode>,
) -> HandlerResult {
    bytes_handler(body.as_bytes(), content_type, status).await
}

pub async fn html_str_handler(body: &str) -> HandlerResult {
    string_handler(body, "text/html", None).await
}

pub async fn image(path_str: &str) -> HandlerResult {
    let path_buf = PathBuf::from(path_str);
    let _file_name = path_buf.file_name().unwrap().to_str().unwrap();
    if let Some(ext) = path_buf.extension() {
        match ext.to_str().unwrap() {
            "ico" => {
                let mut file =
                    File::open("src/assets/images/favicon.ico").expect("Should open icon file");
                let mut buf = Vec::new();
                file.read_to_end(&mut buf).expect("Should read icon file");
                bytes_handler(&buf, "image/x-icon", None).await
            }
            "svg" => {
                // build the response
                let xml = "";
                string_handler(xml, "image/svg+xml", None).await
            }
            _ => page_not_found().await,
        }
    } else {
        page_not_found().await
    }
}

pub async fn page_index() -> HandlerResult {
    let template = IndexTemplate::default();
    let html = template.render().expect("Should render markup");
    html_str_handler(&html).await
}

pub async fn page_signup() -> HandlerResult {
    let template = RegisterTemplate::default();
    let html = template.render().expect("Should render markup");
    html_str_handler(&html).await
}

pub async fn page_signin() -> HandlerResult {
    let template = LoginTemplate::default();
    let html = template.render().expect("Should render markup");
    html_str_handler(&html).await
}

pub async fn page_not_found() -> HandlerResult {
    let template = NotFoundTemplate::default();
    let html = template.render().expect("Should render markup");
    string_handler(&html, "text/html", Some(StatusCode::NOT_FOUND)).await
}

pub async fn page_dashboard(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    if let Some(id) = middleware_auth(&req, &pool).await {
        if get_count(&pool, id).await == 0 {
            Ok(Response::builder()
                .status(StatusCode::SEE_OTHER)
                .header(LOCATION, "/book/create")
                .body(serve_empty())
                .unwrap())
        } else {
            let template = DashboardTemplate::default();
            let html = template.render().expect("Should render markup");
            html_str_handler(&html).await
        }
    } else {
        Ok(Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "/login")
            .body(serve_empty())
            .unwrap())
    }
}
