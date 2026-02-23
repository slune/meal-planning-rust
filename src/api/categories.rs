use crate::models::{Category, CreateCategory, UpdateCategory};
use sqlx::SqlitePool;

pub async fn get_categories(pool: &SqlitePool) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "SELECT id, name, sort_order, created_at, updated_at 
         FROM categories 
         ORDER BY sort_order, name"
    )
    .fetch_all(pool)
    .await
}

pub async fn get_category(pool: &SqlitePool, id: i64) -> Result<Category, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "SELECT id, name, sort_order, created_at, updated_at 
         FROM categories 
         WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn create_category(
    pool: &SqlitePool,
    category: CreateCategory,
) -> Result<Category, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO categories (name, sort_order) 
         VALUES (?, ?)"
    )
    .bind(&category.name)
    .bind(category.sort_order)
    .execute(pool)
    .await?;

    get_category(pool, result.last_insert_rowid()).await
}

pub async fn update_category(
    pool: &SqlitePool,
    id: i64,
    category: UpdateCategory,
) -> Result<Category, sqlx::Error> {
    let existing = get_category(pool, id).await?;

    sqlx::query(
        "UPDATE categories 
         SET name = ?, sort_order = ?, updated_at = CURRENT_TIMESTAMP 
         WHERE id = ?"
    )
    .bind(category.name.unwrap_or(existing.name))
    .bind(category.sort_order.unwrap_or(existing.sort_order))
    .bind(id)
    .execute(pool)
    .await?;

    get_category(pool, id).await
}

pub async fn delete_category(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    // Check if any ingredients are using this category
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM ingredients WHERE category_id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    if count.0 > 0 {
        return Err(sqlx::Error::Protocol(
            format!("Cannot delete category: {} ingredient(s) are still using this category. Please reassign or delete those ingredients first.", count.0)
        ));
    }

    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
