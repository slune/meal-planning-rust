use crate::models::{
    CreateAttendance, CreatePlannedMeal, MealAttendance, MealPlan, MealType, PlannedMeal,
    PlannedMealWithDetails, UpdatePlannedMeal,
};
use chrono::NaiveDate;
use sqlx::{Row, Sqlite, SqlitePool};

pub async fn get_meal_plan(
    pool: &SqlitePool,
    camp_id: i64,
    date: NaiveDate,
) -> Result<Option<MealPlan>, sqlx::Error> {
    sqlx::query_as::<_, MealPlan>(
        "SELECT id, camp_id, date, created_at, updated_at 
         FROM meal_plans 
         WHERE camp_id = ? AND date = ?",
    )
    .bind(camp_id)
    .bind(date)
    .fetch_optional(pool)
    .await
}

async fn get_or_create_meal_plan(
    pool: &SqlitePool,
    camp_id: i64,
    date: NaiveDate,
) -> Result<MealPlan, sqlx::Error> {
    sqlx::query(
        "INSERT INTO meal_plans (camp_id, date)
         VALUES (?, ?)
         ON CONFLICT(camp_id, date) DO NOTHING",
    )
    .bind(camp_id)
    .bind(date)
    .execute(pool)
    .await?;

    get_meal_plan(pool, camp_id, date)
        .await?
        .ok_or(sqlx::Error::RowNotFound)
}

pub async fn get_meal_types(pool: &SqlitePool) -> Result<Vec<MealType>, sqlx::Error> {
    sqlx::query_as::<_, MealType>(
        "SELECT id, key, name, sort_order, created_at, updated_at
         FROM meal_types
         ORDER BY sort_order, name",
    )
    .fetch_all(pool)
    .await
}

pub async fn create_meal_type(
    pool: &SqlitePool,
    name: String,
    sort_order: Option<i32>,
) -> Result<MealType, sqlx::Error> {
    validate_meal_type_name(&name)?;
    let key = meal_type_key(&name);

    let key_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meal_types WHERE key = ?)")
            .bind(&key)
            .fetch_one(pool)
            .await?;

    if key_exists {
        return Err(validation_error("Meal type already exists"));
    }

    let final_sort_order = match sort_order {
        Some(value) => {
            validate_meal_type_sort_order(value)?;
            value
        }
        None => {
            let max_order: Option<i32> =
                sqlx::query_scalar("SELECT MAX(sort_order) FROM meal_types")
                    .fetch_one(pool)
                    .await?;
            max_order.unwrap_or(0) + 1
        }
    };

    let result = sqlx::query(
        "INSERT INTO meal_types (key, name, sort_order)
         VALUES (?, ?, ?)",
    )
    .bind(&key)
    .bind(name.trim())
    .bind(final_sort_order)
    .execute(pool)
    .await?;

    get_meal_type(pool, result.last_insert_rowid()).await
}

