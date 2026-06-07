use crate::api::calc::{Attendance, ingredient_quantity, sum_distinct_meal_attendance};
use crate::models::{
    AttendanceSummary, DailyIngredientItem, MealScheduleItem, MealType, RecipeIngredientItem,
    ShoppingListItem,
};
use chrono::NaiveDate;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;

/// One recipe-ingredient as it appears at a single planned meal, with the
/// attendance already resolved (override if present, else the camp default).
/// The per-meal quantity is computed in Rust via [`crate::api::calc`] so the
/// math has one tested source of truth instead of living in SQL.
struct MealIngredientRow {
    date: NaiveDate,
    planned_meal_id: i64,
    meal_type: String,
    recipe_name: String,
    base_servings: i32,
    ingredient_id: i64,
    ingredient_name: String,
    category_name: String,
    unit: String,
    base_quantity: f64,
    child_multiplier: Option<f64>,
    teen_multiplier: Option<f64>,
    adult_multiplier: Option<f64>,
    attendance: Attendance,
}

impl MealIngredientRow {
    /// Quantity of this ingredient needed for this single meal.
    fn quantity(&self) -> f64 {
        ingredient_quantity(
            self.base_quantity,
            self.base_servings,
            self.attendance,
            self.child_multiplier,
            self.teen_multiplier,
            self.adult_multiplier,
        )
    }
}

/// Fetch every (planned meal × recipe ingredient) row for a camp, optionally
/// restricted to a date range. Shared by all ingredient-quantity reports.
async fn fetch_meal_ingredients(
    pool: &SqlitePool,
    camp_id: i64,
    date_range: Option<(NaiveDate, NaiveDate)>,
) -> Result<Vec<MealIngredientRow>, sqlx::Error> {
    let mut sql = String::from(
        r#"
        SELECT
            mp.date as date,
            pm.id as planned_meal_id,
            pm.meal_type as meal_type,
            r.name as recipe_name,
            r.base_servings as base_servings,
            i.id as ingredient_id,
            i.name as ingredient_name,
            c.name as category_name,
            ri.unit as unit,
            ri.base_quantity as base_quantity,
            ri.child_multiplier as child_multiplier,
            ri.teen_multiplier as teen_multiplier,
            ri.adult_multiplier as adult_multiplier,
            COALESCE(ma.children, camp.default_children) as children,
            COALESCE(ma.teens, camp.default_teens) as teens,
            COALESCE(ma.adults, camp.default_adults) as adults
        FROM planned_meals pm
        JOIN meal_plans mp ON pm.meal_plan_id = mp.id
        JOIN recipes r ON pm.recipe_id = r.id
        JOIN recipe_ingredients ri ON r.id = ri.recipe_id
        JOIN ingredients i ON ri.ingredient_id = i.id
        JOIN categories c ON i.category_id = c.id
        JOIN camps camp ON mp.camp_id = camp.id
        LEFT JOIN meal_attendance ma ON pm.id = ma.planned_meal_id
        WHERE mp.camp_id = ?
        "#,
    );
    if date_range.is_some() {
        sql.push_str(" AND mp.date >= ? AND mp.date <= ?");
    }

    let mut query = sqlx::query(&sql).bind(camp_id);
    if let Some((start, end)) = date_range {
        query = query.bind(start).bind(end);
    }

    let rows = query.fetch_all(pool).await?;

    Ok(rows
        .into_iter()
        .map(|row| MealIngredientRow {
            date: row.get("date"),
            planned_meal_id: row.get("planned_meal_id"),
            meal_type: row.get("meal_type"),
            recipe_name: row.get("recipe_name"),
            base_servings: row.get("base_servings"),
            ingredient_id: row.get("ingredient_id"),
            ingredient_name: row.get("ingredient_name"),
            category_name: row.get("category_name"),
            unit: row.get("unit"),
            base_quantity: row.get("base_quantity"),
            child_multiplier: row.get("child_multiplier"),
            teen_multiplier: row.get("teen_multiplier"),
            adult_multiplier: row.get("adult_multiplier"),
            attendance: Attendance::new(row.get("children"), row.get("teens"), row.get("adults")),
        })
        .collect())
}

/// Generate shopping list for a camp within a date range.
///
/// Ingredients are aggregated across every meal in the range and summed per
/// (ingredient, unit). The per-meal math runs through the tested helper in
/// [`crate::api::calc`].
pub async fn generate_shopping_list(
    pool: &SqlitePool,
    camp_id: i64,
    start_date: NaiveDate,
    end_date: NaiveDate,
) -> Result<Vec<ShoppingListItem>, sqlx::Error> {
    let rows = fetch_meal_ingredients(pool, camp_id, Some((start_date, end_date))).await?;

    // Sum per (ingredient_id, unit).
    let mut totals: HashMap<(i64, String), ShoppingListItem> = HashMap::new();
    for row in &rows {
        let entry = totals
            .entry((row.ingredient_id, row.unit.clone()))
            .or_insert_with(|| ShoppingListItem {
                ingredient_id: row.ingredient_id,
                ingredient_name: row.ingredient_name.clone(),
                category_name: row.category_name.clone(),
                total_quantity: 0.0,
                unit: row.unit.clone(),
            });
        entry.total_quantity += row.quantity();
    }

    let mut items: Vec<ShoppingListItem> = totals.into_values().collect();
    items.sort_by(|a, b| {
        a.category_name
            .cmp(&b.category_name)
            .then_with(|| a.ingredient_name.cmp(&b.ingredient_name))
    });

    Ok(items)
}

