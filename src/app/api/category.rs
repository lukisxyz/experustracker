use std::collections::HashMap;

use http_body_util::BodyExt;
use hyper::{body::Incoming, Request, Response, StatusCode};
use sqlx::PgPool;
use ulid::Ulid;

use crate::{
    app::web::handler::HandlerResult,
    database::model::category::Category,
    utils::{serve_empty, serve_full},
};

use super::get_session_account_id;
static NAME_MISSING: &[u8] = b"missing field: name";
static DESC_MISSING: &[u8] = b"missing field: description";

pub async fn create_category(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    match get_session_account_id(&req, &pool).await {
        Some(_) => {
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

            let new_category =
                Category::new(&name, &description, is_expense, Ulid::from_bytes(book_id));

            match sqlx::query(
                "INSERT INTO categories (id, name, description, is_expense, book_id) VALUES ($1, $2, $3, $4, $5) RETURNING *;",
            )
            .bind(new_category.id.to_bytes())
            .bind(new_category.name)
            .bind(new_category.description)
            .bind(new_category.is_expense)
            .bind(book_id)
            .execute(&pool)
            .await {
                Ok(_) => Ok(Response::builder()
                .status(StatusCode::CREATED)
                .header("HX-Trigger", "createcategorySuccess")
                .body(serve_full("Success create a category"))
                .unwrap()),
                Err(err) => Ok(Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(serve_full(err.to_string()))
                .unwrap()),
            }
        }
        None => Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(serve_empty())
            .unwrap()),
    }
}

pub async fn edit_category(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    match get_session_account_id(&req, &pool).await {
        Some(_) => {
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

            match sqlx::query(
                "UPDATE categories
                SET name = $2, description = $3, updated_at = CURRENT_TIMESTAMP
                WHERE id = $1",
            )
            .bind(Ulid::from_string(&category_id_str).unwrap().to_bytes())
            .bind(name)
            .bind(description)
            .execute(&pool)
            .await
            {
                Ok(_) => Ok(Response::builder()
                    .status(StatusCode::OK)
                    .header("HX-Trigger", "categoryChangeSuccess")
                    .body(serve_full("Success update a category"))
                    .unwrap()),
                Err(err) => Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(serve_full(err.to_string()))
                    .unwrap()),
            }
        }
        None => Ok(Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body(serve_empty())
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
    let category_id = Ulid::from_string(&category).unwrap().to_bytes();
    match sqlx::query(
        "UPDATE categories
        SET deleted_at = CURRENT_TIMESTAMP
        WHERE id = $1",
    )
    .bind(category_id)
    .execute(&pool)
    .await
    {
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
