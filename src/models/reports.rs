use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingListItem {
    pub ingredient_id: i64,
    pub ingredient_name: String,
    pub category_name: String,
    pub total_quantity: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MealScheduleItem {
    pub date: NaiveDate,
    pub meal_type: String,
    pub recipe_name: String,
    pub children: i32,
    pub teens: i32,
    pub adults: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceSummary {
    pub date: NaiveDate,
    pub meal_type: String,
    pub children: i32,
    pub teens: i32,
    pub adults: i32,
    pub total_people: i32,
}
