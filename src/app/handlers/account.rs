use crate::database::model::account::Account;
use crate::utils::serve_full;
use email_address::EmailAddress;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::{Error, Request, Response, StatusCode};
use sqlx::{Error::RowNotFound, PgPool};
use std::collections::HashMap;
use std::convert::Infallible;

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
        has_digit |= c.is_digit(10);
    }

    !has_whitespace && has_upper && has_lower && has_digit && s.len() >= 8
}

pub async fn validate_email(
    req: Request<Incoming>,
    pool: PgPool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
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

    if !EmailAddress::is_valid(&email) {
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

pub async fn validate_password(
    req: Request<Incoming>,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
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

    // TODO: JUST FOR SIMPLE CASE, CHANGE THIS
    if !is_password_valid(&password) {
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

pub async fn create_new_account(
    req: Request<Incoming>,
    pool: PgPool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
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

    if !EmailAddress::is_valid(&email) {
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

    if !is_password_valid(&password) {
        return Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(PASSWORD_WRONG_FORMAT))
            .unwrap());
    }

    let new_account = Account::new(&email, password);

    match sqlx::query(
        "INSERT INTO accounts (id, email, password, code_verification) VALUES ($1, $2, $3, $4)",
    )
    .bind(new_account.id.to_bytes())
    .bind(new_account.email)
    .bind(new_account.password.expose_secret())
    .bind(new_account.code_verification)
    .execute(&pool)
    .await
    {
        Ok(_) => Ok(Response::builder()
            .status(StatusCode::CREATED)
            .header("HX-Trigger", "registerSuccess")
            .body(serve_full(
                "Success create account, please check your email for verification code",
            ))
            .unwrap()),
        Err(err) => {
            return Ok(Response::builder()
                .status(StatusCode::UNPROCESSABLE_ENTITY)
                .body(serve_full(err.to_string()))
                .unwrap())
        }
    }
}
