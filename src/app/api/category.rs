use std::collections::HashMap;

use http_body_util::BodyExt;
use hyper::{body::Incoming, Request, Response, StatusCode};
use sqlx::PgPool;
use ulid::Ulid;

use crate::{
    app::web::handler::HandlerResult,
    database::{
        model::category::Category,
        querier::category::{delete, edit, save},
    },
    utils::{serve_empty, serve_full},
};

static NAME_MISSING: &[u8] = b"missing field: name";
static DESC_MISSING: &[u8] = b"missing field: description";

pub async fn create_category(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
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
    let book = if let Some(e) = params.get("book_id") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_empty())
            .unwrap());
    };
    let book_id = Ulid::from_string(&book).unwrap().to_bytes();
    let category_type = if let Some(e) = params.get("type") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(NAME_MISSING))
            .unwrap());
    };
    let is_expense: bool = category_type == "expense";
    let new_category = Category::new(&name, &description, is_expense, Ulid::from_bytes(book_id));
    match save(&pool, new_category).await {
        Ok(_) => {
            return Ok(Response::builder()
                .status(StatusCode::CREATED)
                .header("HX-Trigger", "createcategorySuccess")
                .body(serve_full("Success create a category"))
                .unwrap());
        }
        Err(err) => {
            return Ok(Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(serve_full(err.to_string()))
                .unwrap());
        }
    }
}

pub async fn edit_category(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
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
    let category_id_str = if let Some(e) = params.get("category_id") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_empty())
            .unwrap());
    };
    match edit(
        &pool,
        name.to_string(),
        description.to_string(),
        Ulid::from_string(&category_id_str).unwrap(),
    )
    .await
    {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("HX-Trigger", "categoryChangeSuccess")
            .body(serve_full("Success edit a category"))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(err.to_string()))
            .unwrap()),
    }
}

pub async fn delete_category(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let category = if let Some(e) = params.get("category_id") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_empty())
            .unwrap());
    };
    let category_id = Ulid::from_string(&category).unwrap();
    match delete(&pool, category_id).await {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("HX-Trigger", "categoryChangeSuccess")
            .body(serve_full("Success delete a category"))
            .unwrap()),
        Err(err) => {
            return Ok(Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(serve_full(err.to_string()))
                .unwrap());
        }
    }
}
