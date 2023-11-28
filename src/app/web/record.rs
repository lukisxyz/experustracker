use sqlx::{FromRow, PgPool};
use ulid::Ulid;

use crate::{app::api::get_book_default_id, database::model::record::Record, utils::format_rupiah};
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
    templates::{AddRecordTemplate, EditRecordTemplate, RecordWithRupiah},
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
        let mut datas_with_rupiah: Vec<RecordWithRupiah> = Vec::new();
        for data in &datas {
            let data_with_rupiah: RecordWithRupiah = RecordWithRupiah {
                record: data.clone(),
                amount_in_rupiah: format_rupiah(data.amount),
                formatted_date: data.created_at.format("%e %b %Y").to_string(),
            };
            datas_with_rupiah.push(data_with_rupiah);
        }
        let template = RecordListsTemplate {
            records: &datas_with_rupiah,
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
        SELECT records.*, categories.name AS category_name
        FROM records
        JOIN categories ON records.category_id = categories.id
        WHERE records.book_id = $1 AND records.deleted_at IS NULL AND records.id < $2
        ORDER BY records.id DESC
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

pub async fn get_record_by_id(id: Ulid, pool: PgPool) -> Option<Record> {
    match sqlx::query(
        "SELECT records.*, categories.name AS category_name
        FROM records
        JOIN categories ON records.category_id = categories.id
        WHERE records.id = $1 AND records.deleted_at IS NULL;
        ",
    )
    .bind(id.to_bytes())
    .fetch_one(&pool)
    .await
    {
        Ok(v) => {
            let record = Record::from_row(&v).unwrap();
            Some(record)
        }
        Err(_) => None,
    }
}

pub async fn edit_record_page(
    req: Request<Incoming>,
    pool: PgPool,
    record: Record,
) -> HandlerResult {
    if let Some(_) = middleware_auth(&req, &pool).await {
        let book_id: Ulid;
        {
            let header = req.headers();
            let id = get_book_default_id(&header);
            book_id = id.await.unwrap();
        }
        let cats = get_category_by_book_id(book_id, pool).await;
        let template = EditRecordTemplate {
            id: record.id.to_string(),
            notes: record.notes,
            amount: record.amount,
            category_id: record.category_id.to_string(),
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
