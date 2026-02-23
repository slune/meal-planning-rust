use crate::models::PlannedMealWithDetails;
use leptos::prelude::*;
use leptos::server_fn::error::NoCustomError;

#[server(GetPlannedMealsForDate, "/api")]
pub async fn get_planned_meals_for_date(
    camp_id: i64,
    date: String,
) -> Result<Vec<PlannedMealWithDetails>, ServerFnError> {
    use crate::api::meal_plans;
    use chrono::NaiveDate;
    let pool = expect_context::<sqlx::SqlitePool>();

    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;

    meal_plans::get_planned_meals_for_date(&pool, camp_id, parsed_date)
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))
}

#[server(CreatePlannedMealFn, "/api")]
pub async fn create_planned_meal(
    camp_id: i64,
    date: String,
    meal_type: String,
    recipe_id: i64,
    children: Option<i32>,
    teens: Option<i32>,
    adults: Option<i32>,
) -> Result<PlannedMealWithDetails, ServerFnError> {
    use crate::api::meal_plans;
    use crate::models::{CreateAttendance, CreatePlannedMeal, MealType};
    use chrono::NaiveDate;
    let pool = expect_context::<sqlx::SqlitePool>();

    let parsed_date = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;

    let parsed_meal_type = MealType::from_str(&meal_type).ok_or_else(|| {
        ServerFnError::<NoCustomError>::ServerError("Invalid meal type".to_string())
    })?;

    let attendance = if let (Some(c), Some(t), Some(a)) = (children, teens, adults) {
        Some(CreateAttendance {
            children: c,
            teens: t,
            adults: a,
        })
    } else {
        None
    };

    let new_meal = CreatePlannedMeal {
        camp_id,
        date: parsed_date,
        meal_type: parsed_meal_type,
        recipe_id,
        attendance,
    };

    meal_plans::create_planned_meal(&pool, new_meal)
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))
}

#[server(UpdatePlannedMealFn, "/api")]
pub async fn update_planned_meal(
    id: i64,
    recipe_id: i64,
    children: Option<i32>,
    teens: Option<i32>,
    adults: Option<i32>,
) -> Result<(), ServerFnError<String>> {
    use crate::api::meal_plans;
    use crate::models::{CreateAttendance, UpdatePlannedMeal};
    let pool = expect_context::<sqlx::SqlitePool>();

    let attendance = if let (Some(c), Some(t), Some(a)) = (children, teens, adults) {
        Some(CreateAttendance {
            children: c,
            teens: t,
            adults: a,
        })
    } else {
        None
    };

    let update = UpdatePlannedMeal {
        recipe_id: Some(recipe_id),
        attendance,
    };

    meal_plans::update_planned_meal(&pool, id, update)
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}

#[server(DeletePlannedMeal, "/api")]
pub async fn delete_planned_meal(id: i64) -> Result<(), ServerFnError> {
    use crate::api::meal_plans;
    let pool = expect_context::<sqlx::SqlitePool>();

    meal_plans::delete_planned_meal(&pool, id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(GetPlannedMealsForCamp, "/api")]
pub async fn get_planned_meals_for_camp(
    camp_id: i64,
) -> Result<Vec<(String, Vec<PlannedMealWithDetails>)>, ServerFnError<String>> {
    use crate::api::meal_plans;
    let pool = expect_context::<sqlx::SqlitePool>();

    meal_plans::get_planned_meals_for_camp(&pool, camp_id)
        .await
        .map(|meals| {
            meals.into_iter()
                .map(|(date, meals)| (date.format("%Y-%m-%d").to_string(), meals))
                .collect()
        })
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))
}

#[server(GetPlannedMealsForDateRange, "/api")]
pub async fn get_planned_meals_for_date_range(
    camp_id: i64,
    start_date: String,
    end_date: String,
) -> Result<Vec<(String, Vec<PlannedMealWithDetails>)>, ServerFnError<String>> {
    use crate::api::meal_plans;
    use chrono::NaiveDate;
    let pool = expect_context::<sqlx::SqlitePool>();

    let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?;
    let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?;

    // Get all meals for camp and filter by date range
    let all_meals = meal_plans::get_planned_meals_for_camp(&pool, camp_id)
        .await
        .map_err(|e| ServerFnError::<String>::ServerError(e.to_string()))?;

    Ok(all_meals
        .into_iter()
        .filter(|(date, _)| *date >= start && *date <= end)
        .map(|(date, meals)| (date.format("%Y-%m-%d").to_string(), meals))
        .collect())
}
