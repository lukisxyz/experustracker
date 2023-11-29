use crate::app::web::common::HandlerResult;
use crate::database::model::account::Account;
use crate::database::model::book::Book;
use crate::database::model::category::Category;
use crate::utils::serve_full;
use cookie::time::Duration;
use cookie::Cookie;
use email_address::EmailAddress;
use http_body_util::BodyExt;
use hyper::body::Incoming;
use hyper::header::SET_COOKIE;
use hyper::{Request, Response, StatusCode};
use sqlx::{Error::RowNotFound, PgPool};
use std::collections::HashMap;

static EMAIL_MISSING: &[u8] = b"missing field: email";
static EMAIL_WRONG_FORMAT: &[u8] = b"email is not valid";
static PASSWORD_MISSING: &[u8] = b"missing field: password";
static PASSWORD_WRONG_FORMAT: &[u8] = b"password does not meet the criteria. please ensure it has at least 8 characters, including at least 1 uppercase letter, 1 lowercase letter, and 1 digit.";

fn is_password_valid(s: &str) -> bool {
    let mut has_whitespace = false;
    let mut has_upper = false;
    let mut has_lower = false;
    let mut has_digit = false;

    for c in s.chars() {
        has_whitespace |= c.is_whitespace();
        has_lower |= c.is_lowercase();
        has_upper |= c.is_uppercase();
        has_digit |= c.is_ascii_digit();
    }

    !has_whitespace && has_upper && has_lower && has_digit && s.len() >= 8
}

pub async fn validate_email(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let email = if let Some(e) = params.get("email") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(EMAIL_MISSING))
            .unwrap());
    };

    if !EmailAddress::is_valid(email) {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(EMAIL_WRONG_FORMAT))
            .unwrap());
    }

    match sqlx::query("SELECT id FROM accounts WHERE email = $1")
        .bind(email)
        .fetch_one(&pool)
        .await
    {
        Ok(row) => row,
        Err(err) => {
            if matches!(err, RowNotFound) {
                return Ok(Response::builder()
                    .status(StatusCode::OK)
                    .body(serve_full("".as_bytes()))
                    .unwrap());
            }
            return Ok(Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(serve_full(err.to_string()))
                .unwrap());
        }
    };
    return Ok(Response::builder()
        .status(StatusCode::CONFLICT)
        .body(serve_full(
            "Email is already taken. Please enter another email. ".as_bytes(),
        ))
        .unwrap());
}

pub async fn validate_password(req: Request<Incoming>) -> HandlerResult {
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let password = if let Some(e) = params.get("password") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(PASSWORD_MISSING))
            .unwrap());
    };

    if !is_password_valid(password) {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(PASSWORD_WRONG_FORMAT))
            .unwrap());
    }

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(serve_full("".as_bytes()))
        .unwrap())
}

pub async fn create_account(req: Request<Incoming>, pool: PgPool) -> HandlerResult {
    let body = req.collect().await?.to_bytes();
    let params = form_urlencoded::parse(body.as_ref())
        .into_owned()
        .collect::<HashMap<String, String>>();
    let email = if let Some(e) = params.get("email") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(EMAIL_MISSING))
            .unwrap());
    };

    if !EmailAddress::is_valid(email) {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(EMAIL_WRONG_FORMAT))
            .unwrap());
    }

    let password = if let Some(e) = params.get("password") {
        e
    } else {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(PASSWORD_MISSING))
            .unwrap());
    };

    if !is_password_valid(password) {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(PASSWORD_WRONG_FORMAT))
            .unwrap());
    }

    let mut tx = pool.begin().await.unwrap();
    let new_account = Account::new(email, password);
    match sqlx::query(
        "INSERT INTO accounts (id, email, password, code_verification) VALUES ($1, $2, $3, $4)",
    )
    .bind(new_account.id.to_bytes())
    .bind(new_account.email)
    .bind(new_account.password.expose_secret())
    .bind(new_account.code_verification)
    .execute(&mut *tx)
    .await
    {
        Ok(_) => {
            let new_book = Book::new("Main", "Main book");
            let id = new_book.id;
            match sqlx::query(
                "INSERT INTO books (id, name, description) VALUES ($1, $2, $3) RETURNING *;",
            )
            .bind(new_book.id.to_bytes())
            .bind(new_book.name)
            .bind(new_book.description)
            .execute(&mut *tx)
            .await
            {
                Ok(_) => {
                    match sqlx::query(
                        "INSERT INTO account_books (account_id, book_id) VALUES ($1, $2) ON CONFLICT (account_id, book_id) DO NOTHING"
                    ).bind(new_account.id.to_bytes())
                    .bind(new_book.id.to_bytes())
                    .execute(&mut *tx)
                    .await {
                        Ok(_) => {
                            let categories = Category::batch(id);
                            for c in categories {
                                match sqlx::query(
                                    "INSERT INTO categories (id, name, description, is_expense, book_id) VALUES ($1, $2, $3, $4, $5)",
                                )
                                .bind(c.id.to_bytes())
                                .bind(c.name)
                                .bind(c.description)
                                .bind(c.is_expense)
                                .bind(c.book_id.to_bytes())
                                .execute(&mut *tx)
                                .await {
                                    Ok(_) => {}
                                    Err(err) => {
                                        let _ = tx.rollback().await;
                                        return Ok(Response::builder()
                                            .status(StatusCode::UNPROCESSABLE_ENTITY)
                                            .body(serve_full(err.to_string()))
                                            .unwrap());
                                    }
                                }
                            }
                            let _ = tx.commit().await;
                            let mut c = Cookie::new("book", new_book.id.to_string());
                            c.set_max_age(Duration::days(30 * 12));
                            c.set_path("/");
                            Ok(Response::builder()
                                .status(StatusCode::CREATED)
                                .header("HX-Trigger", "registerSuccess")
                                .header(SET_COOKIE, c.to_string())
                                .body(serve_full(
                                    "Success create account, please check your email for verification code",
                                ))
                                .unwrap())
                            },
                        Err(err) => {
                            let _ = tx.rollback().await;
                            Ok(Response::builder()
                                .status(StatusCode::UNPROCESSABLE_ENTITY)
                                .body(serve_full(err.to_string()))
                                .unwrap())
                        },
                    }
                }
                Err(err) => {
                    let _ = tx.rollback().await;
                    Ok(Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(serve_full(err.to_string()))
                        .unwrap())
                }
            }
        }
        Err(err) => {
            let _ = tx.rollback().await;
            Ok(Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(serve_full(err.to_string()))
                .unwrap())
        }
    }
}
