use crate::database::model::account::Account;
use crate::database::model::session::Session;
use crate::utils::serve_full;
use cookie::time::Duration;
use cookie::Cookie;
use email_address::EmailAddress;
use http_body_util::combinators::BoxBody;
use http_body_util::BodyExt;
use hyper::body::{Bytes, Incoming};
use hyper::header::{COOKIE, SET_COOKIE};
use hyper::{Error, Request, Response, StatusCode};
use sqlx::postgres::PgRow;
use sqlx::FromRow;
use sqlx::{Error::RowNotFound, PgPool};
use std::collections::HashMap;
use std::convert::Infallible;

static EMAIL_MISSING: &[u8] = b"Missing field: Email";
static EMAIL_WRONG_FORMAT: &[u8] = b"Email is not valid";
static PASSWORD_MISSING: &[u8] = b"Missing field: Password";

pub async fn login_account(
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

    let row: PgRow =
        match sqlx::query("SELECT id, email, password, code_verification, created_at, updated_at, deleted_at, email_verified_at FROM accounts WHERE email = $1 AND deleted_at IS NULL;")
            .bind(email)
            .fetch_one(&pool)
            .await
        {
            Ok(v) => v,
            Err(err) => {
                if matches!(err, RowNotFound) {
                    return Ok(Response::builder()
                        .status(StatusCode::NOT_FOUND)
                        .body(serve_full("Mismatch email and password".as_bytes()))
                        .unwrap());
                }
                return Ok(Response::builder()
                    .status(StatusCode::UNPROCESSABLE_ENTITY)
                    .body(serve_full(err.to_string()))
                    .unwrap());
            }
        };

    match Account::from_row(&row) {
        Ok(acc) => {
            match sqlx::query(
                "
                SELECT *
                FROM sessions
                WHERE user_id = $1
                AND status = TRUE 
                AND (expire_at > CURRENT_TIMESTAMP OR expire_at IS NULL);
            ",
            )
            .bind(acc.id.to_bytes())
            .fetch_one(&pool)
            .await
            {
                Ok(v) => {
                    let session_token = Session::from_row(&v).unwrap().token;
                    return Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("HX-Trigger", "loginSuccess")
                        .header(SET_COOKIE, session_token.to_string())
                        .body(serve_full("Success login"))
                        .unwrap());
                }
                Err(err) => {
                    if !matches!(err, RowNotFound) {
                        return Ok(Response::builder()
                            .status(StatusCode::UNPROCESSABLE_ENTITY)
                            .body(serve_full(err.to_string()))
                            .unwrap());
                    }
                }
            };
            let is_password_match = acc.compare_password(password.as_bytes()).unwrap();

            if !is_password_match {
                return Ok(Response::builder()
                    .status(StatusCode::UNAUTHORIZED)
                    .body(serve_full("Wrong password"))
                    .unwrap());
            }

            let session = Session::new(acc.id, Some("".to_string()), Some("".to_string()));

            match sqlx::query(
                "INSERT INTO sessions 
                    (session_id, user_id, token, issued_at, expire_at, ip_address, user_agent, status)
                VALUES
                    ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *;",
            )
            .bind(&session.session_id.to_bytes())
            .bind(&session.user_id.to_bytes())
            .bind(&session.token)
            .bind(&session.issued_at)
            .bind(&session.expire_at)
            .bind(&session.ip_address)
            .bind(&session.user_agent)
            .bind(&session.status)
            .fetch_one(&pool)
            .await
            {
                Ok(_) => {
                    let mut c = Cookie::new("session", session.token);
                    c.set_http_only(true);
                    c.set_max_age(Duration::days(1));
                    c.set_path("/");
                    c.set_secure(true);

                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("HX-Trigger", "loginSuccess")
                        .header(SET_COOKIE, c.to_string())
                        .body(serve_full("Success login"))
                        .unwrap())
                }
                Err(err) => {
                    return Ok(Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(serve_full(err.to_string()))
                        .unwrap());
                }
            }
        }
        Err(err) => Ok(Response::builder()
            .status(StatusCode::UNPROCESSABLE_ENTITY)
            .body(serve_full(err.to_string()))
            .unwrap()),
    }
}

pub async fn logout_account(
    req: Request<Incoming>,
    pool: PgPool,
) -> Result<Response<BoxBody<Bytes, Infallible>>, Error> {
    let headers = req.headers();
    match headers.get(COOKIE) {
        Some(v) => {
            let c = Cookie::parse(v.to_str().unwrap()).unwrap();
            match sqlx::query(
                "
                        UPDATE
                            sessions
                        SET
                            status = FALSE
                        WHERE
                            token = $1
                        RETURNING *;
                    ",
            )
            .bind(c.value())
            .fetch_one(&pool)
            .await
            {
                Ok(_) => {
                    let mut c = Cookie::new("session", "".to_string());
                    c.make_removal();

                    Ok(Response::builder()
                        .status(StatusCode::OK)
                        .header("HX-Trigger", "logoutSuccess")
                        .header(SET_COOKIE, c.to_string())
                        .body(serve_full("Success logout"))
                        .unwrap())
                }
                Err(err) => {
                    if matches!(err, RowNotFound) {
                        return Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(serve_full(err.to_string()))
                            .unwrap());
                    }
                    return Ok(Response::builder()
                        .status(StatusCode::UNPROCESSABLE_ENTITY)
                        .body(serve_full(err.to_string()))
                        .unwrap());
                }
            }
        }
        None => todo!(),
    }
}
