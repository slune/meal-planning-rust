use std::collections::HashMap;
use std::fs;

use serde::Deserialize;
use sqlx::Row;

use ai_meal_planning::db::init_db;

#[derive(Deserialize)]
struct YamlIngredient {
    name: String,
    #[serde(rename = "type")]
    kind: String,
    unit: String,
}

#[derive(Deserialize, Default)]
struct YamlGroups {
    #[serde(rename = "_general", default)]
    general: Vec<String>,
    #[serde(default)]
    decko: Vec<String>,
    #[serde(default)]
    pubos: Vec<String>,
    #[serde(default)]
    dospelak: Vec<String>,
}

#[derive(Deserialize)]
struct YamlRecipe {
    #[serde(default)]
    ingredients: YamlGroups,
    porci: String,
}

struct IngredientEntry {
    general_qty: f64,
    decko_qty: f64,
    pubos_qty: f64,
    dospelak_qty: f64,
    unit: String,
}

fn parse_ingredient_line(line: &str) -> Option<(f64, String, String)> {
    let (qty_unit, key) = line.split_once("  ")?;
    let (qty_str, unit) = qty_unit.split_once(' ')?;
    let qty = qty_str.trim().parse::<f64>().ok()?;
    Some((qty, unit.trim().to_string(), key.trim().to_string()))
}

