use super::{
    common::{html_str_handler, HandlerResult},
    templates::{AddNewCategoryTemplate, CategoryListsTemplate, EditCategoryTemplate},
};
use crate::{
    app::api::get_book_default_id,
    database::querier::category::{get_by_book_id, get_by_id},
    utils::serve_empty,
};
use askama::Template;
use hyper::{body::Incoming, header::LOCATION, Request, Response, StatusCode};
use sqlx::PgPool;
use ulid::Ulid;

pub async fn page_category_create(req: Request<Incoming>, _: PgPool, _: Ulid) -> HandlerResult {
    let book_id: Ulid;
    {
        let header = req.headers();
        let id = get_book_default_id(header);
        book_id = id.await.unwrap();
    }
    let template = AddNewCategoryTemplate {
        id: book_id.to_string(),
    };
    let html = template.render().expect("Should render markup");
    html_str_handler(&html).await
}

pub async fn page_categories(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
    let book_id: Ulid;
    {
        let header = req.headers();
        let id = get_book_default_id(header);
        book_id = id.await.unwrap();
    }
    let datas = get_by_book_id(book_id, pool).await;
    let template = CategoryListsTemplate { categories: &datas };
    let html = template.render().expect("Should render markup");
    html_str_handler(&html).await
}

pub async fn page_category_edit(_: Request<Incoming>, pool: PgPool, id: Ulid) -> HandlerResult {
    if let Some(category) = get_by_id(id, pool).await {
        let template = EditCategoryTemplate {
            id: category.id.to_string(),
            name: category.name,
            description: category.description,
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
