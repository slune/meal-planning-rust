use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Ingredient {
    pub id: i64,
    pub name: String,
    pub category_id: i64,
    pub primary_unit: String,
    pub secondary_unit: Option<String>,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub created_at: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIngredient {
    pub name: String,
    pub category_id: i64,
    pub primary_unit: String,
    pub secondary_unit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIngredient {
    pub name: Option<String>,
    pub category_id: Option<i64>,
    pub primary_unit: Option<String>,
    pub secondary_unit: Option<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientRecipeUsage {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IngredientUsage {
    pub ingredient_id: i64,
    pub recipes: Vec<IngredientRecipeUsage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeIngredientsResult {
    pub ingredient: Ingredient,
    pub merged_ingredient_count: usize,
    pub updated_recipe_rows: u64,
    pub combined_recipe_rows: u64,
    pub deleted_ingredient_count: u64,
}
