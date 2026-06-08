use chrono::NaiveDate;
use printpdf::*;
use sqlx::SqlitePool;
use std::collections::HashMap;

use crate::api::camps::get_camp;
use crate::api::meal_plans::get_planned_meals_for_date;
use crate::api::recipes::get_recipe_with_ingredients;
use crate::api::reports as report_data;
use crate::models::{
    AttendanceSummary, Camp, DailyIngredientItem, MealScheduleItem, MealType, RecipeIngredientItem,
    ShoppingListItem,
};

#[derive(Debug)]
struct IngredientTotal {
    name: String,
    category_name: String,
    quantities: HashMap<String, f64>,
    sort_order: i32,
}

pub async fn generate_daily_report(
    pool: &SqlitePool,
    camp_id: i64,
    date: NaiveDate,
    language: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let camp = get_camp(pool, camp_id).await?;
    let planned_meals = get_planned_meals_for_date(pool, camp_id, date).await?;

    let mut ingredient_totals: HashMap<i64, IngredientTotal> = HashMap::new();

    for planned_meal in planned_meals {
        let recipe = get_recipe_with_ingredients(pool, planned_meal.planned_meal.recipe_id).await?;

        let (children, teens, adults) = if let Some(ref attendance) = planned_meal.attendance {
            (attendance.children, attendance.teens, attendance.adults)
        } else {
            (
                camp.default_children,
                camp.default_teens,
                camp.default_adults,
            )
        };

        for recipe_ing in recipe.ingredients {
            let child_mult = recipe_ing.recipe_ingredient.child_multiplier.unwrap_or(0.5);
            let teen_mult = recipe_ing.recipe_ingredient.teen_multiplier.unwrap_or(0.75);
            let adult_mult = recipe_ing.recipe_ingredient.adult_multiplier.unwrap_or(1.0);

            let total_multiplier = (children as f64 * child_mult)
                + (teens as f64 * teen_mult)
                + (adults as f64 * adult_mult);

            let quantity = recipe_ing.recipe_ingredient.base_quantity * total_multiplier
                / recipe.recipe.portions as f64;

            let entry = ingredient_totals
                .entry(recipe_ing.recipe_ingredient.ingredient_id)
                .or_insert_with(|| IngredientTotal {
                    name: recipe_ing.ingredient_name.clone(),
                    category_name: String::new(),
                    quantities: HashMap::new(),
                    sort_order: 0,
                });

            *entry
                .quantities
                .entry(recipe_ing.recipe_ingredient.unit.clone())
                .or_insert(0.0) += quantity;
        }
    }

    // Fetch category information
    for (ingredient_id, total) in ingredient_totals.iter_mut() {
        #[derive(sqlx::FromRow)]
        struct IngredientCategory {
            #[allow(dead_code)]
            category_id: i64,
            name: String,
            sort_order: i32,
        }

        let ingredient = sqlx::query_as::<_, IngredientCategory>(
            "SELECT i.category_id, c.name, c.sort_order
             FROM ingredients i
             JOIN categories c ON i.category_id = c.id
             WHERE i.id = ?",
        )
        .bind(ingredient_id)
        .fetch_one(pool)
        .await?;

        total.category_name = ingredient.name;
        total.sort_order = ingredient.sort_order;
    }

    let title = if language == "cz" {
        format!("Denní přehled surovin - {}", date.format("%d.%m.%Y"))
    } else {
        format!("Daily Ingredient Report - {}", date.format("%Y-%m-%d"))
    };
    let subtitle = if language == "cz" {
        format!("Tábor: {}", camp.name)
    } else {
        format!("Camp: {}", camp.name)
    };
    let mut pdf = PdfReport::new(&title, &subtitle)?;

    // Group by category
    let mut sorted_totals: Vec<_> = ingredient_totals.into_iter().collect();
    sorted_totals.sort_by_key(|a| a.1.sort_order);

    let mut current_category = String::new();

    for (_, total) in sorted_totals {
        let category_name = &total.category_name;

        if category_name != &current_category {
            current_category = category_name.clone();
            pdf.section(&current_category);
        }

        let ingredient_name = &total.name;

        for (unit, quantity) in total.quantities {
            let line = format!("  {} {:.2} {}", ingredient_name, quantity, unit);
            pdf.line(&line, 25.0, 10.0, false);
        }
    }

    pdf.finish()
}

