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

use crate::{
    app::{
        api::get_session_account_id,
        web::{
            book::get_books_by_account_id,
            check_atleast_one_book, middleware_auth,
            templates::{
                AddBookOwnerTemplate, AddNewBookTemplate, BookListsBookTemplate, DashboardTemplate,
                EditBookTemplate,
            },
        },
    },
    database::model::book::Book,
    utils::serve_empty,
};

use super::templates::{IndexTemplate, LoginTemplate, NotFoundTemplate, RegisterTemplate};
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
    let file_name = path_buf.file_name().unwrap().to_str().unwrap();
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
                let xml = match file_name {
                    // "dev-badge.svg" => include_str!("assets/svg/dev-badge.svg"), // for example
                    _ => "",
                };
                string_handler(xml, "image/svg+xml", None).await
            }
            _ => not_found_page().await,
        }
    } else {
        not_found_page().await
    }
}

pub async fn index_page() -> HandlerResult {
    let template = IndexTemplate::default();
    let html = template.render().expect("Should render markup");
    html_str_handler(&html).await
}

pub async fn registration_page() -> HandlerResult {
    let template = RegisterTemplate::default();
    let html = template.render().expect("Should render markup");
    html_str_handler(&html).await
}

pub async fn login_page() -> HandlerResult {
    let template = LoginTemplate::default();
    let html = template.render().expect("Should render markup");
    html_str_handler(&html).await
}

pub async fn not_found_page() -> HandlerResult {
    let template = NotFoundTemplate::default();
    let html = template.render().expect("Should render markup");
    string_handler(&html, "text/html", Some(StatusCode::NOT_FOUND)).await
}

pub async fn dashboard_page(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    async fn f() -> HandlerResult {
        let template = DashboardTemplate::default();
        let html = template.render().expect("Should render markup");
        return html_str_handler(&html).await;
    }

    async fn f2(req: &Request<Incoming>, pool: &PgPool) -> HandlerResult {
        check_atleast_one_book(&req, &pool, f().await).await
    }

    middleware_auth(&req, &pool, f2(&req, &pool).await).await
}

pub async fn add_new_book_page(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    async fn f() -> HandlerResult {
        let template = AddNewBookTemplate::default();
        let html = template.render().expect("Should render markup");
        return html_str_handler(&html).await;
    }

    middleware_auth(&req, &pool, f().await).await
}

pub async fn add_book_owner_page(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    async fn f() -> HandlerResult {
        let template = AddBookOwnerTemplate::default();
        let html = template.render().expect("Should render markup");
        return html_str_handler(&html).await;
    }

    middleware_auth(&req, &pool, f().await).await
}

pub async fn book_lists_page(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    if let Some(id) = get_session_account_id(&req, &pool).await {
        let datas = get_books_by_account_id(id, pool).await;
        let template = BookListsBookTemplate { books: &datas };
        let html = template.render().expect("Should render markup");
        return html_str_handler(&html).await;
    } else {
        Ok(Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "/login")
            .body(serve_empty())
            .unwrap())
    }
}

pub async fn edit_book_page(req: Request<Incoming>, pool: PgPool, book: Book) -> HandlerResult {
    async fn f(book: Book) -> HandlerResult {
        let template = EditBookTemplate {
            id: &book.id.to_string(),
            name: &book.name,
            description: &book.description,
        };
        let html = template.render().expect("Should render markup");
        return html_str_handler(&html).await;
    }

    middleware_auth(&req, &pool, f(book).await).await
}
