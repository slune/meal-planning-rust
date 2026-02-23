use leptos::prelude::*;
use crate::models::Ingredient;

#[server(GetIngredients, "/api")]
pub async fn get_ingredients() -> Result<Vec<Ingredient>, ServerFnError> {
    use crate::api::ingredients;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    ingredients::get_ingredients(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(GetIngredientsByCategory, "/api")]
pub async fn get_ingredients_by_category(category_id: i64) -> Result<Vec<Ingredient>, ServerFnError> {
    use crate::api::ingredients;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    ingredients::get_ingredients_by_category(&pool, category_id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(CreateIngredientFn, "/api")]
pub async fn create_ingredient(
    name: String,
    category_id: i64,
    primary_unit: String,
    secondary_unit: Option<String>,
) -> Result<Ingredient, ServerFnError> {
    use crate::api::ingredients;
    use crate::models::CreateIngredient;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    let new_ingredient = CreateIngredient {
        name,
        category_id,
        primary_unit,
        secondary_unit,
    };
    
    ingredients::create_ingredient(&pool, new_ingredient)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(UpdateIngredientFn, "/api")]
pub async fn update_ingredient(
    id: i64,
    name: String,
    category_id: i64,
    primary_unit: String,
    secondary_unit: Option<String>,
) -> Result<Ingredient, ServerFnError> {
    use crate::api::ingredients;
    use crate::models::UpdateIngredient;
    let pool = expect_context::<sqlx::SqlitePool>();

    let update = UpdateIngredient {
        name: Some(name),
        category_id: Some(category_id),
        primary_unit: Some(primary_unit),
        secondary_unit,
    };

    ingredients::update_ingredient(&pool, id, update)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(DeleteIngredient, "/api")]
pub async fn delete_ingredient(id: i64) -> Result<(), ServerFnError> {
    use crate::api::ingredients;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    ingredients::delete_ingredient(&pool, id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}