pub async fn generate_camp_report(
    pool: &SqlitePool,
    camp_id: i64,
    language: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let camp = get_camp(pool, camp_id).await?;

    // Get all meal plans for the camp
    #[derive(sqlx::FromRow)]
    struct MealPlanDate {
        date: chrono::NaiveDate,
    }

    let meal_plans = sqlx::query_as::<_, MealPlanDate>(
        "SELECT DISTINCT date FROM meal_plans WHERE camp_id = ? ORDER BY date",
    )
    .bind(camp_id)
    .fetch_all(pool)
    .await?;

    let mut ingredient_totals: HashMap<i64, IngredientTotal> = HashMap::new();

    for meal_plan in meal_plans {
        let planned_meals = get_planned_meals_for_date(pool, camp_id, meal_plan.date).await?;

        for planned_meal in planned_meals {
            let recipe =
                get_recipe_with_ingredients(pool, planned_meal.planned_meal.recipe_id).await?;

            let (children, teens, adults) = if let Some(ref attendance) = planned_meal.attendance {
                (attendance.children, attendance.teens, attendance.adults)
            } else {
                (
                    camp.default_children,
                    camp.default_teens,
                    camp.default_adults,
                )
            };

            for recipe_ing in recipe.ingredients {
                let child_mult = recipe_ing.recipe_ingredient.child_multiplier.unwrap_or(0.5);
                let teen_mult = recipe_ing.recipe_ingredient.teen_multiplier.unwrap_or(0.75);
                let adult_mult = recipe_ing.recipe_ingredient.adult_multiplier.unwrap_or(1.0);

                let total_multiplier = (children as f64 * child_mult)
                    + (teens as f64 * teen_mult)
                    + (adults as f64 * adult_mult);

                let quantity = recipe_ing.recipe_ingredient.base_quantity * total_multiplier
                    / recipe.recipe.portions as f64;

                let entry = ingredient_totals
                    .entry(recipe_ing.recipe_ingredient.ingredient_id)
                    .or_insert_with(|| IngredientTotal {
                        name: recipe_ing.ingredient_name.clone(),
                        category_name: String::new(),
                        quantities: HashMap::new(),
                        sort_order: 0,
                    });

                *entry
                    .quantities
                    .entry(recipe_ing.recipe_ingredient.unit.clone())
                    .or_insert(0.0) += quantity;
            }
        }
    }

    // Fetch category information
    for (ingredient_id, total) in ingredient_totals.iter_mut() {
        #[derive(sqlx::FromRow)]
        struct IngredientCategory {
            #[allow(dead_code)]
            category_id: i64,
            name: String,
            sort_order: i32,
        }

        let ingredient = sqlx::query_as::<_, IngredientCategory>(
            "SELECT i.category_id, c.name, c.sort_order
             FROM ingredients i
             JOIN categories c ON i.category_id = c.id
             WHERE i.id = ?",
        )
        .bind(ingredient_id)
        .fetch_one(pool)
        .await?;

        total.category_name = ingredient.name;
        total.sort_order = ingredient.sort_order;
    }

    let title = if language == "cz" {
        "Nákupní seznam pro celý tábor"
    } else {
        "Shopping List for Entire Camp"
    };
    let subtitle = if language == "cz" {
        format!(
            "Tábor: {} | Od {} do {}",
            camp.name,
            camp.start_date.format("%d.%m.%Y"),
            camp.end_date.format("%d.%m.%Y")
        )
    } else {
        format!(
            "Camp: {} | From {} to {}",
            camp.name,
            camp.start_date.format("%Y-%m-%d"),
            camp.end_date.format("%Y-%m-%d")
        )
    };
    let mut pdf = PdfReport::new(title, &subtitle)?;

    // Group by category
    let mut sorted_totals: Vec<_> = ingredient_totals.into_iter().collect();
    sorted_totals.sort_by_key(|a| a.1.sort_order);

    let mut current_category = String::new();

    for (_, total) in sorted_totals {
        let category_name = &total.category_name;

        if category_name != &current_category {
            current_category = category_name.clone();
            pdf.section(&current_category);
        }

        let ingredient_name = &total.name;

        for (unit, quantity) in total.quantities {
            let line = format!("  {} {:.2} {}", ingredient_name, quantity, unit);
            pdf.line(&line, 25.0, 10.0, false);
        }
    }

    pdf.finish()
}

#[derive(Clone, Copy, Debug)]
pub enum ReportPdfKind {
    ShoppingList,
    MealSchedule,
    AttendanceSummary,
    DailyIngredients,
    IngredientsByRecipe,
}

impl TryFrom<&str> for ReportPdfKind {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "shopping_list" => Ok(Self::ShoppingList),
            "meal_schedule" => Ok(Self::MealSchedule),
            "attendance_summary" => Ok(Self::AttendanceSummary),
            "daily_ingredients" => Ok(Self::DailyIngredients),
            "ingredients_by_recipe" => Ok(Self::IngredientsByRecipe),
            _ => Err("Invalid report type"),
        }
    }
}

