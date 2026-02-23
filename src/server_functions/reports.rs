use leptos::prelude::*;
use crate::models::{ShoppingListItem, MealScheduleItem, AttendanceSummary};
use chrono::NaiveDate;

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
