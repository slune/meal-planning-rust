pub mod category;
pub mod ingredient;
pub mod recipe;
pub mod camp;
pub mod meal_plan;
pub mod reports;

pub use category::*;
pub use ingredient::*;
pub use recipe::*;
pub use camp::*;
pub use meal_plan::*;
pub use reports::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MealType {
    Breakfast,
    MorningSnack,
    Lunch,
    AfternoonSnack,
    Dinner,
}

impl MealType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MealType::Breakfast => "breakfast",
            MealType::MorningSnack => "morning_snack",
            MealType::Lunch => "lunch",
            MealType::AfternoonSnack => "afternoon_snack",
            MealType::Dinner => "dinner",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "breakfast" => Some(MealType::Breakfast),
            "morning_snack" => Some(MealType::MorningSnack),
            "lunch" => Some(MealType::Lunch),
            "afternoon_snack" => Some(MealType::AfternoonSnack),
            "dinner" => Some(MealType::Dinner),
            _ => None,
        }
    }

    pub fn sort_order(&self) -> u8 {
        match self {
            MealType::Breakfast => 1,
            MealType::MorningSnack => 2,
            MealType::Lunch => 3,
            MealType::AfternoonSnack => 4,
            MealType::Dinner => 5,
        }
    }
}
