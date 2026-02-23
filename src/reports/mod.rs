use chrono::NaiveDate;
use printpdf::*;
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::io::BufWriter;

use crate::api::meal_plans::get_planned_meals_for_date;
use crate::api::recipes::get_recipe_with_ingredients;
use crate::api::camps::get_camp;

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
            (camp.default_children, camp.default_teens, camp.default_adults)
        };

        for recipe_ing in recipe.ingredients {
            let child_mult = recipe_ing.recipe_ingredient.child_multiplier.unwrap_or(0.5);
            let teen_mult = recipe_ing.recipe_ingredient.teen_multiplier.unwrap_or(0.75);
            let adult_mult = recipe_ing.recipe_ingredient.adult_multiplier.unwrap_or(1.0);

            let total_multiplier = 
                (children as f64 * child_mult) +
                (teens as f64 * teen_mult) +
                (adults as f64 * adult_mult);

            let quantity = recipe_ing.recipe_ingredient.base_quantity * total_multiplier / recipe.recipe.base_servings as f64;

            let entry = ingredient_totals.entry(recipe_ing.recipe_ingredient.ingredient_id)
                .or_insert_with(|| IngredientTotal {
                    name: recipe_ing.ingredient_name.clone(),
                    category_name: String::new(),
                    quantities: HashMap::new(),
                    sort_order: 0,
                });

            *entry.quantities.entry(recipe_ing.recipe_ingredient.unit.clone()).or_insert(0.0) += quantity;
        }
    }

    // Fetch category information
    for (ingredient_id, total) in ingredient_totals.iter_mut() {
        #[derive(sqlx::FromRow)]
        struct IngredientCategory {
            category_id: i64,
            name: String,
            sort_order: i32,
        }
        
        let ingredient = sqlx::query_as::<_, IngredientCategory>(
            "SELECT i.category_id, c.name, c.sort_order
             FROM ingredients i
             JOIN categories c ON i.category_id = c.id
             WHERE i.id = ?"
        )
        .bind(ingredient_id)
        .fetch_one(pool)
        .await?;

        total.category_name = ingredient.name;
        total.sort_order = ingredient.sort_order;
    }

    // Generate PDF
    let (doc, page1, layer1) = PdfDocument::new(
        if language == "cz" { "Denní přehled surovin" } else { "Daily Ingredient Report" },
        Mm(210.0),
        Mm(297.0),
        "Layer 1"
    );

    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let mut y_pos = 280.0;

    // Title
    current_layer.use_text(
        if language == "cz" {
            format!("Denní přehled surovin - {}", date.format("%d.%m.%Y"))
        } else {
            format!("Daily Ingredient Report - {}", date.format("%Y-%m-%d"))
        },
        16.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold
    );

    y_pos -= 10.0;

    current_layer.use_text(
        if language == "cz" {
            format!("Tábor: {}", camp.name)
        } else {
            format!("Camp: {}", camp.name)
        },
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font
    );

    y_pos -= 15.0;

    // Group by category
    let mut sorted_totals: Vec<_> = ingredient_totals.into_iter().collect();
    sorted_totals.sort_by(|a, b| a.1.sort_order.cmp(&b.1.sort_order));

    let mut current_category = String::new();

    for (_, total) in sorted_totals {
        let category_name = &total.category_name;
        
        if category_name != &current_category {
            current_category = category_name.clone();
            y_pos -= 5.0;
            
            if y_pos < 20.0 {
                // Start new page
                y_pos = 280.0;
            }

            current_layer.use_text(
                &current_category,
                14.0,
                Mm(20.0),
                Mm(y_pos),
                &font_bold
            );
            y_pos -= 8.0;
        }

        let ingredient_name = &total.name;
        
        for (unit, quantity) in total.quantities {
            let line = format!("  {} {:.2} {}", ingredient_name, quantity, unit);
            current_layer.use_text(
                &line,
                10.0,
                Mm(25.0),
                Mm(y_pos),
                &font
            );
            y_pos -= 6.0;

            if y_pos < 20.0 {
                y_pos = 280.0;
            }
        }
    }

    let mut buffer = BufWriter::new(Vec::new());
    doc.save(&mut buffer)?;
    let bytes = buffer.into_inner()?;

    Ok(bytes)
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
        "SELECT DISTINCT date FROM meal_plans WHERE camp_id = ? ORDER BY date"
    )
    .bind(camp_id)
    .fetch_all(pool)
    .await?;

    let mut ingredient_totals: HashMap<i64, IngredientTotal> = HashMap::new();

    for meal_plan in meal_plans {
        let planned_meals = get_planned_meals_for_date(pool, camp_id, meal_plan.date).await?;

        for planned_meal in planned_meals {
            let recipe = get_recipe_with_ingredients(pool, planned_meal.planned_meal.recipe_id).await?;
            
            let (children, teens, adults) = if let Some(ref attendance) = planned_meal.attendance {
                (attendance.children, attendance.teens, attendance.adults)
            } else {
                (camp.default_children, camp.default_teens, camp.default_adults)
            };

            for recipe_ing in recipe.ingredients {
                let child_mult = recipe_ing.recipe_ingredient.child_multiplier.unwrap_or(0.5);
                let teen_mult = recipe_ing.recipe_ingredient.teen_multiplier.unwrap_or(0.75);
                let adult_mult = recipe_ing.recipe_ingredient.adult_multiplier.unwrap_or(1.0);

                let total_multiplier = 
                    (children as f64 * child_mult) +
                    (teens as f64 * teen_mult) +
                    (adults as f64 * adult_mult);

                let quantity = recipe_ing.recipe_ingredient.base_quantity * total_multiplier / recipe.recipe.base_servings as f64;

                let entry = ingredient_totals.entry(recipe_ing.recipe_ingredient.ingredient_id)
                    .or_insert_with(|| IngredientTotal {
                        name: recipe_ing.ingredient_name.clone(),
                        category_name: String::new(),
                        quantities: HashMap::new(),
                        sort_order: 0,
                    });

                *entry.quantities.entry(recipe_ing.recipe_ingredient.unit.clone()).or_insert(0.0) += quantity;
            }
        }
    }

    // Fetch category information
    for (ingredient_id, total) in ingredient_totals.iter_mut() {
        #[derive(sqlx::FromRow)]
        struct IngredientCategory {
            category_id: i64,
            name: String,
            sort_order: i32,
        }
        
        let ingredient = sqlx::query_as::<_, IngredientCategory>(
            "SELECT i.category_id, c.name, c.sort_order
             FROM ingredients i
             JOIN categories c ON i.category_id = c.id
             WHERE i.id = ?"
        )
        .bind(ingredient_id)
        .fetch_one(pool)
        .await?;

        total.category_name = ingredient.name;
        total.sort_order = ingredient.sort_order;
    }

    // Generate PDF
    let (doc, page1, layer1) = PdfDocument::new(
        if language == "cz" { "Nákupní seznam" } else { "Shopping List" },
        Mm(210.0),
        Mm(297.0),
        "Layer 1"
    );

    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;
    let current_layer = doc.get_page(page1).get_layer(layer1);

    let mut y_pos = 280.0;

    // Title
    current_layer.use_text(
        if language == "cz" {
            "Nákupní seznam pro celý tábor"
        } else {
            "Shopping List for Entire Camp"
        },
        16.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold
    );

    y_pos -= 10.0;

    current_layer.use_text(
        if language == "cz" {
            format!("Tábor: {}", camp.name)
        } else {
            format!("Camp: {}", camp.name)
        },
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font
    );

    current_layer.use_text(
        if language == "cz" {
            format!("Od {} do {}", camp.start_date.format("%d.%m.%Y"), camp.end_date.format("%d.%m.%Y"))
        } else {
            format!("From {} to {}", camp.start_date.format("%Y-%m-%d"), camp.end_date.format("%Y-%m-%d"))
        },
        12.0,
        Mm(20.0),
        Mm(y_pos - 6.0),
        &font
    );

    y_pos -= 20.0;

    // Group by category
    let mut sorted_totals: Vec<_> = ingredient_totals.into_iter().collect();
    sorted_totals.sort_by(|a, b| a.1.sort_order.cmp(&b.1.sort_order));

    let mut current_category = String::new();

    for (_, total) in sorted_totals {
        let category_name = &total.category_name;
        
        if category_name != &current_category {
            current_category = category_name.clone();
            y_pos -= 5.0;
            
            if y_pos < 20.0 {
                y_pos = 280.0;
            }

            current_layer.use_text(
                &current_category,
                14.0,
                Mm(20.0),
                Mm(y_pos),
                &font_bold
            );
            y_pos -= 8.0;
        }

        let ingredient_name = &total.name;
        
        for (unit, quantity) in total.quantities {
            let line = format!("  {} {:.2} {}", ingredient_name, quantity, unit);
            current_layer.use_text(
                &line,
                10.0,
                Mm(25.0),
                Mm(y_pos),
                &font
            );
            y_pos -= 6.0;

            if y_pos < 20.0 {
                y_pos = 280.0;
            }
        }
    }

    let mut buffer = BufWriter::new(Vec::new());
    doc.save(&mut buffer)?;
    let bytes = buffer.into_inner()?;

    Ok(bytes)
}
