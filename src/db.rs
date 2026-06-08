use serde::Deserialize;
use sqlx::{
    AssertSqlSafe, Row, SqlitePool,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::collections::HashMap;
use std::env;
use std::str::FromStr;

#[derive(Deserialize)]
struct RecipePortionSeed {
    porci: String,
}

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data/meal_planning.db".to_string());

    // Create database directory if it doesn't exist
    let db_path = database_url.replace("sqlite://", "");
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            sqlx::Error::Io(std::io::Error::other(format!(
                "Failed to create database directory {}: {}",
                parent.display(),
                e
            )))
        })?;
    }

    // Parse connection options and ensure create_if_missing is set
    let connect_options = SqliteConnectOptions::from_str(&database_url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(connect_options)
        .await?;

    // Run migrations
    run_migrations(&pool).await?;

    Ok(pool)
}

async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    // Ensure migration tracking table exists
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            name TEXT PRIMARY KEY,
            applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(pool)
    .await?;

    let migrations: &[(&str, &str)] = &[
        (
            "001_create_categories",
            include_str!("../migrations/001_create_categories.sql"),
        ),
        (
            "002_create_ingredients",
            include_str!("../migrations/002_create_ingredients.sql"),
        ),
        (
            "003_create_recipes",
            include_str!("../migrations/003_create_recipes.sql"),
        ),
        (
            "004_create_camps",
            include_str!("../migrations/004_create_camps.sql"),
        ),
        (
            "005_create_meal_plans",
            include_str!("../migrations/005_create_meal_plans.sql"),
        ),
        (
            "006_remove_planned_meals_unique",
            include_str!("../migrations/006_remove_planned_meals_unique.sql"),
        ),
        (
            "007_rename_base_servings_to_portions",
            include_str!("../migrations/007_rename_base_servings_to_portions.sql"),
        ),
    ];

    for (name, sql) in migrations {
        let already_applied: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = ?)")
                .bind(name)
                .fetch_one(pool)
                .await?;

        if !already_applied {
            if *name == "007_rename_base_servings_to_portions" {
                rename_base_servings_to_portions(pool).await?;
                continue;
            }

            sqlx::query(*sql).execute(pool).await?;
            sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
                .bind(name)
                .execute(pool)
                .await?;
        }
    }

    normalize_imported_recipe_portions(pool).await?;

    Ok(())
}

async fn rename_base_servings_to_portions(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    const MIGRATION_NAME: &str = "007_rename_base_servings_to_portions";

    let has_portions = table_has_column(pool, "recipes", "portions").await?;
    let has_base_servings = table_has_column(pool, "recipes", "base_servings").await?;

    if has_portions {
        sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
            .bind(MIGRATION_NAME)
            .execute(pool)
            .await?;
        return Ok(());
    }

    if !has_base_servings {
        return Err(sqlx::Error::Protocol(
            "Cannot migrate recipes servings column: neither base_servings nor portions exists"
                .to_string(),
        ));
    }

    sqlx::query(include_str!(
        "../migrations/007_rename_base_servings_to_portions.sql"
    ))
    .execute(pool)
    .await?;

    sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
        .bind(MIGRATION_NAME)
        .execute(pool)
        .await?;

    Ok(())
}

async fn table_has_column(
    pool: &SqlitePool,
    table: &str,
    column: &str,
) -> Result<bool, sqlx::Error> {
    let rows = sqlx::query(AssertSqlSafe(format!("PRAGMA table_info({})", table)))
        .fetch_all(pool)
        .await?;

    for row in rows {
        let name: String = row.try_get("name")?;
        if name == column {
            return Ok(true);
        }
    }

    Ok(false)
}

async fn normalize_imported_recipe_portions(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    const MIGRATION_NAME: &str = "008_normalize_imported_recipe_portions";

    let already_applied: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = ?)")
            .bind(MIGRATION_NAME)
            .fetch_one(pool)
            .await?;

    if already_applied {
        return Ok(());
    }

    let recipes: HashMap<String, RecipePortionSeed> =
        serde_yaml::from_str(include_str!("../source_data/recipes.yaml"))
            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;

    let mut tx = pool.begin().await?;

    for (recipe_name, recipe) in recipes {
        let portions = recipe.porci.parse::<i32>().unwrap_or(1).max(1);

        if portions > 1 {
            sqlx::query(
                "UPDATE recipe_ingredients
                 SET base_quantity = base_quantity * ?
                 WHERE recipe_id IN (
                     SELECT id FROM recipes WHERE name = ? AND portions = 1
                 )",
            )
            .bind(portions as f64)
            .bind(&recipe_name)
            .execute(&mut *tx)
            .await?;
        }

        sqlx::query("UPDATE recipes SET portions = ? WHERE name = ? AND portions = 1")
            .bind(portions)
            .bind(&recipe_name)
            .execute(&mut *tx)
            .await?;
    }

    sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
        .bind(MIGRATION_NAME)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_pool() -> SqlitePool {
        SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn run_migrations_marks_recipe_column_rename_if_already_applied() {
        let pool = setup_pool().await;

        for migration in [
            include_str!("../migrations/001_create_categories.sql"),
            include_str!("../migrations/002_create_ingredients.sql"),
            include_str!("../migrations/003_create_recipes.sql"),
            include_str!("../migrations/004_create_camps.sql"),
            include_str!("../migrations/005_create_meal_plans.sql"),
            include_str!("../migrations/006_remove_planned_meals_unique.sql"),
        ] {
            sqlx::query(migration).execute(&pool).await.unwrap();
        }

        sqlx::query(include_str!(
            "../migrations/007_rename_base_servings_to_portions.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "CREATE TABLE _migrations (
                name TEXT PRIMARY KEY,
                applied_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
        )
        .execute(&pool)
        .await
        .unwrap();

        for name in [
            "001_create_categories",
            "002_create_ingredients",
            "003_create_recipes",
            "004_create_camps",
            "005_create_meal_plans",
            "006_remove_planned_meals_unique",
        ] {
            sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
                .bind(name)
                .execute(&pool)
                .await
                .unwrap();
        }

        run_migrations(&pool).await.unwrap();

        assert!(
            table_has_column(&pool, "recipes", "portions")
                .await
                .unwrap()
        );
        let applied: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = '007_rename_base_servings_to_portions')",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(applied);
    }
}
