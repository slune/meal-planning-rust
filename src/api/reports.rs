use crate::api::calc::{Attendance, ingredient_quantity, sum_distinct_meal_attendance};
use crate::models::{
    AttendanceSummary, DailyIngredientItem, IngredientDayUsageItem, MealScheduleItem,
    RecipeIngredientItem, ShoppingListItem,
};
use chrono::NaiveDate;
use sqlx::{AssertSqlSafe, Row, SqlitePool};
use std::collections::HashMap;

/// One recipe-ingredient as it appears at a single planned meal, with the
/// attendance already resolved (override if present, else the camp default).
/// The per-meal quantity is computed in Rust via [`crate::api::calc`] so the
/// math has one tested source of truth instead of living in SQL.
struct MealIngredientRow {
    date: NaiveDate,
    planned_meal_id: i64,
    meal_type: String,
    meal_sort_order: i32,
    recipe_name: String,
    portions: i32,
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
            self.portions,
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
            COALESCE(mt.name, pm.meal_type) as meal_type,
            COALESCE(mt.sort_order, 999) as meal_sort_order,
            r.name as recipe_name,
            r.portions as portions,
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
        LEFT JOIN meal_types mt ON mt.key = pm.meal_type
        LEFT JOIN meal_attendance ma ON pm.id = ma.planned_meal_id
        WHERE mp.camp_id = ?
        "#,
    );
    if date_range.is_some() {
        sql.push_str(" AND mp.date >= ? AND mp.date <= ?");
    }

    let mut query = sqlx::query(AssertSqlSafe(sql)).bind(camp_id);
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
            meal_sort_order: row.get("meal_sort_order"),
            recipe_name: row.get("recipe_name"),
            portions: row.get("portions"),
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
/// then meal type order, recipe, category, ingredient.
pub async fn generate_ingredients_by_recipe(
    pool: &SqlitePool,
    camp_id: i64,
) -> Result<Vec<RecipeIngredientItem>, sqlx::Error> {
    let rows = fetch_meal_ingredients(pool, camp_id, None).await?;
    let day_totals = daily_attendance(&rows);

    let mut items: Vec<(RecipeIngredientItem, i32)> = rows
        .iter()
        .map(|row| {
            let day = day_totals
                .get(&row.date)
                .copied()
                .unwrap_or(Attendance::new(0, 0, 0));
            (
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
                },
                row.meal_sort_order,
            )
        })
        .collect();

    items.sort_by(|a, b| {
        a.0.date
            .cmp(&b.0.date)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.0.meal_type.cmp(&b.0.meal_type))
            .then_with(|| a.0.recipe_name.cmp(&b.0.recipe_name))
            .then_with(|| a.0.category_name.cmp(&b.0.category_name))
            .then_with(|| a.0.ingredient_name.cmp(&b.0.ingredient_name))
    });

    Ok(items.into_iter().map(|(item, _)| item).collect())
}

