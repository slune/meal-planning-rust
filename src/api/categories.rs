use crate::models::{
    Category, CategoryIngredientUsage, CategoryUsage, CreateCategory, UpdateCategory,
};
use sqlx::SqlitePool;
use std::collections::BTreeMap;

#[derive(sqlx::FromRow)]
struct CategoryUsageRow {
    category_id: i64,
    ingredient_id: i64,
    ingredient_name: String,
}

pub async fn get_categories(pool: &SqlitePool) -> Result<Vec<Category>, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "SELECT id, name, sort_order, created_at, updated_at 
         FROM categories 
         ORDER BY sort_order, name",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_category_usage(pool: &SqlitePool) -> Result<Vec<CategoryUsage>, sqlx::Error> {
    let rows = sqlx::query_as::<_, CategoryUsageRow>(
        "SELECT
            category_id,
            id as ingredient_id,
            name as ingredient_name
         FROM ingredients
         ORDER BY category_id, name",
    )
    .fetch_all(pool)
    .await?;

    let mut usage_by_category = BTreeMap::<i64, Vec<CategoryIngredientUsage>>::new();
    for row in rows {
        usage_by_category
            .entry(row.category_id)
            .or_default()
            .push(CategoryIngredientUsage {
                id: row.ingredient_id,
                name: row.ingredient_name,
            });
    }

    Ok(usage_by_category
        .into_iter()
        .map(|(category_id, ingredients)| CategoryUsage {
            category_id,
            ingredients,
        })
        .collect())
}

pub async fn get_category(pool: &SqlitePool, id: i64) -> Result<Category, sqlx::Error> {
    sqlx::query_as::<_, Category>(
        "SELECT id, name, sort_order, created_at, updated_at 
         FROM categories 
         WHERE id = ?",
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
         VALUES (?, ?)",
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
         WHERE id = ?",
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
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM ingredients WHERE category_id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    if count.0 > 0 {
        return Err(sqlx::Error::Protocol(format!(
            "Cannot delete category: {} ingredient(s) are still using this category. Please reassign or delete those ingredients first.",
            count.0
        )));
    }

    sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        for migration in [
            include_str!("../../migrations/001_create_categories.sql"),
            include_str!("../../migrations/002_create_ingredients.sql"),
        ] {
            sqlx::query(migration).execute(&pool).await.unwrap();
        }

        pool
    }

    #[tokio::test]
    async fn get_category_usage_returns_ingredients_by_category() {
        let pool = setup_pool().await;

        sqlx::query(
            "INSERT INTO ingredients (name, category_id, primary_unit) VALUES (?, 1, 'kg')",
        )
        .bind("Brambory")
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO ingredients (name, category_id, primary_unit) VALUES (?, 1, 'kg')",
        )
        .bind("Cibule")
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO ingredients (name, category_id, primary_unit) VALUES (?, 2, 'ks')",
        )
        .bind("Jablko")
        .execute(&pool)
        .await
        .unwrap();

        let usage = get_category_usage(&pool).await.unwrap();

        assert_eq!(usage.len(), 2);
        assert_eq!(usage[0].category_id, 1);
        assert_eq!(
            usage[0]
                .ingredients
                .iter()
                .map(|ingredient| ingredient.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Brambory", "Cibule"]
        );
        assert_eq!(usage[1].category_id, 2);
        assert_eq!(usage[1].ingredients[0].name, "Jablko");
    }
}
