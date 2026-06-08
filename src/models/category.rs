use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub sort_order: i32,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub created_at: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryIngredientUsage {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CategoryUsage {
    pub category_id: i64,
    pub ingredients: Vec<CategoryIngredientUsage>,
}
