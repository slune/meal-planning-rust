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

/// One ingredient total for a single day, aggregated across every meal planned
/// that day. Used by the "ingredients day by day" report. The `day_*` fields
/// carry that day's total person-meals (summed attendance over distinct meals)
/// and repeat on every item for the same date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyIngredientItem {
    pub date: NaiveDate,
    pub day_children: i32,
    pub day_teens: i32,
    pub day_adults: i32,
    pub category_name: String,
    pub ingredient_name: String,
    pub total_quantity: f64,
    pub unit: String,
}

/// One ingredient amount for a single recipe served at a single meal on a single
/// day. Used by the "ingredients day by day per recipe" report. The `day_*`
/// fields carry that day's total person-meals and repeat on every item for the
/// same date.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipeIngredientItem {
    pub date: NaiveDate,
    pub day_children: i32,
    pub day_teens: i32,
    pub day_adults: i32,
    pub meal_type: String,
    pub recipe_name: String,
    pub category_name: String,
    pub ingredient_name: String,
    pub quantity: f64,
    pub unit: String,
}

/// One occurrence of a selected ingredient in the camp meal plan. Used by the
/// "selected ingredient by day" report so planners can audit which recipes
/// contribute to an ingredient total.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngredientDayUsageItem {
    pub date: NaiveDate,
    pub meal_type: String,
    pub recipe_name: String,
    pub ingredient_name: String,
    pub children: i32,
    pub teens: i32,
    pub adults: i32,
    pub total_people: i32,
    pub portions: i32,
    pub base_quantity: f64,
    pub quantity: f64,
    pub unit: String,
}
