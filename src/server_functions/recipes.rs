use leptos::prelude::*;
use crate::models::{Recipe, RecipeWithIngredients, CreateRecipeIngredient};

#[server(GetRecipes, "/api")]
pub async fn get_recipes() -> Result<Vec<Recipe>, ServerFnError> {
    use crate::api::recipes;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    recipes::get_recipes(&pool)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(GetRecipeWithIngredients, "/api")]
pub async fn get_recipe_with_ingredients(id: i64) -> Result<RecipeWithIngredients, ServerFnError> {
    use crate::api::recipes;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    recipes::get_recipe_with_ingredients(&pool, id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(CreateRecipeFn, "/api")]
pub async fn create_recipe(
    name: String,
    instructions: Option<String>,
    base_servings: i32,
    ingredients: Vec<CreateRecipeIngredient>,
) -> Result<RecipeWithIngredients, ServerFnError> {
    use crate::api::recipes;
    use crate::models::CreateRecipe;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    let new_recipe = CreateRecipe {
        name,
        instructions,
        base_servings,
        ingredients,
    };
    
    recipes::create_recipe(&pool, new_recipe)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(UpdateRecipeFn, "/api")]
pub async fn update_recipe(
    id: i64,
    name: String,
    instructions: Option<String>,
    base_servings: i32,
    ingredients: Vec<CreateRecipeIngredient>,
) -> Result<RecipeWithIngredients, ServerFnError> {
    use crate::api::recipes;
    use crate::models::UpdateRecipe;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    let update_recipe = UpdateRecipe {
        name: Some(name),
        instructions,
        base_servings: Some(base_servings),
        ingredients: Some(ingredients),
    };
    
    recipes::update_recipe(&pool, id, update_recipe)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}

#[server(DeleteRecipe, "/api")]
pub async fn delete_recipe(id: i64) -> Result<(), ServerFnError> {
    use crate::api::recipes;
    let pool = expect_context::<sqlx::SqlitePool>();
    
    recipes::delete_recipe(&pool, id)
        .await
        .map_err(|e| ServerFnError::ServerError(e.to_string()))
}
