pub mod camp;
pub mod category;
pub mod ingredient;
pub mod meal_plan;
pub mod recipe;
pub mod reports;

pub use camp::*;
pub use category::*;
pub use ingredient::*;
pub use meal_plan::*;
pub use recipe::*;
pub use reports::*;

use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

impl FromStr for MealType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "breakfast" => Ok(MealType::Breakfast),
            "morning_snack" => Ok(MealType::MorningSnack),
            "lunch" => Ok(MealType::Lunch),
            "afternoon_snack" => Ok(MealType::AfternoonSnack),
            "dinner" => Ok(MealType::Dinner),
            _ => Err(()),
        }
    }
}