/// Show every planned meal where a selected ingredient is used, grouped by day
/// by the caller. This is an audit view over the inputs that make up a shopping
/// list total for one ingredient.
pub async fn generate_ingredient_usage_by_day(
    pool: &SqlitePool,
    camp_id: i64,
    ingredient_id: i64,
) -> Result<Vec<IngredientDayUsageItem>, sqlx::Error> {
    let rows = fetch_meal_ingredients(pool, camp_id, None).await?;

    let mut items: Vec<(IngredientDayUsageItem, i32)> = rows
        .iter()
        .filter(|row| row.ingredient_id == ingredient_id)
        .map(|row| {
            (
                IngredientDayUsageItem {
                    date: row.date,
                    meal_type: row.meal_type.clone(),
                    recipe_name: row.recipe_name.clone(),
                    ingredient_name: row.ingredient_name.clone(),
                    children: row.attendance.children,
                    teens: row.attendance.teens,
                    adults: row.attendance.adults,
                    total_people: row.attendance.children
                        + row.attendance.teens
                        + row.attendance.adults,
                    portions: row.portions,
                    base_quantity: row.base_quantity,
                    quantity: row.quantity(),
                    unit: row.unit.clone(),
                },
                row.meal_sort_order,
            )
        })
        .collect();

    items.sort_by(|a, b| {
        a.0.date
            .cmp(&b.0.date)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.0.meal_type.cmp(&b.0.meal_type))
            .then_with(|| a.0.recipe_name.cmp(&b.0.recipe_name))
    });

    Ok(items.into_iter().map(|(item, _)| item).collect())
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
            COALESCE(mt.name, pm.meal_type) as meal_type,
            r.name as recipe_name,
            COALESCE(ma.children, camp.default_children) as children,
            COALESCE(ma.teens, camp.default_teens) as teens,
            COALESCE(ma.adults, camp.default_adults) as adults
        FROM planned_meals pm
        JOIN meal_plans mp ON pm.meal_plan_id = mp.id
        JOIN recipes r ON pm.recipe_id = r.id
        JOIN camps camp ON mp.camp_id = camp.id
        LEFT JOIN meal_types mt ON mt.key = pm.meal_type
        LEFT JOIN meal_attendance ma ON pm.id = ma.planned_meal_id
        WHERE mp.camp_id = ?
        ORDER BY
            mp.date,
            COALESCE(mt.sort_order, 999),
            mt.name,
            pm.meal_type,
            pm.id
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
            COALESCE(mt.name, pm.meal_type) as meal_type,
            COALESCE(ma.children, camp.default_children) as children,
            COALESCE(ma.teens, camp.default_teens) as teens,
            COALESCE(ma.adults, camp.default_adults) as adults
        FROM planned_meals pm
        JOIN meal_plans mp ON pm.meal_plan_id = mp.id
        JOIN camps camp ON mp.camp_id = camp.id
        LEFT JOIN meal_types mt ON mt.key = pm.meal_type
        LEFT JOIN meal_attendance ma ON pm.id = ma.planned_meal_id
        WHERE mp.camp_id = ?
        ORDER BY
            mp.date,
            COALESCE(mt.sort_order, 999),
            mt.name,
            pm.meal_type,
            pm.id
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
            include_str!("../../migrations/009_create_meal_types.sql"),
        ] {
            sqlx::query(migration).execute(&pool).await.unwrap();
        }

        pool
    }

    fn date(value: &str) -> NaiveDate {
        NaiveDate::parse_from_str(value, "%Y-%m-%d").unwrap()
    }

    #[tokio::test]
    async fn generate_ingredient_usage_by_day_filters_selected_ingredient() {
        let pool = setup_pool().await;

        let camp_id = sqlx::query(
            "INSERT INTO camps
             (name, start_date, end_date, default_children, default_teens, default_adults)
             VALUES ('Střediskový tábor 2026', '2026-07-18', '2026-08-01', 22, 24, 28)",
        )
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();

        let bread_id = sqlx::query(
            "INSERT INTO ingredients (name, category_id, primary_unit)
             VALUES ('chleba na kusy', 5, 'ks')",
        )
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();

        let milk_id = sqlx::query(
            "INSERT INTO ingredients (name, category_id, primary_unit)
             VALUES ('mléko', 4, 'l')",
        )
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();

        let recipe_id = sqlx::query(
            "INSERT INTO recipes (name, portions) VALUES ('chléb s tvrdým salámem', 61)",
        )
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();

        for (ingredient_id, quantity, unit) in [(bread_id, 4.64, "ks"), (milk_id, 2.0, "l")] {
            sqlx::query(
                "INSERT INTO recipe_ingredients
                 (recipe_id, ingredient_id, base_quantity, unit, child_multiplier, teen_multiplier, adult_multiplier)
                 VALUES (?, ?, ?, ?, 1.0, 1.0, 1.0)",
            )
            .bind(recipe_id)
            .bind(ingredient_id)
            .bind(quantity)
            .bind(unit)
            .execute(&pool)
            .await
            .unwrap();
        }

        let meal_plan_id =
            sqlx::query("INSERT INTO meal_plans (camp_id, date) VALUES (?, '2026-07-20')")
                .bind(camp_id)
                .execute(&pool)
                .await
                .unwrap()
                .last_insert_rowid();

        let planned_meal_id = sqlx::query(
            "INSERT INTO planned_meals (meal_plan_id, recipe_id, meal_type)
             VALUES (?, ?, 'breakfast')",
        )
        .bind(meal_plan_id)
        .bind(recipe_id)
        .execute(&pool)
        .await
        .unwrap()
        .last_insert_rowid();

        sqlx::query(
            "INSERT INTO meal_attendance (planned_meal_id, children, teens, adults)
             VALUES (?, 2, 45, 26)",
        )
        .bind(planned_meal_id)
        .execute(&pool)
        .await
        .unwrap();

        let items = generate_ingredient_usage_by_day(&pool, camp_id, bread_id)
            .await
            .unwrap();

        assert_eq!(items.len(), 1);
        let item = &items[0];
        assert_eq!(item.date, date("2026-07-20"));
        assert_eq!(item.meal_type, "Breakfast");
        assert_eq!(item.recipe_name, "chléb s tvrdým salámem");
        assert_eq!(item.ingredient_name, "chleba na kusy");
        assert_eq!((item.children, item.teens, item.adults), (2, 45, 26));
        assert_eq!(item.total_people, 73);
        assert_eq!(item.portions, 61);
        assert_eq!(item.base_quantity, 4.64);
        assert_eq!(item.unit, "ks");
        assert!((item.quantity - 5.5527868852459).abs() < 1e-9);
    }
}
