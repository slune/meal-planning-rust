use leptos::prelude::*;
use crate::models::Category;

#[server(GetCategories, "/api")]
pub async fn get_categories() -> Result<Vec<Category>, ServerFnError> {
    use crate::api::categories;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    categories::get_categories(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(GetCategory, "/api")]
pub async fn get_category(id: i64) -> Result<Category, ServerFnError> {
    use crate::api::categories;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    categories::get_category(&pool, id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(CreateCategoryFn, "/api")]
pub async fn create_category(
    name: String,
    sort_order: i32,
) -> Result<Category, ServerFnError> {
    use crate::api::categories;
    use crate::models::CreateCategory;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    let new_category = CreateCategory {
        name,
        sort_order,
    };
    
    categories::create_category(&pool, new_category)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(UpdateCategoryFn, "/api")]
pub async fn update_category(
    id: i64,
    name: Option<String>,
    sort_order: Option<i32>,
) -> Result<Category, ServerFnError> {
    use crate::api::categories;
    use crate::models::UpdateCategory;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    let update = UpdateCategory {
        name,
        sort_order,
    };
    
    categories::update_category(&pool, id, update)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(DeleteCategory, "/api")]
pub async fn delete_category(id: i64) -> Result<(), ServerFnError> {
    use crate::api::categories;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    categories::delete_category(&pool, id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}
