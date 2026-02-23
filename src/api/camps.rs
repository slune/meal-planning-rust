use crate::models::{Camp, CreateCamp, UpdateCamp};
use sqlx::SqlitePool;

pub async fn get_camps(pool: &SqlitePool) -> Result<Vec<Camp>, sqlx::Error> {
    sqlx::query_as::<_, Camp>(
        "SELECT id, name, start_date, end_date, default_children, default_teens, default_adults, notes, created_at, updated_at 
         FROM camps 
         ORDER BY start_date DESC"
    )
    .fetch_all(pool)
    .await
}

pub async fn get_camp(pool: &SqlitePool, id: i64) -> Result<Camp, sqlx::Error> {
    sqlx::query_as::<_, Camp>(
        "SELECT id, name, start_date, end_date, default_children, default_teens, default_adults, notes, created_at, updated_at 
         FROM camps 
         WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn create_camp(
    pool: &SqlitePool,
    camp: CreateCamp,
) -> Result<Camp, sqlx::Error> {
    // Validate date range
    if camp.start_date >= camp.end_date {
        return Err(sqlx::Error::Decode(
            "End date must be after start date".into()
        ));
    }

    // Validate counts are non-negative
    if camp.default_children < 0 {
        return Err(sqlx::Error::Decode(
            "Number of children cannot be negative".into()
        ));
    }
    if camp.default_teens < 0 {
        return Err(sqlx::Error::Decode(
            "Number of teens cannot be negative".into()
        ));
    }
    if camp.default_adults < 0 {
        return Err(sqlx::Error::Decode(
            "Number of adults cannot be negative".into()
        ));
    }

    let result = sqlx::query(
        "INSERT INTO camps (name, start_date, end_date, default_children, default_teens, default_adults, notes) 
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(&camp.name)
    .bind(camp.start_date)
    .bind(camp.end_date)
    .bind(camp.default_children)
    .bind(camp.default_teens)
    .bind(camp.default_adults)
    .bind(&camp.notes)
    .execute(pool)
    .await?;

    get_camp(pool, result.last_insert_rowid()).await
}

pub async fn update_camp(
    pool: &SqlitePool,
    id: i64,
    camp: UpdateCamp,
) -> Result<Camp, sqlx::Error> {
    let existing = get_camp(pool, id).await?;

    // Determine final values
    let final_start_date = camp.start_date.unwrap_or(existing.start_date);
    let final_end_date = camp.end_date.unwrap_or(existing.end_date);
    let final_children = camp.default_children.unwrap_or(existing.default_children);
    let final_teens = camp.default_teens.unwrap_or(existing.default_teens);
    let final_adults = camp.default_adults.unwrap_or(existing.default_adults);

    // Validate date range
    if final_start_date >= final_end_date {
        return Err(sqlx::Error::Decode(
            "End date must be after start date".into()
        ));
    }

    // Validate counts are non-negative
    if final_children < 0 {
        return Err(sqlx::Error::Decode(
            "Number of children cannot be negative".into()
        ));
    }
    if final_teens < 0 {
        return Err(sqlx::Error::Decode(
            "Number of teens cannot be negative".into()
        ));
    }
    if final_adults < 0 {
        return Err(sqlx::Error::Decode(
            "Number of adults cannot be negative".into()
        ));
    }

    sqlx::query(
        "UPDATE camps
         SET name = ?, start_date = ?, end_date = ?, default_children = ?,
             default_teens = ?, default_adults = ?, notes = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?"
    )
    .bind(camp.name.unwrap_or(existing.name))
    .bind(final_start_date)
    .bind(final_end_date)
    .bind(final_children)
    .bind(final_teens)
    .bind(final_adults)
    .bind(camp.notes.or(existing.notes))
    .bind(id)
    .execute(pool)
    .await?;

    get_camp(pool, id).await
}

pub async fn delete_camp(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM camps WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