impl ReportPdfKind {
    fn title(self) -> &'static str {
        match self {
            Self::ShoppingList => "Shopping List",
            Self::MealSchedule => "Meal Schedule",
            Self::AttendanceSummary => "Attendance Summary",
            Self::DailyIngredients => "Ingredients Day by Day",
            Self::IngredientsByRecipe => "Ingredients Day by Day (per Recipe)",
        }
    }
}

pub async fn generate_report_pdf(
    pool: &SqlitePool,
    camp_id: i64,
    kind: ReportPdfKind,
    start_date: Option<NaiveDate>,
    end_date: Option<NaiveDate>,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let camp = get_camp(pool, camp_id).await?;
    let date_range = match (start_date, end_date) {
        (Some(start), Some(end)) => Some((start, end)),
        _ => None,
    };

    let subtitle = report_subtitle(&camp, date_range);
    match kind {
        ReportPdfKind::ShoppingList => {
            let (start, end) = date_range.ok_or("Shopping list requires a date range")?;
            let items = report_data::generate_shopping_list(pool, camp_id, start, end).await?;
            let mut pdf = PdfReport::new(kind.title(), &subtitle)?;
            render_shopping_list_pdf(&mut pdf, items);
            pdf.finish()
        }
        ReportPdfKind::MealSchedule => {
            let items = report_data::generate_meal_schedule(pool, camp_id).await?;
            let mut pdf = PdfReport::new(kind.title(), &subtitle)?;
            render_meal_schedule_pdf(&mut pdf, items);
            pdf.finish()
        }
        ReportPdfKind::AttendanceSummary => {
            let items = report_data::generate_attendance_summary(pool, camp_id).await?;
            let mut pdf = PdfReport::new(kind.title(), &subtitle)?;
            render_attendance_summary_pdf(&mut pdf, items);
            pdf.finish()
        }
        ReportPdfKind::DailyIngredients => {
            let items = report_data::generate_daily_ingredients(pool, camp_id).await?;
            let mut pdf = PdfReport::new(kind.title(), &subtitle)?;
            render_daily_ingredients_pdf(&mut pdf, items);
            pdf.finish()
        }
        ReportPdfKind::IngredientsByRecipe => {
            let items = report_data::generate_ingredients_by_recipe(pool, camp_id).await?;
            let mut pdf = PdfReport::new(kind.title(), &subtitle)?;
            render_ingredients_by_recipe_pdf(&mut pdf, items);
            pdf.finish()
        }
    }
}

fn report_subtitle(camp: &Camp, date_range: Option<(NaiveDate, NaiveDate)>) -> String {
    match date_range {
        Some((start, end)) => format!("{} | {} to {}", camp.name, start, end),
        None => format!("{} | {} to {}", camp.name, camp.start_date, camp.end_date),
    }
}

#[derive(Clone, Copy)]
struct PdfColumn {
    title: &'static str,
    x: f32,
    width_chars: usize,
}

struct PdfReport {
    doc: PdfDocument,
    ops: Vec<Op>,
    title: String,
    subtitle: String,
    page_number: usize,
    y: f32,
}

impl PdfReport {
    const PAGE_W: f32 = 210.0;
    const PAGE_H: f32 = 297.0;
    const MARGIN_X: f32 = 14.0;
    const BOTTOM_Y: f32 = 16.0;
    const HEADER_TOP_Y: f32 = 282.0;
    const CONTENT_TOP_Y: f32 = 258.0;
    const ROW_H: f32 = 6.2;

    fn new(title: &str, subtitle: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mut pdf = Self {
            doc: PdfDocument::new(title),
            ops: Vec::new(),
            title: title.to_string(),
            subtitle: subtitle.to_string(),
            page_number: 1,
            y: Self::CONTENT_TOP_Y,
        };
        pdf.draw_page_header();
        Ok(pdf)
    }

    fn finish(mut self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.push_page();
        let mut warnings = Vec::new();
        Ok(self.doc.save(&PdfSaveOptions::default(), &mut warnings))
    }

    fn draw_page_header(&mut self) {
        let title = self.title.clone();
        let subtitle = self.subtitle.clone();
        let page = format!("Page {}", self.page_number);

        self.y = Self::HEADER_TOP_Y;
        self.text(&title, Self::MARGIN_X, self.y, 16.0, true);
        self.text(&page, 178.0, self.y, 9.0, false);
        self.y -= 8.0;
        self.text(&subtitle, Self::MARGIN_X, self.y, 9.5, false);
        self.y = Self::CONTENT_TOP_Y;
    }