pub async fn update_meal_type(
    pool: &SqlitePool,
    id: i64,
    name: String,
    sort_order: i32,
) -> Result<MealType, sqlx::Error> {
    validate_meal_type_name(&name)?;
    validate_meal_type_sort_order(sort_order)?;

    sqlx::query(
        "UPDATE meal_types
         SET name = ?, sort_order = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(name.trim())
    .bind(sort_order)
    .bind(id)
    .execute(pool)
    .await?;

    get_meal_type(pool, id).await
}

pub async fn delete_meal_type(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    let meal_type = get_meal_type(pool, id).await?;

    let total_types: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM meal_types")
        .fetch_one(pool)
        .await?;
    if total_types <= 1 {
        return Err(validation_error("At least one meal type is required"));
    }

    let usage_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM planned_meals WHERE meal_type = ?")
            .bind(&meal_type.key)
            .fetch_one(pool)
            .await?;
    if usage_count > 0 {
        return Err(validation_error(
            "Cannot delete a meal type that is used by planned meals",
        ));
    }

    sqlx::query("DELETE FROM meal_types WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

async fn get_meal_type(pool: &SqlitePool, id: i64) -> Result<MealType, sqlx::Error> {
    sqlx::query_as::<_, MealType>(
        "SELECT id, key, name, sort_order, created_at, updated_at
         FROM meal_types
         WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

async fn validate_meal_type_exists(pool: &SqlitePool, key: &str) -> Result<(), sqlx::Error> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM meal_types WHERE key = ?)")
        .bind(key)
        .fetch_one(pool)
        .await?;

    if !exists {
        return Err(validation_error("Invalid meal type"));
    }

    Ok(())
}

fn validate_meal_type_name(name: &str) -> Result<(), sqlx::Error> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err(validation_error("Meal type name is required"));
    }
    if trimmed.chars().count() > 80 {
        return Err(validation_error(
            "Meal type name must be 80 characters or less",
        ));
    }
    if meal_type_key(trimmed).is_empty() {
        return Err(validation_error(
            "Meal type name must include at least one letter or number",
        ));
    }
    Ok(())
}

fn validate_meal_type_sort_order(sort_order: i32) -> Result<(), sqlx::Error> {
    if sort_order <= 0 {
        return Err(validation_error("Meal type order must be greater than 0"));
    }
    Ok(())
}

fn validation_error(message: &str) -> sqlx::Error {
    sqlx::Error::Decode(message.to_string().into())
}

fn meal_type_key(name: &str) -> String {
    let mut key = String::new();
    let mut last_was_separator = true;

    for ch in name.trim().chars().flat_map(char::to_lowercase) {
        if ch.is_alphanumeric() {
            key.push(ch);
            last_was_separator = false;
        } else if !last_was_separator {
            key.push('_');
            last_was_separator = true;
        }
    }

    while key.ends_with('_') {
        key.pop();
    }

    key
}

pub async fn get_planned_meals_for_date(
    pool: &SqlitePool,
    camp_id: i64,
    date: NaiveDate,
) -> Result<Vec<PlannedMealWithDetails>, sqlx::Error> {
    let rows = sqlx::query(
        "SELECT 
            pm.id, pm.meal_plan_id, pm.recipe_id, pm.meal_type, pm.created_at,
            r.name as recipe_name,
            ma.id as attendance_id, ma.planned_meal_id as attendance_planned_meal_id,
            ma.children, ma.teens, ma.adults, 
            ma.created_at as attendance_created_at, ma.updated_at as attendance_updated_at
         FROM meal_plans mp
         JOIN planned_meals pm ON mp.id = pm.meal_plan_id
         JOIN recipes r ON pm.recipe_id = r.id
         LEFT JOIN meal_types mt ON mt.key = pm.meal_type
         LEFT JOIN meal_attendance ma ON pm.id = ma.planned_meal_id
         WHERE mp.camp_id = ? AND mp.date = ?
         ORDER BY 
            COALESCE(mt.sort_order, 999),
            mt.name,
            pm.meal_type,
            pm.id",
    )
    .bind(camp_id)
    .bind(date)
    .fetch_all(pool)
    .await?;

    let mut results = Vec::new();
    for row in rows {
        let planned_meal = PlannedMeal {
            id: row.try_get("id")?,
            meal_plan_id: row.try_get("meal_plan_id")?,
            recipe_id: row.try_get("recipe_id")?,
            meal_type: row.try_get("meal_type")?,
            created_at: row.try_get("created_at").ok(),
        };

        let attendance = if let Ok(attendance_id) = row.try_get::<i64, _>("attendance_id") {
            Some(MealAttendance {
                id: attendance_id,
                planned_meal_id: row.try_get("attendance_planned_meal_id")?,
                children: row.try_get("children")?,
                teens: row.try_get("teens")?,
                adults: row.try_get("adults")?,
                created_at: row.try_get("attendance_created_at").ok(),
                updated_at: row.try_get("attendance_updated_at").ok(),
            })
        } else {
            None
        };

        results.push(PlannedMealWithDetails {
            planned_meal,
            recipe_name: row.try_get("recipe_name")?,
            attendance,
        });
    }

    Ok(results)
}

pub async fn get_planned_meals_for_camp(
    pool: &SqlitePool,
    camp_id: i64,
) -> Result<Vec<(NaiveDate, Vec<PlannedMealWithDetails>)>, sqlx::Error> {
    let meal_plans = sqlx::query_as::<_, MealPlan>(
        "SELECT id, camp_id, date, created_at, updated_at 
         FROM meal_plans 
         WHERE camp_id = ?
         ORDER BY date",
    )
    .bind(camp_id)
    .fetch_all(pool)
    .await?;

    let mut result = Vec::new();
    for plan in meal_plans {
        let meals = get_planned_meals_for_date(pool, camp_id, plan.date).await?;
        result.push((plan.date, meals));
    }

    Ok(result)
}

pub async fn create_planned_meal(
    pool: &SqlitePool,
    meal: CreatePlannedMeal,
) -> Result<PlannedMealWithDetails, sqlx::Error> {
    validate_meal_date(pool, meal.camp_id, meal.date).await?;
    validate_meal_type_exists(pool, &meal.meal_type).await?;
    if let Some(attendance) = meal.attendance.as_ref() {
        validate_attendance(attendance)?;
    }

    let meal_plan = get_or_create_meal_plan(pool, meal.camp_id, meal.date).await?;
    let mut tx = pool.begin().await?;

    let result = sqlx::query(
        "INSERT INTO planned_meals (meal_plan_id, recipe_id, meal_type)
         VALUES (?, ?, ?)",
    )
    .bind(meal_plan.id)
    .bind(meal.recipe_id)
    .bind(&meal.meal_type)
    .execute(&mut *tx)
    .await?;
    let planned_meal_id = result.last_insert_rowid();

    // Handle attendance
    if let Some(attendance) = meal.attendance {
        create_or_update_attendance(&mut *tx, planned_meal_id, attendance).await?;
    }

    tx.commit().await?;

    // Fetch and return the created meal
    let meals = get_planned_meals_for_date(pool, meal.camp_id, meal.date).await?;
    meals
        .into_iter()
        .find(|m| m.planned_meal.id == planned_meal_id)
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

pub async fn update_planned_meal(
    pool: &SqlitePool,
    id: i64,
    update: UpdatePlannedMeal,
) -> Result<(), sqlx::Error> {
    if let Some(meal_type) = update.meal_type.as_ref() {
        validate_meal_type_exists(pool, meal_type).await?;
    }

    if let Some(attendance) = update.attendance.as_ref() {
        validate_attendance(attendance)?;
    }

    let mut tx = pool.begin().await?;

    if let Some(meal_type) = update.meal_type {
        sqlx::query("UPDATE planned_meals SET meal_type = ? WHERE id = ?")
            .bind(meal_type)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(recipe_id) = update.recipe_id {
        sqlx::query("UPDATE planned_meals SET recipe_id = ? WHERE id = ?")
            .bind(recipe_id)
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }

    if let Some(attendance) = update.attendance {
        create_or_update_attendance(&mut *tx, id, attendance).await?;
    }

    tx.commit().await?;

    Ok(())
}

async fn validate_meal_date(
    pool: &SqlitePool,
    camp_id: i64,
    date: NaiveDate,
) -> Result<(), sqlx::Error> {
    let camp = crate::api::camps::get_camp(pool, camp_id).await?;
    if date < camp.start_date || date > camp.end_date {
        return Err(sqlx::Error::Decode(
            format!(
                "Meal date must be within camp range ({} to {})",
                camp.start_date, camp.end_date
            )
            .into(),
        ));
    }

    Ok(())
}

fn validate_attendance(attendance: &CreateAttendance) -> Result<(), sqlx::Error> {
    // Validate attendance counts are non-negative
    if attendance.children < 0 {
        return Err(sqlx::Error::Decode(
            "Number of children cannot be negative".into(),
        ));
    }
    if attendance.teens < 0 {
        return Err(sqlx::Error::Decode(
            "Number of teens cannot be negative".into(),
        ));
    }
    if attendance.adults < 0 {
        return Err(sqlx::Error::Decode(
            "Number of adults cannot be negative".into(),
        ));
    }

    Ok(())
}

async fn create_or_update_attendance<'e, E>(
    executor: E,
    planned_meal_id: i64,
    attendance: CreateAttendance,
) -> Result<(), sqlx::Error>
where
    E: sqlx::Executor<'e, Database = Sqlite>,
{
    sqlx::query(
        "INSERT INTO meal_attendance (planned_meal_id, children, teens, adults)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(planned_meal_id) DO UPDATE SET
            children = excluded.children,
            teens = excluded.teens,
            adults = excluded.adults,
            updated_at = CURRENT_TIMESTAMP",
    )
    .bind(planned_meal_id)
    .bind(attendance.children)
    .bind(attendance.teens)
    .bind(attendance.adults)
    .execute(executor)
    .await?;

    Ok(())
}

pub async fn delete_planned_meal(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM planned_meals WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
