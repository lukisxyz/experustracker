use askama::Template;
use hyper::{body::Incoming, header::LOCATION, Request, Response, StatusCode};
use sqlx::{FromRow, PgPool};
use ulid::Ulid;

use crate::{
    app::api::get_book_default_id, database::model::category::Category, utils::serve_empty,
};

use super::{
    handler::{html_str_handler, HandlerResult},
    middleware_auth,
    templates::{AddNewCategoryTemplate, CategoryListsTemplate, EditCategoryTemplate},
};

pub async fn page_category_create(req: Request<Incoming>, _: PgPool, _: Ulid) -> HandlerResult {
    let book_id: Ulid;
    {
        let header = req.headers();
        let id = get_book_default_id(&header);
        book_id = id.await.unwrap();
    }
    let template = AddNewCategoryTemplate {
        id: book_id.to_string(),
    };
    let html = template.render().expect("Should render markup");
    return html_str_handler(&html).await;
}

pub async fn get_category_by_book_id(id: Ulid, pool: PgPool) -> Vec<Category> {
    match sqlx::query(
        "SELECT *
            FROM categories
            WHERE book_id = $1 AND deleted_at IS NULL
            ORDER BY id DESC;
        ",
    )
    .bind(id.to_bytes())
    .fetch_all(&pool)
    .await
    {
        Ok(v) => {
            let mut datas: Vec<Category> = Vec::new();
            for category in v {
                let b = Category::from_row(&category).unwrap();
                datas.push(b)
            }
            return datas;
        }
        Err(_) => todo!(),
    }
}

pub async fn page_categories(req: Request<Incoming>, pool: PgPool, _: Ulid) -> HandlerResult {
    let book_id: Ulid;
    {
        let header = req.headers();
        let id = get_book_default_id(&header);
        book_id = id.await.unwrap();
    }
    let datas = get_category_by_book_id(book_id, pool).await;
    let template = CategoryListsTemplate { categories: &datas };
    let html = template.render().expect("Should render markup");
    return html_str_handler(&html).await;
}

pub async fn edit_category_page(
    req: Request<Incoming>,
    pool: PgPool,
    category: Category,
) -> HandlerResult {
    if let Some(_) = middleware_auth(&req, &pool).await {
        let template = EditCategoryTemplate {
            id: category.id.to_string(),
            name: category.name,
            description: category.description,
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

pub async fn get_category_by_id(id: Ulid, pool: PgPool) -> Option<Category> {
    match sqlx::query(
        "SELECT *
        FROM categories
        WHERE id = $1 AND deleted_at IS NULL;
        ",
    )
    .bind(id.to_bytes())
    .fetch_one(&pool)
    .await
    {
        Ok(v) => {
            let category = Category::from_row(&v).unwrap();
            Some(category)
        }
        Err(_) => None,
    }
}
