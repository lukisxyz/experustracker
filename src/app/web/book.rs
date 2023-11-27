use askama::Template;
use hyper::{body::Incoming, header::LOCATION, Request, Response, StatusCode};
use sqlx::{FromRow, PgPool};

use ulid::Ulid;

use crate::{
    app::{
        api::{book::check_book_owned, get_session_account_id},
        web::{
            middleware_auth,
            templates::{
                AddBookOwnerTemplate, AddNewBookTemplate, BookListsBookTemplate, EditBookTemplate,
            },
        },
    },
    database::model::book::Book,
    utils::serve_empty,
};

use super::handler::{html_str_handler, HandlerResult};

pub async fn add_new_book_page(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    if let Some(id) = middleware_auth(&req, &pool).await {
        if check_book_owned(&pool, id, 0).await {
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
    } else {
        Ok(Response::builder()
            .status(StatusCode::TEMPORARY_REDIRECT)
            .header(LOCATION, "/login")
            .body(serve_empty())
            .unwrap())
    }
}

pub async fn add_book_owner_page(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
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

pub async fn book_lists_page(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    if let Some(id) = get_session_account_id(&req, &pool).await {
        let datas = get_books_by_account_id(id, pool).await;
        let template = BookListsBookTemplate { books: &datas };
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

pub async fn edit_book_page(req: Request<Incoming>, pool: PgPool, book: Book) -> HandlerResult {
    if let Some(id) = middleware_auth(&req, &pool).await {
        let mut can_delete = true;

        if check_book_owned(&pool, id, 2).await {
            can_delete = false
        }

        let template = EditBookTemplate {
            id: book.id.to_string(),
            name: book.name,
            description: book.description,
            is_can_delete: can_delete,
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

pub async fn get_books_by_account_id(id: Ulid, pool: PgPool) -> Vec<Book> {
    match sqlx::query(
        "SELECT b.*
        FROM books b
        JOIN account_books ab ON b.id = ab.book_id
        WHERE ab.account_id = $1 AND ab.deleted_at IS NULL AND b.deleted_at IS NULL
        ORDER BY b.id DESC;
        ",
    )
    .bind(id.to_bytes())
    .fetch_all(&pool)
    .await
    {
        Ok(v) => {
            let mut datas: Vec<Book> = Vec::new();
            for book in v {
                let b = Book::from_row(&book).unwrap();
                datas.push(b)
            }
            return datas;
        }
        Err(_) => todo!(),
    }
}

pub async fn get_books_by_id(id: Ulid, pool: PgPool) -> Option<Book> {
    match sqlx::query(
        "SELECT *
        FROM books
        WHERE id = $1 AND deleted_at IS NULL;
        ",
    )
    .bind(id.to_bytes())
    .fetch_one(&pool)
    .await
    {
        Ok(v) => {
            let book = Book::from_row(&v).unwrap();
            Some(book)
        }
        Err(_) => None,
    }
}