    fn new_page(&mut self) {
        self.push_page();
        self.page_number += 1;
        self.draw_page_header();
    }

    fn push_page(&mut self) {
        let ops = std::mem::take(&mut self.ops);
        self.doc
            .pages
            .push(PdfPage::new(Mm(Self::PAGE_W), Mm(Self::PAGE_H), ops));
    }

    fn ensure_space(&mut self, height: f32) -> bool {
        if self.y - height < Self::BOTTOM_Y {
            self.new_page();
            true
        } else {
            false
        }
    }

    fn section(&mut self, text: impl AsRef<str>) {
        self.ensure_space(14.0);
        self.y -= 2.0;
        self.text(text.as_ref(), Self::MARGIN_X, self.y, 12.5, true);
        self.y -= 7.0;
    }

    fn subsection(&mut self, text: impl AsRef<str>) {
        self.ensure_space(10.0);
        self.text(text.as_ref(), Self::MARGIN_X + 2.0, self.y, 10.5, true);
        self.y -= 6.5;
    }

    fn line(&mut self, text: impl AsRef<str>, x: f32, size: f32, bold: bool) {
        self.ensure_space(Self::ROW_H);
        self.text(text.as_ref(), x, self.y, size, bold);
        self.y -= Self::ROW_H;
    }

    fn table_header(&mut self, columns: &[PdfColumn]) {
        self.ensure_space(Self::ROW_H * 2.0);
        for column in columns {
            self.text(column.title, column.x, self.y, 8.5, true);
        }
        self.y -= Self::ROW_H;
    }

    fn table_row(&mut self, columns: &[PdfColumn], values: &[String]) {
        if self.ensure_space(Self::ROW_H) {
            self.table_header(columns);
        }

        for (column, value) in columns.iter().zip(values) {
            self.text(
                &fit_text(value, column.width_chars),
                column.x,
                self.y,
                8.3,
                false,
            );
        }
        self.y -= Self::ROW_H;
    }

    fn text(&mut self, text: &str, x: f32, y: f32, size: f32, bold: bool) {
        let font = if bold {
            BuiltinFont::HelveticaBold
        } else {
            BuiltinFont::Helvetica
        };
        self.ops.extend([
            Op::StartTextSection,
            Op::SetTextCursor {
                pos: Point::new(Mm(x), Mm(y)),
            },
            Op::SetFont {
                font: PdfFontHandle::Builtin(font),
                size: Pt(size),
            },
            Op::SetLineHeight { lh: Pt(size) },
            Op::ShowText {
                items: vec![TextItem::Text(text.to_string())],
            },
            Op::EndTextSection,
        ]);
    }
}

fn fit_text(text: &str, max_chars: usize) -> String {
    let len = text.chars().count();
    if len <= max_chars {
        return text.to_string();
    }

    let keep = max_chars.saturating_sub(3);
    let mut shortened: String = text.chars().take(keep).collect();
    shortened.push_str("...");
    shortened
}

fn meal_type_label(meal_type: &str) -> String {
    meal_type
        .parse::<MealType>()
        .map(|meal_type| match meal_type {
            MealType::Breakfast => "Breakfast",
            MealType::MorningSnack => "Morning Snack",
            MealType::Lunch => "Lunch",
            MealType::AfternoonSnack => "Afternoon Snack",
            MealType::Dinner => "Dinner",
        })
        .unwrap_or(meal_type)
        .to_string()
}

fn render_shopping_list_pdf(pdf: &mut PdfReport, items: Vec<ShoppingListItem>) {
    let columns = [
        PdfColumn {
            title: "Ingredient",
            x: 18.0,
            width_chars: 52,
        },
        PdfColumn {
            title: "Quantity",
            x: 150.0,
            width_chars: 10,
        },
        PdfColumn {
            title: "Unit",
            x: 176.0,
            width_chars: 12,
        },
    ];

    let mut current_category = String::new();
    for item in items {
        if item.category_name != current_category {
            current_category = item.category_name.clone();
            pdf.section(&current_category);
            pdf.table_header(&columns);
        }

        pdf.table_row(
            &columns,
            &[
                item.ingredient_name,
                format!("{:.2}", item.total_quantity),
                item.unit,
            ],
        );
    }
}

