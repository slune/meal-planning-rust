use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Recipe {
    pub id: i64,
    pub name: String,
    pub instructions: Option<String>,
    pub base_servings: i32,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub created_at: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct RecipeIngredient {
    pub id: i64,
    pub recipe_id: i64,
    pub ingredient_id: i64,
    pub base_quantity: f64,
    pub unit: String,
    pub child_multiplier: Option<f64>,
    pub teen_multiplier: Option<f64>,
    pub adult_multiplier: Option<f64>,
    pub notes: Option<String>,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeWithIngredients {
    #[serde(flatten)]
    pub recipe: Recipe,
    pub ingredients: Vec<RecipeIngredientDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct RecipeIngredientDetail {
    #[serde(flatten)]
    #[cfg_attr(feature = "ssr", sqlx(flatten))]
    pub recipe_ingredient: RecipeIngredient,
    pub ingredient_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecipe {
    pub name: String,
    pub instructions: Option<String>,
    pub base_servings: i32,
    pub ingredients: Vec<CreateRecipeIngredient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRecipeIngredient {
    pub ingredient_id: i64,
    pub base_quantity: f64,
    pub unit: String,
    pub child_multiplier: Option<f64>,
    pub teen_multiplier: Option<f64>,
    pub adult_multiplier: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecipe {
    pub name: Option<String>,
    pub instructions: Option<String>,
    pub base_servings: Option<i32>,
    pub ingredients: Option<Vec<CreateRecipeIngredient>>,
}
