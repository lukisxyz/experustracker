use crate::{
    app::web::common::HandlerResult,
    database::{
        model::record::Record,
        querier::record::{delete, edit, save},
    },
    utils::{serve_empty, serve_full},
};
use http_body_util::BodyExt;
use hyper::{body::Incoming, Request, Response, StatusCode};
use sqlx::PgPool;
use std::collections::HashMap;
use ulid::Ulid;
static NOTES_MISSING: &[u8] = b"missing field: notes";
static AMOUNT_MISSING: &[u8] = b"missing field: amount";
static CAT_MISSING: &[u8] = b"missing field: record";
static AMOUNT_ZERO: &[u8] = b"amount cannot be zero";

pub async fn create_record(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let notes = if let Some(e) = params.get("notes") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(NOTES_MISSING))
            .unwrap());
    };
    let amount_str = if let Some(e) = params.get("amount") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(AMOUNT_MISSING))
            .unwrap());
    };
    let amount: f32 = amount_str.parse().unwrap();
    if amount == 0.0 {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(AMOUNT_ZERO))
            .unwrap());
    }
    let book = if let Some(e) = params.get("book_id") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_empty())
            .unwrap());
    };
    let book_id = Ulid::from_string(&book).unwrap().to_bytes();

    let category_id_str = if let Some(e) = params.get("category") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(CAT_MISSING))
            .unwrap());
    };
    let category_id = Ulid::from_string(&category_id_str).unwrap().to_bytes();
    let new_record = Record::new(
        &notes,
        amount,
        Ulid::from_bytes(book_id),
        Ulid::from_bytes(category_id),
    );
    match save(&pool, new_record).await {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::CREATED)
            .header("HX-Trigger", "createrecordSuccess")
            .body(serve_full("Success create a record"))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(err.to_string()))
            .unwrap()),
    }
}

pub async fn edit_record(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let notes = if let Some(e) = params.get("notes") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(NOTES_MISSING))
            .unwrap());
    };
    let amount_str = if let Some(e) = params.get("amount") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(AMOUNT_MISSING))
            .unwrap());
    };
    let amount: f32 = amount_str.parse().unwrap();
    if amount == 0.0 {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(AMOUNT_ZERO))
            .unwrap());
    }
    let record = if let Some(e) = params.get("id") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_empty())
            .unwrap());
    };
    let record_id = Ulid::from_string(&record).unwrap().to_bytes();
    let category_id_str = if let Some(e) = params.get("category") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(CAT_MISSING))
            .unwrap());
    };
    let category_id = Ulid::from_string(&category_id_str).unwrap().to_bytes();
    match edit(
        &pool,
        notes.to_string(),
        amount,
        record_id.into(),
        category_id.into(),
    )
    .await
    {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("HX-Trigger", "recordChangeSuccess")
            .body(serve_full("Success change record"))
            .unwrap()),
        Err(err) => Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(err.to_string()))
            .unwrap()),
    }
}

pub async fn delete_record(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let record = if let Some(e) = params.get("record_id") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_empty())
            .unwrap());
    };
    let record_id = Ulid::from_string(&record).unwrap().to_bytes();
    match delete(&pool, record_id.into()).await {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("HX-Trigger", "recordChangeSuccess")
            .body(serve_full("Success delete a record"))
            .unwrap()),
        Err(err) => {
            return Ok(Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(serve_full(err.to_string()))
                .unwrap());
        }
    }
}
