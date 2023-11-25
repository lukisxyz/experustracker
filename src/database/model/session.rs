use chrono::serde::ts_milliseconds;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgRow;
use sqlx::prelude::FromRow;
use sqlx::Row;
use ulid::{serde::ulid_as_u128, Ulid};

use crate::utils::generate_random_string;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Session {
    #[serde(with = "ulid_as_u128")]
    pub session_id: Ulid,
    #[serde(with = "ulid_as_u128")]
    pub user_id: Ulid,
    pub token: String,
    #[serde(with = "ts_milliseconds")]
    pub issued_at: DateTime<Utc>,
    pub expire_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: bool,
}

impl Session {
    pub fn new(user_id: Ulid, ip_address: Option<String>, user_agent: Option<String>) -> Self {
        let token = generate_random_string(26);
        let session_id = ulid::Ulid::new();
        let issued_at = chrono::offset::Utc::now();
        Self {
            session_id,
            user_id,
            token,
            issued_at,
            expire_at: issued_at + Duration::days(7),
            ip_address,
            user_agent,
            status: true,
        }
    }
}

impl FromRow<'_, PgRow> for Session {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let id: [u8; 16] = row.get("session_id");
        let user_id: [u8; 16] = row.get("user_id");
        let token: String = row.get("token");
        let status: bool = row.get("status");
        let issued_at: DateTime<Utc> = row.get("issued_at");
        let expire_at: DateTime<Utc> = row.get("expire_at");

        let ip_address: Option<String> = match row.try_get("token") {
            Ok(v) => v,
            Err(_) => None,
        };
        let user_agent: Option<String> = match row.try_get("token") {
            Ok(v) => v,
            Err(_) => None,
        };

        let res: Session = Self {
            session_id: Ulid::from_bytes(id),
            user_id: Ulid::from_bytes(user_id),
            token,
            issued_at,
            expire_at,
            ip_address,
            user_agent,
            status,
        };
        Ok(res)
    }
}
