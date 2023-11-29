use super::handler::{html_str_handler, HandlerResult};
use crate::{
    app::web::{
        middleware_auth,
        templates::{
            AddBookOwnerTemplate, AddNewBookTemplate, BookListsBookTemplate, EditBookTemplate,
        },
    },
    database::querier::book::{get_by_account_id, get_by_id, get_count},
    utils::serve_empty,
};
use askama::Template;
use hyper::{body::Incoming, header::LOCATION, Request, Response, StatusCode};
use sqlx::PgPool;
use ulid::Ulid;

pub async fn page_book_create(_: Request<Incoming>, pool: PgPool, id: Ulid) -> HandlerResult {
    if get_count(&pool, id).await == 0 {
        let template = AddNewBookTemplate {
            is_first_time: true,
        };
        let html = template.render().expect("Should render markup");
        return html_str_handler(&html).await;
    } else {
        let template = AddNewBookTemplate {
            is_first_time: false,
        };
        let html = template.render().expect("Should render markup");
        return html_str_handler(&html).await;
    }
}

pub async fn page_book_add_owner(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    if let Some(_) = middleware_auth(&req, &pool).await {
        let template = AddBookOwnerTemplate::default();
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

pub async fn page_books(_: Request<Incoming>, pool: PgPool, id: Ulid) -> HandlerResult {
    let datas = get_by_account_id(id, pool).await;
    let template = BookListsBookTemplate { books: &datas };
    let html = template.render().expect("Should render markup");
    return html_str_handler(&html).await;
}

pub async fn page_book_edit(_: Request<Incoming>, pool: PgPool, id: Ulid) -> HandlerResult {
    let mut can_delete = true;
    if get_count(&pool, id).await > 1 {
        can_delete = false
    }
    match get_by_id(pool, id).await {
        Some(book) => {
            let template = EditBookTemplate {
                id: book.id.to_string(),
                name: book.name,
                description: book.description,
                is_can_delete: can_delete,
            };
            let html = template.render().expect("Should render markup");
            return html_str_handler(&html).await;
        }
        None => Ok(Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(LOCATION, "/book")
            .body(serve_empty())
            .unwrap()),
    }
}
