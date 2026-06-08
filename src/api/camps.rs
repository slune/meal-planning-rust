use crate::models::{Camp, CreateCamp, UpdateCamp};
use chrono::Duration;
use sqlx::SqlitePool;
use std::collections::HashMap;

type SourceMealCopyRow = (i64, i64, String, Option<i32>, Option<i32>, Option<i32>);

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

pub async fn create_camp(pool: &SqlitePool, camp: CreateCamp) -> Result<Camp, sqlx::Error> {
    validate_camp_values(&camp)?;

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

pub async fn duplicate_camp(
    pool: &SqlitePool,
    source_id: i64,
    camp: CreateCamp,
) -> Result<Camp, sqlx::Error> {
    let source = get_camp(pool, source_id).await?;
    validate_camp_values(&camp)?;

    if camp.name.trim() == source.name.trim() {
        return Err(sqlx::Error::Decode(
            "Copied camp must use a different name".into(),
        ));
    }

    let source_days = (source.end_date - source.start_date).num_days() + 1;
    let target_days = (camp.end_date - camp.start_date).num_days() + 1;
    if source_days != target_days {
        return Err(sqlx::Error::Decode(
            "Copied camp must have the same number of days as the source camp".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    let result = sqlx::query(
        "INSERT INTO camps (name, start_date, end_date, default_children, default_teens, default_adults, notes)
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&camp.name)
    .bind(camp.start_date)
    .bind(camp.end_date)
    .bind(camp.default_children)
    .bind(camp.default_teens)
    .bind(camp.default_adults)
    .bind(&camp.notes)
    .execute(&mut *tx)
    .await?;
    let new_camp_id = result.last_insert_rowid();

    let source_plans: Vec<(i64, chrono::NaiveDate)> =
        sqlx::query_as("SELECT id, date FROM meal_plans WHERE camp_id = ? ORDER BY date")
            .bind(source_id)
            .fetch_all(&mut *tx)
            .await?;

    let mut plan_id_map = HashMap::new();
    for (source_plan_id, source_date) in source_plans {
        let day_offset = (source_date - source.start_date).num_days();
        let target_date = camp.start_date + Duration::days(day_offset);

        let result = sqlx::query("INSERT INTO meal_plans (camp_id, date) VALUES (?, ?)")
            .bind(new_camp_id)
            .bind(target_date)
            .execute(&mut *tx)
            .await?;

        plan_id_map.insert(source_plan_id, result.last_insert_rowid());
    }

    let source_meals: Vec<SourceMealCopyRow> = sqlx::query_as(
        "SELECT
            pm.meal_plan_id,
            pm.recipe_id,
            pm.meal_type,
            ma.children,
            ma.teens,
            ma.adults
         FROM meal_plans mp
         JOIN planned_meals pm ON pm.meal_plan_id = mp.id
         LEFT JOIN meal_attendance ma ON ma.planned_meal_id = pm.id
         WHERE mp.camp_id = ?
         ORDER BY mp.date, pm.id",
    )
    .bind(source_id)
    .fetch_all(&mut *tx)
    .await?;

    for (source_plan_id, recipe_id, meal_type, children, teens, adults) in source_meals {
        let Some(new_plan_id) = plan_id_map.get(&source_plan_id) else {
            continue;
        };

        let result = sqlx::query(
            "INSERT INTO planned_meals (meal_plan_id, recipe_id, meal_type)
             VALUES (?, ?, ?)",
        )
        .bind(new_plan_id)
        .bind(recipe_id)
        .bind(&meal_type)
        .execute(&mut *tx)
        .await?;

        if let (Some(children), Some(teens), Some(adults)) = (children, teens, adults) {
            sqlx::query(
                "INSERT INTO meal_attendance (planned_meal_id, children, teens, adults)
                 VALUES (?, ?, ?, ?)",
            )
            .bind(result.last_insert_rowid())
            .bind(children)
            .bind(teens)
            .bind(adults)
            .execute(&mut *tx)
            .await?;
        }
    }

    tx.commit().await?;

    get_camp(pool, new_camp_id).await
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

    validate_camp_range_and_counts(
        final_start_date,
        final_end_date,
        final_children,
        final_teens,
        final_adults,
    )?;

    sqlx::query(
        "UPDATE camps
         SET name = ?, start_date = ?, end_date = ?, default_children = ?,
             default_teens = ?, default_adults = ?, notes = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(camp.name.unwrap_or(existing.name))
    .bind(final_start_date)
    .bind(final_end_date)
    .bind(final_children)
    .bind(final_teens)
    .bind(final_adults)
    .bind(camp.notes.unwrap_or(existing.notes))
    .bind(id)
    .execute(pool)
    .await?;

    get_camp(pool, id).await
}

pub async fn delete_camp(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "DELETE FROM meal_attendance
         WHERE planned_meal_id IN (
            SELECT pm.id
            FROM planned_meals pm
            JOIN meal_plans mp ON mp.id = pm.meal_plan_id
            WHERE mp.camp_id = ?
         )",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "DELETE FROM planned_meals
         WHERE meal_plan_id IN (
            SELECT id
            FROM meal_plans
            WHERE camp_id = ?
         )",
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM meal_plans WHERE camp_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM camps WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}

fn validate_camp_values(camp: &CreateCamp) -> Result<(), sqlx::Error> {
    validate_camp_range_and_counts(
        camp.start_date,
        camp.end_date,
        camp.default_children,
        camp.default_teens,
        camp.default_adults,
    )
}

fn validate_camp_range_and_counts(
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
    children: i32,
    teens: i32,
    adults: i32,
) -> Result<(), sqlx::Error> {
    if start_date > end_date {
        return Err(sqlx::Error::Decode(
            "End date must be after start date".into(),
        ));
    }

    if children < 0 {
        return Err(sqlx::Error::Decode(
            "Number of children cannot be negative".into(),
        ));
    }
    if teens < 0 {
        return Err(sqlx::Error::Decode(
            "Number of teens cannot be negative".into(),
        ));
    }
    if adults < 0 {
        return Err(sqlx::Error::Decode(
            "Number of adults cannot be negative".into(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
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
            include_str!("../../migrations/003_create_recipes.sql"),
            include_str!("../../migrations/004_create_camps.sql"),
            include_str!("../../migrations/005_create_meal_plans.sql"),
            include_str!("../../migrations/006_remove_planned_meals_unique.sql"),
            include_str!("../../migrations/007_rename_base_servings_to_portions.sql"),
        ] {
            sqlx::query(migration).execute(&pool).await.unwrap();
        }

        pool
    }

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    fn camp(name: &str, start: &str, end: &str) -> CreateCamp {
        CreateCamp {
            name: name.to_string(),
            start_date: date(start),
            end_date: date(end),
            default_children: 10,
            default_teens: 5,
            default_adults: 3,
            notes: Some("notes".to_string()),
        }
    }

    async fn insert_recipe(pool: &SqlitePool, name: &str) -> i64 {
        sqlx::query("INSERT INTO recipes (name, portions) VALUES (?, 4)")
            .bind(name)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    async fn insert_meal(
        pool: &SqlitePool,
        camp_id: i64,
        date: NaiveDate,
        recipe_id: i64,
        meal_type: &str,
        attendance: (i32, i32, i32),
    ) {
        let meal_plan_id = sqlx::query("INSERT INTO meal_plans (camp_id, date) VALUES (?, ?)")
            .bind(camp_id)
            .bind(date)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid();

        let planned_meal_id = sqlx::query(
            "INSERT INTO planned_meals (meal_plan_id, recipe_id, meal_type) VALUES (?, ?, ?)",
        )
        .bind(meal_plan_id)
        .bind(recipe_id)
        .bind(meal_type)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_rowid();

        sqlx::query(
            "INSERT INTO meal_attendance (planned_meal_id, children, teens, adults) VALUES (?, ?, ?, ?)",
        )
        .bind(planned_meal_id)
        .bind(attendance.0)
        .bind(attendance.1)
        .bind(attendance.2)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn duplicate_camp_copies_meals_by_day_offset_and_attendance() {
        let pool = setup_pool().await;
        let recipe_id = insert_recipe(&pool, "Pasta").await;
        let source = create_camp(&pool, camp("Summer camp", "2026-07-01", "2026-07-03"))
            .await
            .unwrap();

        insert_meal(
            &pool,
            source.id,
            date("2026-07-02"),
            recipe_id,
            "lunch",
            (11, 6, 4),
        )
        .await;

        let copied = duplicate_camp(
            &pool,
            source.id,
            camp("Autumn camp", "2026-09-10", "2026-09-12"),
        )
        .await
        .unwrap();

        let copied_meal: (NaiveDate, i64, String, i32, i32, i32) = sqlx::query_as(
            "SELECT mp.date, pm.recipe_id, pm.meal_type, ma.children, ma.teens, ma.adults
             FROM meal_plans mp
             JOIN planned_meals pm ON pm.meal_plan_id = mp.id
             JOIN meal_attendance ma ON ma.planned_meal_id = pm.id
             WHERE mp.camp_id = ?",
        )
        .bind(copied.id)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert_eq!(copied_meal.0, date("2026-09-11"));
        assert_eq!(copied_meal.1, recipe_id);
        assert_eq!(copied_meal.2, "lunch");
        assert_eq!((copied_meal.3, copied_meal.4, copied_meal.5), (11, 6, 4));

        let wrong_length = duplicate_camp(
            &pool,
            source.id,
            camp("Wrong length camp", "2026-10-01", "2026-10-02"),
        )
        .await;
        assert!(wrong_length.is_err());

        let same_name = duplicate_camp(
            &pool,
            source.id,
            camp("Summer camp", "2026-10-01", "2026-10-03"),
        )
        .await;
        assert!(same_name.is_err());
    }

    #[tokio::test]
    async fn delete_camp_removes_associated_days_meals_and_attendance() {
        let pool = setup_pool().await;
        let recipe_id = insert_recipe(&pool, "Pasta").await;
        let deleted = create_camp(&pool, camp("Delete me", "2026-07-01", "2026-07-03"))
            .await
            .unwrap();
        let kept = create_camp(&pool, camp("Keep me", "2026-08-01", "2026-08-03"))
            .await
            .unwrap();

        insert_meal(
            &pool,
            deleted.id,
            date("2026-07-02"),
            recipe_id,
            "lunch",
            (11, 6, 4),
        )
        .await;
        insert_meal(
            &pool,
            kept.id,
            date("2026-08-02"),
            recipe_id,
            "dinner",
            (1, 2, 3),
        )
        .await;

        delete_camp(&pool, deleted.id).await.unwrap();

        let deleted_camp_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM camps WHERE id = ?")
            .bind(deleted.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        let total_plan_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM meal_plans")
            .fetch_one(&pool)
            .await
            .unwrap();
        let total_meal_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM planned_meals")
            .fetch_one(&pool)
            .await
            .unwrap();
        let total_attendance_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM meal_attendance")
                .fetch_one(&pool)
                .await
                .unwrap();
        let kept_plan_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM meal_plans WHERE camp_id = ?")
                .bind(kept.id)
                .fetch_one(&pool)
                .await
                .unwrap();

        assert_eq!(deleted_camp_count, 0);
        assert_eq!(total_plan_count, 1);
        assert_eq!(total_meal_count, 1);
        assert_eq!(total_attendance_count, 1);
        assert_eq!(kept_plan_count, 1);
    }
}