fn render_meal_schedule_pdf(pdf: &mut PdfReport, items: Vec<MealScheduleItem>) {
    let columns = [
        PdfColumn {
            title: "Date",
            x: 14.0,
            width_chars: 11,
        },
        PdfColumn {
            title: "Meal",
            x: 41.0,
            width_chars: 17,
        },
        PdfColumn {
            title: "Recipe",
            x: 82.0,
            width_chars: 39,
        },
        PdfColumn {
            title: "Child",
            x: 161.0,
            width_chars: 6,
        },
        PdfColumn {
            title: "Teen",
            x: 174.0,
            width_chars: 6,
        },
        PdfColumn {
            title: "Adult",
            x: 187.0,
            width_chars: 6,
        },
    ];

    pdf.table_header(&columns);
    for item in items {
        pdf.table_row(
            &columns,
            &[
                item.date.to_string(),
                meal_type_label(&item.meal_type),
                item.recipe_name,
                item.children.to_string(),
                item.teens.to_string(),
                item.adults.to_string(),
            ],
        );
    }
}

fn render_attendance_summary_pdf(pdf: &mut PdfReport, items: Vec<AttendanceSummary>) {
    let columns = [
        PdfColumn {
            title: "Date",
            x: 14.0,
            width_chars: 11,
        },
        PdfColumn {
            title: "Meal",
            x: 45.0,
            width_chars: 17,
        },
        PdfColumn {
            title: "Children",
            x: 100.0,
            width_chars: 8,
        },
        PdfColumn {
            title: "Teens",
            x: 126.0,
            width_chars: 8,
        },
        PdfColumn {
            title: "Adults",
            x: 152.0,
            width_chars: 8,
        },
        PdfColumn {
            title: "Total",
            x: 178.0,
            width_chars: 8,
        },
    ];

    pdf.table_header(&columns);
    for item in items {
        pdf.table_row(
            &columns,
            &[
                item.date.to_string(),
                meal_type_label(&item.meal_type),
                item.children.to_string(),
                item.teens.to_string(),
                item.adults.to_string(),
                item.total_people.to_string(),
            ],
        );
    }
}

fn render_daily_ingredients_pdf(pdf: &mut PdfReport, items: Vec<DailyIngredientItem>) {
    let columns = [
        PdfColumn {
            title: "Ingredient",
            x: 18.0,
            width_chars: 52,
        },
        PdfColumn {
            title: "Quantity",
            x: 150.0,
            width_chars: 10,
        },
        PdfColumn {
            title: "Unit",
            x: 176.0,
            width_chars: 12,
        },
    ];

    let mut current_date = None;
    let mut current_category = String::new();
    for item in items {
        if current_date != Some(item.date) {
            current_date = Some(item.date);
            current_category.clear();
            pdf.section(format!(
                "{} | person-meals: children {}, teens {}, adults {}",
                item.date, item.day_children, item.day_teens, item.day_adults
            ));
        }

        if item.category_name != current_category {
            current_category = item.category_name.clone();
            pdf.subsection(&current_category);
            pdf.table_header(&columns);
        }

        pdf.table_row(
            &columns,
            &[
                item.ingredient_name,
                format!("{:.2}", item.total_quantity),
                item.unit,
            ],
        );
    }
}

fn render_ingredients_by_recipe_pdf(pdf: &mut PdfReport, items: Vec<RecipeIngredientItem>) {
    let columns = [
        PdfColumn {
            title: "Category",
            x: 18.0,
            width_chars: 22,
        },
        PdfColumn {
            title: "Ingredient",
            x: 66.0,
            width_chars: 38,
        },
        PdfColumn {
            title: "Quantity",
            x: 151.0,
            width_chars: 10,
        },
        PdfColumn {
            title: "Unit",
            x: 176.0,
            width_chars: 12,
        },
    ];

    let mut current_date = None;
    let mut current_meal = String::new();
    let mut current_recipe = String::new();

    for item in items {
        if current_date != Some(item.date) {
            current_date = Some(item.date);
            current_meal.clear();
            current_recipe.clear();
            pdf.section(format!(
                "{} | person-meals: children {}, teens {}, adults {}",
                item.date, item.day_children, item.day_teens, item.day_adults
            ));
        }

        if item.meal_type != current_meal {
            current_meal = item.meal_type.clone();
            current_recipe.clear();
            pdf.subsection(meal_type_label(&item.meal_type));
        }

        if item.recipe_name != current_recipe {
            current_recipe = item.recipe_name.clone();
            pdf.subsection(format!("Recipe: {}", current_recipe));
            pdf.table_header(&columns);
        }

        pdf.table_row(
            &columns,
            &[
                item.category_name,
                item.ingredient_name,
                format!("{:.2}", item.quantity),
                item.unit,
            ],
        );
    }
}
