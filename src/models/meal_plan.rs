use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};
use super::MealType;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct MealPlan {
    pub id: i64,
    pub camp_id: i64,
    pub date: NaiveDate,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub created_at: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct PlannedMeal {
    pub id: i64,
    pub meal_plan_id: i64,
    pub recipe_id: i64,
    pub meal_type: String,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct MealAttendance {
    pub id: i64,
    pub planned_meal_id: i64,
    pub children: i32,
    pub teens: i32,
    pub adults: i32,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub created_at: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedMealWithDetails {
    #[serde(flatten)]
    pub planned_meal: PlannedMeal,
    pub recipe_name: String,
    pub attendance: Option<MealAttendance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePlannedMeal {
    pub camp_id: i64,
    pub date: NaiveDate,
    pub meal_type: MealType,
    pub recipe_id: i64,
    pub attendance: Option<CreateAttendance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAttendance {
    pub children: i32,
    pub teens: i32,
    pub adults: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePlannedMeal {
    pub recipe_id: Option<i64>,
    pub attendance: Option<CreateAttendance>,
}
