use crate::models::{
    AttendanceSummary, DailyIngredientItem, IngredientDayUsageItem, MealScheduleItem,
    RecipeIngredientItem, ShoppingListItem,
};
#[cfg(feature = "ssr")]
use chrono::NaiveDate;
use leptos::prelude::*;

#[server(GenerateShoppingList, "/api")]
pub async fn generate_shopping_list(
    camp_id: i64,
    start_date: String,
    end_date: String,
) -> Result<Vec<ShoppingListItem>, ServerFnError<String>> {
    use crate::api::reports;

    let pool = expect_context::<sqlx::SqlitePool>();

    let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?;

    let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?;

    reports::generate_shopping_list(&pool, camp_id, start, end)
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}

#[server(GenerateMealSchedule, "/api")]
pub async fn generate_meal_schedule(
    camp_id: i64,
) -> Result<Vec<MealScheduleItem>, ServerFnError<String>> {
    use crate::api::reports;

    let pool = expect_context::<sqlx::SqlitePool>();

    reports::generate_meal_schedule(&pool, camp_id)
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}

#[server(GenerateAttendanceSummary, "/api")]
pub async fn generate_attendance_summary(
    camp_id: i64,
) -> Result<Vec<AttendanceSummary>, ServerFnError<String>> {
    use crate::api::reports;

    let pool = expect_context::<sqlx::SqlitePool>();

    reports::generate_attendance_summary(&pool, camp_id)
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}

#[server(GenerateDailyIngredients, "/api")]
pub async fn generate_daily_ingredients(
    camp_id: i64,
) -> Result<Vec<DailyIngredientItem>, ServerFnError<String>> {
    use crate::api::reports;

    let pool = expect_context::<sqlx::SqlitePool>();

    reports::generate_daily_ingredients(&pool, camp_id)
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}

#[server(GenerateIngredientsByRecipe, "/api")]
pub async fn generate_ingredients_by_recipe(
    camp_id: i64,
) -> Result<Vec<RecipeIngredientItem>, ServerFnError<String>> {
    use crate::api::reports;

    let pool = expect_context::<sqlx::SqlitePool>();

    reports::generate_ingredients_by_recipe(&pool, camp_id)
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}

#[server(GenerateIngredientUsageByDay, "/api")]
pub async fn generate_ingredient_usage_by_day(
    camp_id: i64,
    ingredient_id: i64,
) -> Result<Vec<IngredientDayUsageItem>, ServerFnError<String>> {
    use crate::api::reports;

    let pool = expect_context::<sqlx::SqlitePool>();

    reports::generate_ingredient_usage_by_day(&pool, camp_id, ingredient_id)
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}

#[server(GenerateReportPdf, "/api")]
pub async fn generate_report_pdf(
    camp_id: i64,
    report_type: String,
    start_date: Option<String>,
    end_date: Option<String>,
    ingredient_id: Option<i64>,
) -> Result<Vec<u8>, ServerFnError<String>> {
    use crate::reports::{ReportPdfKind, generate_report_pdf as build_report_pdf};

    let pool = expect_context::<sqlx::SqlitePool>();
    let kind = ReportPdfKind::try_from(report_type.as_str())
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?;

    let start = match start_date {
        Some(date) => Some(
            NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?,
        ),
        None => None,
    };
    let end = match end_date {
        Some(date) => Some(
            NaiveDate::parse_from_str(&date, "%Y-%m-%d")
                .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?,
        ),
        None => None,
    };

    build_report_pdf(&pool, camp_id, kind, start, end, ingredient_id)
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}
