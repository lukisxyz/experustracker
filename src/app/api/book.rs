use super::get_book_default_id;
use crate::{
    app::web::common::HandlerResult,
    database::{
        model::book::Book,
        querier::book::{add_owner_by_email, delete, edit, save},
    },
    utils::{serve_empty, serve_full},
};
use cookie::{time::Duration, Cookie};
use http_body_util::BodyExt;
use hyper::{body::Incoming, header::SET_COOKIE, Request, Response, StatusCode};
use sqlx::PgPool;
use std::collections::HashMap;
use ulid::Ulid;

static EMAILS_MISSING: &[u8] =
    b"missing field: email, example: 'example2@gmail.com,example1@gg.com'";
static ID_MISSING: &[u8] = b"missing field: id";
static NAME_MISSING: &[u8] = b"missing field: name";
static DESC_MISSING: &[u8] = b"missing field: description";

pub async fn create_book(req: Request<Incoming>, pool: PgPool, account_id: Ulid) -> HandlerResult {
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let name = if let Some(e) = params.get("name") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(NAME_MISSING))
            .unwrap());
    };
    let description = if let Some(e) = params.get("description") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(DESC_MISSING))
            .unwrap());
    };
    let is_default = if let Some(e) = params.get("set_default") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(NAME_MISSING))
            .unwrap());
    };
    let new_book = Book::new(name, description);
    let new_book_id = new_book.clone().id;
    match save(&pool, account_id, new_book).await {
        Ok(_) => {
            if is_default == "1" {
                let mut c = Cookie::new("book", new_book_id.to_string());
                c.set_max_age(Duration::days(30 * 12));
                c.set_path("/");
                Ok(Response::builder()
                    .status(StatusCode::CREATED)
                    .header("HX-Trigger", "createbookSuccess")
                    .header(SET_COOKIE, c.to_string())
                    .body(serve_full("Success create a book"))
                    .unwrap())
            } else {
                Ok(Response::builder()
                    .status(StatusCode::CREATED)
                    .header("HX-Trigger", "createbookSuccess")
                    .body(serve_full("Success create a book"))
                    .unwrap())
            }
        }
        Err(err) => {
            Ok(Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(serve_full(err.to_string()))
                .unwrap())
        }
    }
}

pub async fn add_book_owner(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
    let book_id: Ulid;
    {
        let header = req.headers();
        let id = get_book_default_id(header);
        book_id = id.await.unwrap();
    }
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let emails = if let Some(e) = params.get("people") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(EMAILS_MISSING))
            .unwrap());
    };
    let email: Vec<&str> = emails.split(',').collect();
    match add_owner_by_email(&pool, book_id, email).await {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::CREATED)
            .header("HX-Trigger", "addBookOwnerSuccess")
            .body(serve_full("Success create a book"))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(err.to_string()))
            .unwrap()),
    }
}

pub async fn edit_book(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let id = if let Some(e) = params.get("id") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(ID_MISSING))
            .unwrap());
    };
    let name = if let Some(e) = params.get("name") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(NAME_MISSING))
            .unwrap());
    };
    let description = if let Some(e) = params.get("description") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(DESC_MISSING))
            .unwrap());
    };
    match edit(
        &pool,
        name.to_string(),
        description.to_string(),
        Ulid::from_string(id).unwrap(),
    )
    .await
    {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("HX-Trigger", "bookChangeSuccess")
            .body(serve_full("Success edit a book"))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(err.to_string()))
            .unwrap()),
    }
}

pub async fn delete_book(req: Request<Incoming>, pool: PgPool, account_id: Ulid) -> HandlerResult {
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let book = if let Some(e) = params.get("book_id") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_empty())
            .unwrap());
    };
    let book_id = Ulid::from_string(book).unwrap().to_bytes();
    match delete(&pool, book_id.into(), account_id).await {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("HX-Trigger", "bookChangeSuccess")
            .body(serve_full("Success delete a book"))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(err.to_string()))
            .unwrap()),
    }
}
