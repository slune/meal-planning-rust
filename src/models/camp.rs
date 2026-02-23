use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveDate, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Camp {
    pub id: i64,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub default_children: i32,
    pub default_teens: i32,
    pub default_adults: i32,
    pub notes: Option<String>,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub created_at: Option<DateTime<Utc>>,
    #[cfg_attr(feature = "ssr", sqlx(default))]
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCamp {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub default_children: i32,
    pub default_teens: i32,
    pub default_adults: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCamp {
    pub name: Option<String>,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub default_children: Option<i32>,
    pub default_teens: Option<i32>,
    pub default_adults: Option<i32>,
    pub notes: Option<String>,
}
