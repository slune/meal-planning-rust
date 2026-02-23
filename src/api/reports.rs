use crate::models::{ShoppingListItem, MealScheduleItem, AttendanceSummary};
use chrono::NaiveDate;
use sqlx::{SqlitePool, Row};

/// Generate shopping list for a camp within a date range
pub async fn generate_shopping_list(
    pool: &SqlitePool,
    camp_id: i64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<ShoppingListItem>, sqlx::Error> {
    // Query to aggregate ingredients across all meals in the date range
    let rows = sqlx::query(
        r#"
        SELECT
            i.id as ingredient_id,
            i.name as ingredient_name,
            c.name as category_name,
            ri.unit,
            SUM(
                ri.base_quantity * (
                    COALESCE(ri.child_multiplier, 1.0) * COALESCE(ma.children, camp.default_children) +
                    COALESCE(ri.teen_multiplier, 1.0) * COALESCE(ma.teens, camp.default_teens) +
                    COALESCE(ri.adult_multiplier, 1.0) * COALESCE(ma.adults, camp.default_adults)
                ) / r.base_servings
            ) as total_quantity
        FROM planned_meals pm
        JOIN meal_plans mp ON pm.meal_plan_id = mp.id
        JOIN recipes r ON pm.recipe_id = r.id
        JOIN recipe_ingredients ri ON r.id = ri.recipe_id
        JOIN ingredients i ON ri.ingredient_id = i.id
        JOIN categories c ON i.category_id = c.id
        JOIN camps camp ON mp.camp_id = camp.id
        LEFT JOIN meal_attendance ma ON pm.id = ma.planned_meal_id
        WHERE mp.camp_id = ?
            AND mp.date >= ?
            AND mp.date <= ?
        GROUP BY i.id, ri.unit, c.name
        ORDER BY c.name, i.name
        "#
    )
    .bind(camp_id)
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    let items = rows.into_iter().map(|row| {
        ShoppingListItem {
            ingredient_id: row.get("ingredient_id"),
            ingredient_name: row.get("ingredient_name"),
            category_name: row.get("category_name"),
            total_quantity: row.get("total_quantity"),
            unit: row.get("unit"),
        }
    }).collect();

    Ok(items)
}

/// Generate meal schedule for a camp
pub async fn generate_meal_schedule(
    pool: &SqlitePool,
    camp_id: i64,
) -> Result<Vec<MealScheduleItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            mp.date,
            pm.meal_type,
            r.name as recipe_name,
            COALESCE(ma.children, camp.default_children) as children,
            COALESCE(ma.teens, camp.default_teens) as teens,
            COALESCE(ma.adults, camp.default_adults) as adults
        FROM planned_meals pm
        JOIN meal_plans mp ON pm.meal_plan_id = mp.id
        JOIN recipes r ON pm.recipe_id = r.id
        JOIN camps camp ON mp.camp_id = camp.id
        LEFT JOIN meal_attendance ma ON pm.id = ma.planned_meal_id
        WHERE mp.camp_id = ?
        ORDER BY mp.date, pm.meal_type
        "#
    )
    .bind(camp_id)
    .fetch_all(pool)
    .await?;

    let items = rows.into_iter().map(|row| {
        MealScheduleItem {
            date: row.get("date"),
            meal_type: row.get("meal_type"),
            recipe_name: row.get("recipe_name"),
            children: row.get("children"),
            teens: row.get("teens"),
            adults: row.get("adults"),
        }
    }).collect();

    Ok(items)
}

/// Generate attendance summary for a camp
pub async fn generate_attendance_summary(
    pool: &SqlitePool,
    camp_id: i64,
) -> Result<Vec<AttendanceSummary>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT
            mp.date,
            pm.meal_type,
            COALESCE(ma.children, camp.default_children) as children,
            COALESCE(ma.teens, camp.default_teens) as teens,
            COALESCE(ma.adults, camp.default_adults) as adults
        FROM planned_meals pm
        JOIN meal_plans mp ON pm.meal_plan_id = mp.id
        JOIN camps camp ON mp.camp_id = camp.id
        LEFT JOIN meal_attendance ma ON pm.id = ma.planned_meal_id
        WHERE mp.camp_id = ?
        ORDER BY mp.date, pm.meal_type
        "#
    )
    .bind(camp_id)
    .fetch_all(pool)
    .await?;

    let items = rows.into_iter().map(|row| {
        let children: i32 = row.get("children");
        let teens: i32 = row.get("teens");
        let adults: i32 = row.get("adults");

        AttendanceSummary {
            date: row.get("date"),
            meal_type: row.get("meal_type"),
            children,
            teens,
            adults,
            total_people: children + teens + adults,
        }
    }).collect();

    Ok(items)
}
