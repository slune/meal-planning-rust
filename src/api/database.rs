use axum::{
    body::Body,
    extract::Extension,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::SqlitePool;

#[derive(Debug, thiserror::Error)]
enum DatabaseExportError {
    #[error("failed to export database snapshot: {0}")]
    Sqlx(#[from] sqlx::Error),
}

struct DatabaseSnapshot {
    filename: String,
    bytes: Vec<u8>,
}

pub async fn download_database(Extension(pool): Extension<SqlitePool>) -> Response {
    match export_database_snapshot(&pool).await {
        Ok(snapshot) => database_response(snapshot),
        Err(error) => {
            tracing::error!(?error, "Failed to export database");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to export database",
            )
                .into_response()
        }
    }
}

fn database_response(snapshot: DatabaseSnapshot) -> Response {
    let mut response = Response::new(Body::from(snapshot.bytes));
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/vnd.sqlite3"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_str(&format!("attachment; filename=\"{}\"", snapshot.filename))
            .expect("generated filename must be a valid header value"),
    );
    headers.insert(
        header::CACHE_CONTROL,
        header::HeaderValue::from_static("no-store"),
    );
    response
}

async fn export_database_snapshot(
    pool: &SqlitePool,
) -> Result<DatabaseSnapshot, DatabaseExportError> {
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let filename = format!("meal-planning-database-{}.sqlite", timestamp);
    let mut connection = pool.acquire().await?;
    let serialized = (&mut *connection).serialize(None).await?;

    Ok(DatabaseSnapshot {
        filename,
        bytes: serialized.as_ref().to_vec(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn export_database_snapshot_returns_sqlite_file() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("CREATE TABLE meals (id INTEGER PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO meals (name) VALUES ('breakfast')")
            .execute(&pool)
            .await
            .unwrap();

        let snapshot = export_database_snapshot(&pool).await.unwrap();

        assert!(snapshot.filename.starts_with("meal-planning-database-"));
        assert!(snapshot.filename.ends_with(".sqlite"));
        assert!(snapshot.bytes.starts_with(b"SQLite format 3\0"));
    }
}