fn map_category(kind: &str, categories: &HashMap<String, i64>) -> i64 {
    let other_id = *categories.get("Other").unwrap_or(&99);
    match kind {
        "ovoce zelenina" => *categories.get("Vegetables").unwrap_or(&other_id),
        "pekarna" => *categories.get("Baking Goods").unwrap_or(&other_id),
        "maso" => *categories.get("Meat & Fish").unwrap_or(&other_id),
        _ => other_id,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = init_db().await?;

    // Fetch categories
    let cat_rows = sqlx::query("SELECT id, name FROM categories")
        .fetch_all(&pool)
        .await?;
    let categories: HashMap<String, i64> = cat_rows
        .iter()
        .map(|r| (r.get::<String, _>("name"), r.get::<i64, _>("id")))
        .collect();
    let other_id = *categories.get("Other").unwrap_or(&99);

    // ----- Import ingredients -----
    let ingredients_content = fs::read_to_string("source_data/ingredients.yaml")?;
    let ingredients_yaml: HashMap<String, YamlIngredient> =
        serde_yaml::from_str(&ingredients_content)?;

    let mut ingredient_lookup: HashMap<String, i64> = HashMap::new();
    let mut ingredients_imported = 0usize;

    let mut ingredient_keys: Vec<&String> = ingredients_yaml.keys().collect();
    ingredient_keys.sort();

    for key in &ingredient_keys {
        let ing = &ingredients_yaml[*key];
        let category_id = map_category(&ing.kind, &categories);

        let existing = sqlx::query("SELECT id FROM ingredients WHERE name = ?")
            .bind(&ing.name)
            .fetch_optional(&pool)
            .await?;

        let id: i64 = if let Some(row) = existing {
            row.get::<i64, _>("id")
        } else {
            let result = sqlx::query(
                "INSERT INTO ingredients (name, category_id, primary_unit) VALUES (?, ?, ?)",
            )
            .bind(&ing.name)
            .bind(category_id)
            .bind(&ing.unit)
            .execute(&pool)
            .await?;
            ingredients_imported += 1;
            result.last_insert_rowid()
        };

        ingredient_lookup.insert((*key).clone(), id);
        ingredient_lookup.entry(ing.name.to_lowercase()).or_insert(id);
    }

    // ----- Import recipes -----
    let recipes_content = fs::read_to_string("source_data/recipes.yaml")?;
    let recipes_yaml: HashMap<String, YamlRecipe> = serde_yaml::from_str(&recipes_content)?;

    let mut recipe_keys: Vec<&String> = recipes_yaml.keys().collect();
    recipe_keys.sort();

    let mut recipes_imported = 0usize;
    let mut auto_created = 0usize;

    for recipe_name in &recipe_keys {
        let recipe = &recipes_yaml[*recipe_name];

        let existing = sqlx::query("SELECT id FROM recipes WHERE name = ?")
            .bind(*recipe_name)
            .fetch_optional(&pool)
            .await?;

        if existing.is_some() {
            continue;
        }

        let result = sqlx::query("INSERT INTO recipes (name, base_servings) VALUES (?, 1)")
            .bind(*recipe_name)
            .execute(&pool)
            .await?;
        let recipe_id = result.last_insert_rowid();
        recipes_imported += 1;

        let porci = recipe.porci.parse::<f64>().unwrap_or(1.0).max(1.0);

        // Build per-ingredient entry map across all groups
        let mut entries: HashMap<String, IngredientEntry> = HashMap::new();

        let group_data: [(&[String], &str); 4] = [
            (&recipe.ingredients.general, "general"),
            (&recipe.ingredients.decko, "decko"),
            (&recipe.ingredients.pubos, "pubos"),
            (&recipe.ingredients.dospelak, "dospelak"),
        ];

        for (lines, group) in &group_data {
            for line in *lines {
                let Some((qty, unit, key)) = parse_ingredient_line(line) else {
                    continue;
                };
                let e = entries.entry(key).or_insert(IngredientEntry {
                    general_qty: 0.0,
                    decko_qty: 0.0,
                    pubos_qty: 0.0,
                    dospelak_qty: 0.0,
                    unit,
                });
                match *group {
                    "general" => e.general_qty += qty,
                    "decko" => e.decko_qty += qty,
                    "pubos" => e.pubos_qty += qty,
                    "dospelak" => e.dospelak_qty += qty,
                    _ => {}
                }
            }
        }

        let mut entry_keys: Vec<String> = entries.keys().cloned().collect();
        entry_keys.sort();

        for ing_key in &entry_keys {
            let entry = &entries[ing_key];

            // Resolve ingredient id; auto-create if not found
            let ing_id: i64 = if let Some(&id) = ingredient_lookup.get(ing_key) {
                id
            } else if let Some(&id) = ingredient_lookup.get(&ing_key.to_lowercase()) {
                id
            } else {
                let result = sqlx::query(
                    "INSERT INTO ingredients (name, category_id, primary_unit) VALUES (?, ?, ?)",
                )
                .bind(ing_key)
                .bind(other_id)
                .bind("g")
                .execute(&pool)
                .await?;
                let new_id = result.last_insert_rowid();
                ingredient_lookup.insert(ing_key.clone(), new_id);
                auto_created += 1;
                new_id
            };

            // Compute per-person quantities (totals / porci)
            let adult_per = entry.dospelak_qty / porci + entry.general_qty / porci;
            let child_per = entry.decko_qty / porci + entry.general_qty / porci;
            let teen_per = entry.pubos_qty / porci + entry.general_qty / porci;

            let (base_quantity, child_mult, teen_mult, adult_mult) = if adult_per > 0.0 {
                (adult_per, child_per / adult_per, teen_per / adult_per, 1.0f64)
            } else if child_per > 0.0 {
                let teen_m = teen_per / child_per;
                (child_per, 1.0f64, teen_m, 0.0f64)
            } else if teen_per > 0.0 {
                (teen_per, 0.0f64, 1.0f64, 0.0f64)
            } else {
                continue; // zero quantity everywhere — skip
            };

            sqlx::query(
                "INSERT INTO recipe_ingredients \
                 (recipe_id, ingredient_id, base_quantity, unit, child_multiplier, teen_multiplier, adult_multiplier) \
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(recipe_id)
            .bind(ing_id)
            .bind(base_quantity)
            .bind(&entry.unit)
            .bind(child_mult)
            .bind(teen_mult)
            .bind(adult_mult)
            .execute(&pool)
            .await?;
        }
    }

    println!(
        "{} ingredients imported, {} recipes imported, {} ingredients auto-created",
        ingredients_imported, recipes_imported, auto_created
    );

    Ok(())
}
