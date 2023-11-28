use sqlx::{FromRow, PgPool};
use ulid::Ulid;

use crate::{app::api::get_book_default_id, database::model::record::Record};
use askama::Template;
use hyper::{body::Incoming, header::LOCATION, Request, Response, StatusCode};

use crate::{
    app::{api::get_session_account_id, web::templates::RecordListsTemplate},
    utils::serve_empty,
};

use super::{
    category::get_category_by_book_id,
    handler::{html_str_handler, HandlerResult},
    middleware_auth,
    templates::AddRecordTemplate,
};

pub async fn record_lists_page(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    if let Some(_) = get_session_account_id(&req, &pool).await {
        let book_id: Ulid;
        {
            let header = req.headers();
            let id = get_book_default_id(&header);
            book_id = id.await.unwrap();
        }
        let datas = get_latest_record(book_id, "", 10, pool).await;
        let template = RecordListsTemplate { records: &datas };
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

pub async fn get_latest_record(
    book_id: Ulid,
    cursor: &str,
    count: i32,
    pool: PgPool,
) -> Vec<Record> {
    let start_id = match Ulid::from_string(cursor) {
        Ok(v) => v,
        Err(_) => ulid::Ulid::new(),
    };
    match sqlx::query(
        "
        SELECT *
        FROM records
        WHERE book_id = $1 AND deleted_at IS NULL AND id < $2
        ORDER BY id DESC
        LIMIT $3;
    ",
    )
    .bind(book_id.to_bytes())
    .bind(start_id.to_bytes())
    .bind(count)
    .fetch_all(&pool)
    .await
    {
        Ok(v) => {
            let mut datas: Vec<Record> = Vec::new();
            for record in v {
                let b = Record::from_row(&record).unwrap();
                datas.push(b)
            }
            return datas;
        }
        Err(_) => [].to_vec(),
    }
}

pub async fn add_new_record_page(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    if let Some(_) = middleware_auth(&req, &pool).await {
        let book_id: Ulid;
        {
            let header = req.headers();
            let id = get_book_default_id(&header);
            book_id = id.await.unwrap();
        }
        let cats = get_category_by_book_id(book_id, pool).await;
        let template = AddRecordTemplate {
            id: book_id.to_string(),
            categories: &cats,
        };
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
