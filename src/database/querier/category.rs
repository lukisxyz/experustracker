use crate::database::model::category::Category;
use sqlx::{FromRow, PgPool};
use sqlx_core::error::BoxDynError;
use ulid::Ulid;

pub async fn save(pool: &PgPool, c: Category) -> Result<(), BoxDynError> {
    let mut tx = pool.begin().await.unwrap();
    match sqlx::query(
        "INSERT INTO categories (id, name, description, is_expense, book_id) VALUES ($1, $2, $3, $4, $5) RETURNING *;",
    )
    .bind(c.id.to_bytes())
    .bind(c.name)
    .bind(c.description)
    .bind(c.is_expense)
    .bind(c.book_id.to_bytes())
    .execute(&mut *tx)
    .await{
        Ok(_) => {
            tx.commit().await.unwrap();
            Ok(())
        }
        Err(e) => {
            tx.rollback().await.unwrap();
            Err(Box::new(e))
        }
    }
}

pub async fn edit(
    pool: &PgPool,
    name: String,
    desc: String,
    category_id: Ulid,
) -> Result<(), BoxDynError> {
    let mut tx = pool.begin().await.unwrap();
    match sqlx::query(
        "UPDATE categories
        SET name = $2, description = $3, updated_at = CURRENT_TIMESTAMP
        WHERE id = $1",
    )
    .bind(category_id.to_bytes())
    .bind(name)
    .bind(desc)
    .execute(&mut *tx)
    .await
    {
        Ok(_) => {
            tx.commit().await.unwrap();
            Ok(())
        }
        Err(e) => {
            tx.rollback().await.unwrap();
            Err(Box::new(e))
        }
    }
}

pub async fn delete(pool: &PgPool, category_id: Ulid) -> Result<(), BoxDynError> {
    let mut tx = pool.begin().await.unwrap();
    match sqlx::query(
        "UPDATE categories
        SET deleted_at = CURRENT_TIMESTAMP
        WHERE id = $1",
    )
    .bind(category_id.to_bytes())
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

pub async fn get_by_book_id(id: Ulid, pool: PgPool) -> Vec<Category> {
    match sqlx::query(
        "SELECT *
            FROM categories
            WHERE book_id = $1 AND deleted_at IS NULL
            ORDER BY id DESC;
        ",
    )
    .bind(id.to_bytes())
    .fetch_all(&pool)
    .await
    {
        Ok(v) => {
            let mut datas: Vec<Category> = Vec::new();
            for category in v {
                let b = Category::from_row(&category).unwrap();
                datas.push(b)
            }
            datas
        }
        Err(_) => todo!(),
    }
}

pub async fn get_by_id(id: Ulid, pool: PgPool) -> Option<Category> {
    match sqlx::query(
        "SELECT *
            FROM categories
            WHERE id = $1 AND deleted_at IS NULL;
        ",
    )
    .bind(id.to_bytes())
    .fetch_one(&pool)
    .await
    {
        Ok(v) => {
            let cat = Category::from_row(&v).unwrap();
            Some(cat)
        }
        Err(_) => None,
    }
}
