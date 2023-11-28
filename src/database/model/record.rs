use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::prelude::FromRow;
use sqlx::Row;
use ulid::{serde::ulid_as_u128, Ulid};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Record {
    #[serde(with = "ulid_as_u128")]
    pub id: Ulid,
    #[serde(with = "ulid_as_u128")]
    pub book_id: Ulid,
    #[serde(with = "ulid_as_u128")]
    pub category_id: Ulid,
    pub category_name: String,
    pub notes: String,
    pub amount: f32,

    #[serde(with = "ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Record {
    pub fn new(notes: &str, amount: f32, book_id: Ulid, category_id: Ulid) -> Self {
        let id = ulid::Ulid::new();
        let created_at = chrono::offset::Utc::now();
        Self {
            id,
            created_at,
            category_name: "".to_string(),
            updated_at: None,
            deleted_at: None,
            book_id,
            category_id,
            notes: notes.to_string(),
            amount,
        }
    }
}

impl FromRow<'_, PgRow> for Record {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let id: [u8; 16] = row.get("id");
        let category_id: [u8; 16] = row.get("category_id");
        let category_name: String = row.get("category_name");
        let book_id: [u8; 16] = row.get("book_id");
        let notes: String = row.get("notes");
        let amount: f32 = row.get("amount");
        let created_at: DateTime<Utc> = row.get("created_at");
        let updated_at: Option<DateTime<Utc>> = match row.try_get("updated_at") {
            Ok(v) => v,
            Err(_) => None,
        };
        let deleted_at: Option<DateTime<Utc>> = match row.try_get("deleted_at") {
            Ok(v) => v,
            Err(_) => None,
        };

        let res: Record = Self {
            id: Ulid::from_bytes(id),
            created_at,
            updated_at,
            deleted_at,
            book_id: Ulid::from_bytes(book_id),
            category_id: Ulid::from_bytes(category_id),
            notes,
            amount,
            category_name,
        };
        Ok(res)
    }
}
