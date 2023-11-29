use sqlx::{FromRow, PgPool};
use sqlx_core::error::BoxDynError;
use ulid::Ulid;

use crate::database::model::record::Record;

pub async fn save(pool: &PgPool, r: Record) -> Result<(), BoxDynError> {
    match sqlx::query(
        "INSERT INTO records (id, notes, amount, category_id, book_id) VALUES ($1, $2, $3, $4, $5) RETURNING *;",
    )
    .bind(r.id.to_bytes())
    .bind(r.notes)
    .bind(r.amount)
    .bind(r.category_id.to_bytes())
    .bind(r.book_id.to_bytes())
    .execute(pool)
    .await {
        Ok(_) => Ok(()),
        Err(err) => Err(Box::new(err)),
    }
}

pub async fn edit(
    pool: &PgPool,
    notes: String,
    amount: f32,
    record_id: Ulid,
    category_id: Ulid,
) -> Result<(), BoxDynError> {
    match sqlx::query(
        "UPDATE records
                SET notes = $2, amount = $3, category_id = $4, updated_at = CURRENT_TIMESTAMP
                WHERE id = $1",
    )
    .bind(record_id.to_bytes())
    .bind(notes)
    .bind(amount)
    .bind(category_id.to_bytes())
    .execute(pool)
    .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}

pub async fn delete(pool: &PgPool, record_id: Ulid) -> Result<(), BoxDynError> {
    let mut tx = pool.begin().await.unwrap();
    match sqlx::query(
        "UPDATE records
        SET deleted_at = CURRENT_TIMESTAMP
        WHERE id = $1",
    )
    .bind(record_id.to_bytes())
    .execute(&mut *tx)
    .await
    {
        Ok(_) => {
            tx.commit().await.unwrap();
            Ok(())
        }
        Err(err) => {
            tx.rollback().await.unwrap();
            Err(Box::new(err))
        }
    }
}

pub async fn fetch(book_id: Ulid, cursor: &str, count: i32, pool: PgPool) -> Vec<Record> {
    let start_id = match Ulid::from_string(cursor) {
        Ok(v) => v,
        Err(_) => ulid::Ulid::new(),
    };
    match sqlx::query(
        "
        SELECT records.*, categories.name AS category_name
        FROM records
        JOIN categories ON records.category_id = categories.id
        WHERE records.book_id = $1 AND records.deleted_at IS NULL AND records.id < $2
        ORDER BY records.id DESC
        LIMIT $3;        
    ",
    )
    .bind(book_id.to_bytes())
    .bind(start_id.to_bytes())
    .bind(count)
    .fetch_all(&pool)
    .await
    {
        Ok(v) => {
            let mut datas: Vec<Record> = Vec::new();
            for record in v {
                let b = Record::from_row(&record).unwrap();
                datas.push(b)
            }
            return datas;
        }
        Err(_) => [].to_vec(),
    }
}

pub async fn get_by_id(id: Ulid, pool: PgPool) -> Option<Record> {
    match sqlx::query(
        "SELECT records.*, categories.name AS category_name
        FROM records
        JOIN categories ON records.category_id = categories.id
        WHERE records.id = $1 AND records.deleted_at IS NULL;
        ",
    )
    .bind(id.to_bytes())
    .fetch_one(&pool)
    .await
    {
        Ok(v) => {
            let record = Record::from_row(&v).unwrap();
            Some(record)
        }
        Err(_) => None,
    }
}
