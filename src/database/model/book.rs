use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::prelude::FromRow;
use sqlx::Row;
use ulid::{serde::ulid_as_u128, Ulid};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Book {
    #[serde(with = "ulid_as_u128")]
    pub id: Ulid,
    pub name: String,
    pub description: String,

    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

pub struct BookJson {
    pub id: String,
    pub name: String,
    pub description: String,
}

impl Book {
    pub fn new(name: &str, desc: &str) -> Self {
        let id = ulid::Ulid::new();
        let created_at = chrono::offset::Utc::now();
        Self {
            id,
            name: (&name).to_string(),
            description: (&desc).to_string(),
            created_at,
            updated_at: None,
            deleted_at: None,
        }
    }
    pub fn to_json(&self) -> BookJson {
        let book = self.clone();
        BookJson {
            id: book.id.to_string(),
            name: book.name,
            description: book.description,
        }
    }
}

impl FromRow<'_, PgRow> for Book {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let id: [u8; 16] = row.get("id");
        let name: String = row.get("name");
        let description: String = row.get("description");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: Option<DateTime<Utc>> = match row.try_get("updated_at") {
            Ok(v) => v,
            Err(_) => None,
        };
        let deleted_at: Option<DateTime<Utc>> = match row.try_get("deleted_at") {
            Ok(v) => v,
            Err(_) => None,
        };

        let res: Book = Self {
            id: Ulid::from_bytes(id),
            created_at,
            updated_at,
            deleted_at,
            name,
            description,
        };
        Ok(res)
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct AccountBook {
    #[serde(with = "ulid_as_u128")]
    pub account_id: Ulid,
    #[serde(with = "ulid_as_u128")]
    pub book_id: Ulid,
}
