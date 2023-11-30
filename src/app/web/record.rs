use super::{
    common::{html_str_handler, HandlerResult},
    templates::{AddRecordTemplate, EditRecordTemplate, RecordWithRupiah},
};
use crate::{
    app::api::get_book_default_id,
    database::querier::{
        category::get_by_book_id,
        record::{fetch, get_by_id},
    },
    utils::format_rupiah,
};
use crate::{app::web::templates::RecordListsTemplate, utils::serve_empty};
use askama::Template;
use hyper::{body::Incoming, header::LOCATION, Request, Response, StatusCode};
use sqlx::PgPool;
use ulid::Ulid;

pub async fn page_records(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
    let book_id: Ulid;
    {
        let header = req.headers();
        let id = get_book_default_id(header);
        book_id = id.await.unwrap();
    }
    let datas = fetch(book_id, "", 10, pool).await;
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
    html_str_handler(&html).await
}

pub async fn page_record_create(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
    let book_id: Ulid;
    {
        let header = req.headers();
        let id = get_book_default_id(header);
        book_id = id.await.unwrap();
    }
    let cats = get_by_book_id(book_id, pool).await;
    let template = AddRecordTemplate {
        id: book_id.to_string(),
        categories: &cats,
    };
    let html = template.render().expect("Should render markup");
    html_str_handler(&html).await
}

pub async fn page_record_edit(req: Request<Incoming>, pool: PgPool, id: Ulid) -> HandlerResult {
    let book_id: Ulid;
    {
        let header = req.headers();
        let id = get_book_default_id(header);
        book_id = id.await.unwrap();
    }
    let pool2 = pool.clone();
    if let Some(record) = get_by_id(id, pool2).await {
        let cats = get_by_book_id(book_id, pool).await;
        let template = EditRecordTemplate {
            id: record.id.to_string(),
            notes: record.notes,
            amount: record.amount,
            category_id: record.category_id,
            categories: &cats,
        };
        let html = template.render().expect("Should render markup");
        html_str_handler(&html).await
    } else {
        Ok(Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "/login")
            .body(serve_empty())
            .unwrap())
    }
}
