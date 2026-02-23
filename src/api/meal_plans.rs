use crate::models::{
    MealPlan, PlannedMeal, MealAttendance, PlannedMealWithDetails,
    CreatePlannedMeal, UpdatePlannedMeal, CreateAttendance,
};
use chrono::NaiveDate;
use sqlx::SqlitePool;

pub async fn get_meal_plan(
    pool: &SqlitePool,
    camp_id: i64,
    date: NaiveDate,
) -> Result<Option<MealPlan>, sqlx::Error> {
    sqlx::query_as::<_, MealPlan>(
        "SELECT id, camp_id, date, created_at, updated_at 
         FROM meal_plans 
         WHERE camp_id = ? AND date = ?"
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
    if let Some(plan) = get_meal_plan(pool, camp_id, date).await? {
        return Ok(plan);
    }

    let result = sqlx::query(
        "INSERT INTO meal_plans (camp_id, date) VALUES (?, ?)"
    )
    .bind(camp_id)
    .bind(date)
    .execute(pool)
    .await?;

    sqlx::query_as::<_, MealPlan>(
        "SELECT id, camp_id, date, created_at, updated_at 
         FROM meal_plans 
         WHERE id = ?"
    )
    .bind(result.last_insert_rowid())
    .fetch_one(pool)
    .await
}

pub async fn get_planned_meals_for_date(
    pool: &SqlitePool,
    camp_id: i64,
    date: NaiveDate,
) -> Result<Vec<PlannedMealWithDetails>, sqlx::Error> {
    use sqlx::Row;
    
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
         LEFT JOIN meal_attendance ma ON pm.id = ma.planned_meal_id
         WHERE mp.camp_id = ? AND mp.date = ?
         ORDER BY 
            CASE pm.meal_type
                WHEN 'breakfast' THEN 1
                WHEN 'morning_snack' THEN 2
                WHEN 'lunch' THEN 3
                WHEN 'afternoon_snack' THEN 4
                WHEN 'dinner' THEN 5
            END"
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
         ORDER BY date"
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
    let meal_plan = get_or_create_meal_plan(pool, meal.camp_id, meal.date).await?;

    let result = sqlx::query(
        "INSERT INTO planned_meals (meal_plan_id, recipe_id, meal_type)
         VALUES (?, ?, ?)"
    )
    .bind(meal_plan.id)
    .bind(meal.recipe_id)
    .bind(meal.meal_type.as_str())
    .execute(pool)
    .await?;
    let planned_meal_id = result.last_insert_rowid();

    // Handle attendance
    if let Some(attendance) = meal.attendance {
        create_or_update_attendance(pool, planned_meal_id, attendance).await?;
    }

    // Fetch and return the created meal
    let meals = get_planned_meals_for_date(pool, meal.camp_id, meal.date).await?;
    meals.into_iter()
        .find(|m| m.planned_meal.id == planned_meal_id)
        .ok_or_else(|| sqlx::Error::RowNotFound)
}

pub async fn update_planned_meal(
    pool: &SqlitePool,
    id: i64,
    update: UpdatePlannedMeal,
) -> Result<(), sqlx::Error> {
    if let Some(recipe_id) = update.recipe_id {
        sqlx::query("UPDATE planned_meals SET recipe_id = ? WHERE id = ?")
            .bind(recipe_id)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(attendance) = update.attendance {
        create_or_update_attendance(pool, id, attendance).await?;
    }

    Ok(())
}

async fn create_or_update_attendance(
    pool: &SqlitePool,
    planned_meal_id: i64,
    attendance: CreateAttendance,
) -> Result<(), sqlx::Error> {
    // Validate attendance counts are non-negative
    if attendance.children < 0 {
        return Err(sqlx::Error::Decode(
            "Number of children cannot be negative".into()
        ));
    }
    if attendance.teens < 0 {
        return Err(sqlx::Error::Decode(
            "Number of teens cannot be negative".into()
        ));
    }
    if attendance.adults < 0 {
        return Err(sqlx::Error::Decode(
            "Number of adults cannot be negative".into()
        ));
    }

    sqlx::query(
        "INSERT INTO meal_attendance (planned_meal_id, children, teens, adults)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(planned_meal_id) DO UPDATE SET
            children = excluded.children,
            teens = excluded.teens,
            adults = excluded.adults,
            updated_at = CURRENT_TIMESTAMP"
    )
    .bind(planned_meal_id)
    .bind(attendance.children)
    .bind(attendance.teens)
    .bind(attendance.adults)
    .execute(pool)
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
