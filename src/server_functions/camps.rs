use crate::models::Camp;
use chrono::NaiveDate;
use leptos::prelude::*;
use leptos::server_fn::error::NoCustomError;

#[server(GetCamps, "/api")]
pub async fn get_camps() -> Result<Vec<Camp>, ServerFnError> {
    use crate::api::camps;
    let pool = expect_context::<sqlx::SqlitePool>();

    camps::get_camps(&pool)
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))
}

#[server(GetCamp, "/api")]
pub async fn get_camp(id: i64) -> Result<Camp, ServerFnError> {
    use crate::api::camps;
    let pool = expect_context::<sqlx::SqlitePool>();

    camps::get_camp(&pool, id)
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))
}

#[server(CreateCampFn, "/api")]
pub async fn create_camp(
    name: String,
    start_date: String,
    end_date: String,
    default_children: i32,
    default_teens: i32,
    default_adults: i32,
    notes: Option<String>,
) -> Result<Camp, ServerFnError> {
    use crate::api::camps;
    use crate::models::CreateCamp;
    let pool = expect_context::<sqlx::SqlitePool>();

    let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;
    let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))?;

    let new_camp = CreateCamp {
        name,
        start_date: start,
        end_date: end,
        default_children,
        default_teens,
        default_adults,
        notes,
    };

    camps::create_camp(&pool, new_camp)
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))
}

#[server(DeleteCamp, "/api")]
pub async fn delete_camp(id: i64) -> Result<(), ServerFnError> {
    use crate::api::camps;
    let pool = expect_context::<sqlx::SqlitePool>();

    camps::delete_camp(&pool, id)
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.to_string()))
}