/// Total person-meals per day: attendance summed over the distinct meals
/// planned that day (the "počet jídel" shown in each day header).
fn daily_attendance(rows: &[MealIngredientRow]) -> HashMap<NaiveDate, Attendance> {
    let mut by_day: HashMap<NaiveDate, Vec<(i64, Attendance)>> = HashMap::new();
    for row in rows {
        by_day
            .entry(row.date)
            .or_default()
            .push((row.planned_meal_id, row.attendance));
    }
    by_day
        .into_iter()
        .map(|(date, pairs)| (date, sum_distinct_meal_attendance(pairs)))
        .collect()
}

/// Total ingredients required for each day, aggregated across every meal that
/// day. Sorted by date, then category, then ingredient.
pub async fn generate_daily_ingredients(
    pool: &SqlitePool,
    camp_id: i64,
) -> Result<Vec<DailyIngredientItem>, sqlx::Error> {
    let rows = fetch_meal_ingredients(pool, camp_id, None).await?;
    let day_totals = daily_attendance(&rows);

    // Sum per (date, ingredient_id, unit).
    let mut totals: HashMap<(NaiveDate, i64, String), DailyIngredientItem> = HashMap::new();
    for row in &rows {
        let day = day_totals
            .get(&row.date)
            .copied()
            .unwrap_or(Attendance::new(0, 0, 0));
        let entry = totals
            .entry((row.date, row.ingredient_id, row.unit.clone()))
            .or_insert_with(|| DailyIngredientItem {
                date: row.date,
                day_children: day.children,
                day_teens: day.teens,
                day_adults: day.adults,
                category_name: row.category_name.clone(),
                ingredient_name: row.ingredient_name.clone(),
                total_quantity: 0.0,
                unit: row.unit.clone(),
            });
        entry.total_quantity += row.quantity();
    }

    let mut items: Vec<DailyIngredientItem> = totals.into_values().collect();
    items.sort_by(|a, b| {
        a.date
            .cmp(&b.date)
            .then_with(|| a.category_name.cmp(&b.category_name))
            .then_with(|| a.ingredient_name.cmp(&b.ingredient_name))
    });

    Ok(items)
}

/// Ingredient amounts broken down by day → meal → recipe. One entry per recipe
/// ingredient at each meal (no aggregation across recipes). Sorted by date,
/// then meal order (breakfast → dinner), recipe, category, ingredient.
pub async fn generate_ingredients_by_recipe(
    pool: &SqlitePool,
    camp_id: i64,
) -> Result<Vec<RecipeIngredientItem>, sqlx::Error> {
    let rows = fetch_meal_ingredients(pool, camp_id, None).await?;
    let day_totals = daily_attendance(&rows);

    let mut items: Vec<RecipeIngredientItem> = rows
        .iter()
        .map(|row| {
            let day = day_totals
                .get(&row.date)
                .copied()
                .unwrap_or(Attendance::new(0, 0, 0));
            RecipeIngredientItem {
                date: row.date,
                day_children: day.children,
                day_teens: day.teens,
                day_adults: day.adults,
                meal_type: row.meal_type.clone(),
                recipe_name: row.recipe_name.clone(),
                category_name: row.category_name.clone(),
                ingredient_name: row.ingredient_name.clone(),
                quantity: row.quantity(),
                unit: row.unit.clone(),
            }
        })
        .collect();

    items.sort_by(|a, b| {
        a.date
            .cmp(&b.date)
            .then_with(|| meal_type_order(&a.meal_type).cmp(&meal_type_order(&b.meal_type)))
            .then_with(|| a.recipe_name.cmp(&b.recipe_name))
            .then_with(|| a.category_name.cmp(&b.category_name))
            .then_with(|| a.ingredient_name.cmp(&b.ingredient_name))
    });

    Ok(items)
}

/// Chronological meal ordering (breakfast first). Unknown meal types sort last.
fn meal_type_order(meal_type: &str) -> u8 {
    meal_type
        .parse::<MealType>()
        .map(|m| m.sort_order())
        .unwrap_or(u8::MAX)
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
        ORDER BY
            mp.date,
            CASE pm.meal_type
                WHEN 'breakfast' THEN 1
                WHEN 'morning_snack' THEN 2
                WHEN 'lunch' THEN 3
                WHEN 'afternoon_snack' THEN 4
                WHEN 'dinner' THEN 5
                ELSE 99
            END
        "#,
    )
    .bind(camp_id)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|row| MealScheduleItem {
            date: row.get("date"),
            meal_type: row.get("meal_type"),
            recipe_name: row.get("recipe_name"),
            children: row.get("children"),
            teens: row.get("teens"),
            adults: row.get("adults"),
        })
        .collect();

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
        ORDER BY
            mp.date,
            CASE pm.meal_type
                WHEN 'breakfast' THEN 1
                WHEN 'morning_snack' THEN 2
                WHEN 'lunch' THEN 3
                WHEN 'afternoon_snack' THEN 4
                WHEN 'dinner' THEN 5
                ELSE 99
            END
        "#,
    )
    .bind(camp_id)
    .fetch_all(pool)
    .await?;

    let items = rows
        .into_iter()
        .map(|row| {
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
        })
        .collect();

    Ok(items)
}
