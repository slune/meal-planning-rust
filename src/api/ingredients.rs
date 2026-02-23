use crate::models::{Ingredient, CreateIngredient, UpdateIngredient};
use sqlx::SqlitePool;

pub async fn get_ingredients(pool: &SqlitePool) -> Result<Vec<Ingredient>, sqlx::Error> {
    sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category_id, primary_unit, secondary_unit, created_at, updated_at 
         FROM ingredients 
         ORDER BY name"
    )
    .fetch_all(pool)
    .await
}

pub async fn get_ingredients_by_category(
    pool: &SqlitePool,
    category_id: i64,
) -> Result<Vec<Ingredient>, sqlx::Error> {
    sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category_id, primary_unit, secondary_unit, created_at, updated_at 
         FROM ingredients 
         WHERE category_id = ?
         ORDER BY name"
    )
    .bind(category_id)
    .fetch_all(pool)
    .await
}

pub async fn get_ingredient(pool: &SqlitePool, id: i64) -> Result<Ingredient, sqlx::Error> {
    sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category_id, primary_unit, secondary_unit, created_at, updated_at 
         FROM ingredients 
         WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn create_ingredient(
    pool: &SqlitePool,
    ingredient: CreateIngredient,
) -> Result<Ingredient, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO ingredients (name, category_id, primary_unit, secondary_unit) 
         VALUES (?, ?, ?, ?)"
    )
    .bind(&ingredient.name)
    .bind(ingredient.category_id)
    .bind(&ingredient.primary_unit)
    .bind(&ingredient.secondary_unit)
    .execute(pool)
    .await?;

    get_ingredient(pool, result.last_insert_rowid()).await
}

pub async fn update_ingredient(
    pool: &SqlitePool,
    id: i64,
    ingredient: UpdateIngredient,
) -> Result<Ingredient, sqlx::Error> {
    let existing = get_ingredient(pool, id).await?;

    sqlx::query(
        "UPDATE ingredients 
         SET name = ?, category_id = ?, primary_unit = ?, 
             secondary_unit = ?, updated_at = CURRENT_TIMESTAMP 
         WHERE id = ?"
    )
    .bind(ingredient.name.unwrap_or(existing.name))
    .bind(ingredient.category_id.unwrap_or(existing.category_id))
    .bind(ingredient.primary_unit.unwrap_or(existing.primary_unit))
    .bind(ingredient.secondary_unit.or(existing.secondary_unit))
    .bind(id)
    .execute(pool)
    .await?;

    get_ingredient(pool, id).await
}

pub async fn delete_ingredient(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM ingredients WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
