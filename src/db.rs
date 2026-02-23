use sqlx::{sqlite::{SqliteConnectOptions, SqlitePoolOptions}, SqlitePool};
use std::env;
use std::str::FromStr;

pub async fn init_db() -> Result<SqlitePool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite://data/meal_planning.db".to_string());

    // Create database directory if it doesn't exist
    let db_path = database_url.replace("sqlite://", "");
    if let Some(parent) = std::path::Path::new(&db_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            sqlx::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create database directory {}: {}", parent.display(), e)
            ))
        })?;
    }

    // Parse connection options and ensure create_if_missing is set
    let connect_options = SqliteConnectOptions::from_str(&database_url)?
        .create_if_missing(true);

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
        )"
    )
    .execute(pool)
    .await?;

    let migrations: &[(&str, &str)] = &[
        ("001_create_categories",          include_str!("../migrations/001_create_categories.sql")),
        ("002_create_ingredients",         include_str!("../migrations/002_create_ingredients.sql")),
        ("003_create_recipes",             include_str!("../migrations/003_create_recipes.sql")),
        ("004_create_camps",               include_str!("../migrations/004_create_camps.sql")),
        ("005_create_meal_plans",          include_str!("../migrations/005_create_meal_plans.sql")),
        ("006_remove_planned_meals_unique", include_str!("../migrations/006_remove_planned_meals_unique.sql")),
    ];

    for (name, sql) in migrations {
        let already_applied: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM _migrations WHERE name = ?)"
        )
        .bind(name)
        .fetch_one(pool)
        .await?;

        if !already_applied {
            sqlx::query(sql).execute(pool).await?;
            sqlx::query("INSERT INTO _migrations (name) VALUES (?)")
                .bind(name)
                .execute(pool)
                .await?;
        }
    }

    Ok(())
}
