use crate::models::{Recipe, RecipeIngredientDetail, RecipeWithIngredients, CreateRecipe, UpdateRecipe};
use sqlx::SqlitePool;

pub async fn get_recipes(pool: &SqlitePool) -> Result<Vec<Recipe>, sqlx::Error> {
    sqlx::query_as::<_, Recipe>(
        "SELECT id, name, instructions, base_servings, created_at, updated_at 
         FROM recipes 
         ORDER BY name"
    )
    .fetch_all(pool)
    .await
}

pub async fn get_recipe(pool: &SqlitePool, id: i64) -> Result<Recipe, sqlx::Error> {
    sqlx::query_as::<_, Recipe>(
        "SELECT id, name, instructions, base_servings, created_at, updated_at 
         FROM recipes 
         WHERE id = ?"
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn get_recipe_with_ingredients(
    pool: &SqlitePool,
    id: i64,
) -> Result<RecipeWithIngredients, sqlx::Error> {
    let recipe = get_recipe(pool, id).await?;
    let ingredients = get_recipe_ingredients_with_details(pool, id).await?;

    Ok(RecipeWithIngredients {
        recipe,
        ingredients,
    })
}

async fn get_recipe_ingredients_with_details(
    pool: &SqlitePool,
    recipe_id: i64,
) -> Result<Vec<RecipeIngredientDetail>, sqlx::Error> {
    sqlx::query_as::<_, RecipeIngredientDetail>(
        "SELECT 
            ri.id, ri.recipe_id, ri.ingredient_id, ri.base_quantity, ri.unit,
            ri.child_multiplier, ri.teen_multiplier, ri.adult_multiplier, ri.notes, ri.created_at,
            i.name as ingredient_name
         FROM recipe_ingredients ri
         JOIN ingredients i ON ri.ingredient_id = i.id
         WHERE ri.recipe_id = ?
         ORDER BY ri.id"
    )
    .bind(recipe_id)
    .fetch_all(pool)
    .await
}

pub async fn create_recipe(
    pool: &SqlitePool,
    recipe: CreateRecipe,
) -> Result<RecipeWithIngredients, sqlx::Error> {
    // Validate base servings is positive
    if recipe.base_servings <= 0 {
        return Err(sqlx::Error::Decode(
            "Base servings must be greater than 0".into()
        ));
    }

    // Validate ingredient quantities and multipliers
    for ingredient in &recipe.ingredients {
        if ingredient.base_quantity <= 0.0 {
            return Err(sqlx::Error::Decode(
                "All ingredient quantities must be greater than 0".into()
            ));
        }

        if let Some(mult) = ingredient.child_multiplier {
            if mult < 0.0 {
                return Err(sqlx::Error::Decode(
                    "Multipliers cannot be negative".into()
                ));
            }
        }
        if let Some(mult) = ingredient.teen_multiplier {
            if mult < 0.0 {
                return Err(sqlx::Error::Decode(
                    "Multipliers cannot be negative".into()
                ));
            }
        }
        if let Some(mult) = ingredient.adult_multiplier {
            if mult < 0.0 {
                return Err(sqlx::Error::Decode(
                    "Multipliers cannot be negative".into()
                ));
            }
        }
    }

    let result = sqlx::query(
        "INSERT INTO recipes (name, instructions, base_servings) 
         VALUES (?, ?, ?)"
    )
    .bind(&recipe.name)
    .bind(&recipe.instructions)
    .bind(recipe.base_servings)
    .execute(pool)
    .await?;

    let recipe_id = result.last_insert_rowid();

    // Insert ingredients
    for ingredient in recipe.ingredients {
        sqlx::query(
            "INSERT INTO recipe_ingredients 
             (recipe_id, ingredient_id, base_quantity, unit, child_multiplier, teen_multiplier, adult_multiplier, notes) 
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(recipe_id)
        .bind(ingredient.ingredient_id)
        .bind(ingredient.base_quantity)
        .bind(&ingredient.unit)
        .bind(ingredient.child_multiplier.unwrap_or(0.5))
        .bind(ingredient.teen_multiplier.unwrap_or(0.75))
        .bind(ingredient.adult_multiplier.unwrap_or(1.0))
        .bind(&ingredient.notes)
        .execute(pool)
        .await?;
    }

    get_recipe_with_ingredients(pool, recipe_id).await
}

pub async fn update_recipe(
    pool: &SqlitePool,
    id: i64,
    recipe: UpdateRecipe,
) -> Result<RecipeWithIngredients, sqlx::Error> {
    let existing = get_recipe(pool, id).await?;

    // Validate base servings if provided
    let final_base_servings = recipe.base_servings.unwrap_or(existing.base_servings);
    if final_base_servings <= 0 {
        return Err(sqlx::Error::Decode(
            "Base servings must be greater than 0".into()
        ));
    }

    // Validate ingredients if provided
    if let Some(ref ingredients) = recipe.ingredients {
        for ingredient in ingredients {
            if ingredient.base_quantity <= 0.0 {
                return Err(sqlx::Error::Decode(
                    "All ingredient quantities must be greater than 0".into()
                ));
            }

            if let Some(mult) = ingredient.child_multiplier {
                if mult < 0.0 {
                    return Err(sqlx::Error::Decode(
                        "Multipliers cannot be negative".into()
                    ));
                }
            }
            if let Some(mult) = ingredient.teen_multiplier {
                if mult < 0.0 {
                    return Err(sqlx::Error::Decode(
                        "Multipliers cannot be negative".into()
                    ));
                }
            }
            if let Some(mult) = ingredient.adult_multiplier {
                if mult < 0.0 {
                    return Err(sqlx::Error::Decode(
                        "Multipliers cannot be negative".into()
                    ));
                }
            }
        }
    }

    sqlx::query(
        "UPDATE recipes
         SET name = ?, instructions = ?,
             base_servings = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?"
    )
    .bind(recipe.name.unwrap_or(existing.name))
    .bind(recipe.instructions.or(existing.instructions))
    .bind(final_base_servings)
    .bind(id)
    .execute(pool)
    .await?;

    // Update ingredients if provided
    if let Some(ingredients) = recipe.ingredients {
        // Delete existing ingredients
        sqlx::query("DELETE FROM recipe_ingredients WHERE recipe_id = ?")
            .bind(id)
            .execute(pool)
            .await?;

        // Insert new ingredients
        for ingredient in ingredients {
            sqlx::query(
                "INSERT INTO recipe_ingredients 
                 (recipe_id, ingredient_id, base_quantity, unit, child_multiplier, teen_multiplier, adult_multiplier, notes) 
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
            )
            .bind(id)
            .bind(ingredient.ingredient_id)
            .bind(ingredient.base_quantity)
            .bind(&ingredient.unit)
            .bind(ingredient.child_multiplier.unwrap_or(0.5))
            .bind(ingredient.teen_multiplier.unwrap_or(0.75))
            .bind(ingredient.adult_multiplier.unwrap_or(1.0))
            .bind(&ingredient.notes)
            .execute(pool)
            .await?;
        }
    }

    get_recipe_with_ingredients(pool, id).await
}

pub async fn delete_recipe(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM recipes WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}
