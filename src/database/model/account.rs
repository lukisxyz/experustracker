use argon2::{Config, Error};
use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use rand::Rng;
use redact::{expose_secret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::prelude::FromRow;
use sqlx::Row;
use ulid::{serde::ulid_as_u128, Ulid};

use crate::utils;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Account {
    #[serde(with = "ulid_as_u128")]
    pub id: Ulid,
    pub email: String,

    #[serde(serialize_with = "expose_secret")]
    pub password: Secret<String>,
    pub code_verification: String,

    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub email_verified_at: Option<DateTime<Utc>>,
}

pub struct AccountJson {
    pub id: String,
    pub email: String,
    pub is_verified: bool,
}

impl Account {
    pub fn new(email: &str, password: &str) -> Self {
        let id = ulid::Ulid::new();
        let created_at = chrono::offset::Utc::now();
        let code_verification = utils::generate_random_string(21);
        let hashed_password = Self::hash_password(password.as_bytes()).unwrap();
        let secret_hashed_password = Secret::from(hashed_password);
        Self {
            id,
            email: email.to_string(),
            password: secret_hashed_password,
            code_verification,
            created_at,
            updated_at: None,
            deleted_at: None,
            email_verified_at: None,
        }
    }

    pub fn to_json(&self) -> AccountJson {
        let acc = self.clone();
        let is_verified = acc.email_verified_at.is_some();

        AccountJson {
            id: acc.id.to_string(),
            email: acc.email,
            is_verified,
        }
    }

    fn hash_password(password: &[u8]) -> Result<String, Error> {
        let salt: [u8; 32] = rand::thread_rng().gen();
        let config = Config::default();

        match argon2::hash_encoded(password, &salt, &config) {
            Ok(v) => Ok(v),
            Err(err) => Err(err),
        }
    }

    pub fn compare_password(&self, password: &[u8]) -> Result<bool, Error> {
        argon2::verify_encoded(self.password.expose_secret(), password)
    }
}

impl FromRow<'_, PgRow> for Account {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let id: [u8; 16] = row.get("id");
        let password: String = row.get("password");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: Option<DateTime<Utc>> = match row.try_get("updated_at") {
            Ok(v) => v,
            Err(_) => None,
        };
        let deleted_at: Option<DateTime<Utc>> = match row.try_get("deleted_at") {
            Ok(v) => v,
            Err(_) => None,
        };
        let email_verified_at: Option<DateTime<Utc>> = match row.try_get("email_verified_at") {
            Ok(v) => v,
            Err(_) => None,
        };

        let res: Account = Self {
            id: Ulid::from_bytes(id),
            email: row.get("email"),
            password: Secret::from(password),
            code_verification: row.get("code_verification"),
            created_at,
            updated_at,
            deleted_at,
            email_verified_at,
        };
        Ok(res)
    }
}
