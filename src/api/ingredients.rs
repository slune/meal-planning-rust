use crate::models::{
    CreateIngredient, Ingredient, IngredientRecipeUsage, IngredientUsage, MergeIngredientsResult,
    UpdateIngredient,
};
use sqlx::{QueryBuilder, Sqlite, SqlitePool};
use std::collections::BTreeMap;

#[derive(sqlx::FromRow)]
struct DuplicateRecipeIngredientGroup {
    recipe_id: i64,
    unit: String,
    child_multiplier: Option<f64>,
    teen_multiplier: Option<f64>,
    adult_multiplier: Option<f64>,
    notes: Option<String>,
    keep_id: i64,
    total_quantity: f64,
}

#[derive(sqlx::FromRow)]
struct IngredientUsageRow {
    ingredient_id: i64,
    recipe_id: i64,
    recipe_name: String,
}

pub async fn get_ingredients(pool: &SqlitePool) -> Result<Vec<Ingredient>, sqlx::Error> {
    sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category_id, primary_unit, secondary_unit, created_at, updated_at 
         FROM ingredients 
         ORDER BY name",
    )
    .fetch_all(pool)
    .await
}

pub async fn get_ingredient_usage(pool: &SqlitePool) -> Result<Vec<IngredientUsage>, sqlx::Error> {
    let rows = sqlx::query_as::<_, IngredientUsageRow>(
        "SELECT
            ri.ingredient_id,
            r.id as recipe_id,
            r.name as recipe_name
         FROM recipe_ingredients ri
         JOIN recipes r ON r.id = ri.recipe_id
         GROUP BY ri.ingredient_id, r.id, r.name
         ORDER BY ri.ingredient_id, r.name",
    )
    .fetch_all(pool)
    .await?;

    let mut usage_by_ingredient = BTreeMap::<i64, Vec<IngredientRecipeUsage>>::new();
    for row in rows {
        usage_by_ingredient
            .entry(row.ingredient_id)
            .or_default()
            .push(IngredientRecipeUsage {
                id: row.recipe_id,
                name: row.recipe_name,
            });
    }

    Ok(usage_by_ingredient
        .into_iter()
        .map(|(ingredient_id, recipes)| IngredientUsage {
            ingredient_id,
            recipes,
        })
        .collect())
}

pub async fn get_ingredients_by_category(
    pool: &SqlitePool,
    category_id: i64,
) -> Result<Vec<Ingredient>, sqlx::Error> {
    sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category_id, primary_unit, secondary_unit, created_at, updated_at 
         FROM ingredients 
         WHERE category_id = ?
         ORDER BY name",
    )
    .bind(category_id)
    .fetch_all(pool)
    .await
}

pub async fn get_ingredient(pool: &SqlitePool, id: i64) -> Result<Ingredient, sqlx::Error> {
    sqlx::query_as::<_, Ingredient>(
        "SELECT id, name, category_id, primary_unit, secondary_unit, created_at, updated_at 
         FROM ingredients 
         WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
}

pub async fn create_ingredient(
    pool: &SqlitePool,
    ingredient: CreateIngredient,
) -> Result<Ingredient, sqlx::Error> {
    let result = sqlx::query(
        "INSERT INTO ingredients (name, category_id, primary_unit, secondary_unit) 
         VALUES (?, ?, ?, ?)",
    )
    .bind(&ingredient.name)
    .bind(ingredient.category_id)
    .bind(&ingredient.primary_unit)
    .bind(&ingredient.secondary_unit)
    .execute(pool)
    .await?;

    get_ingredient(pool, result.last_insert_rowid()).await
}

pub async fn update_ingredient(
    pool: &SqlitePool,
    id: i64,
    ingredient: UpdateIngredient,
) -> Result<Ingredient, sqlx::Error> {
    let existing = get_ingredient(pool, id).await?;

    sqlx::query(
        "UPDATE ingredients 
         SET name = ?, category_id = ?, primary_unit = ?, 
             secondary_unit = ?, updated_at = CURRENT_TIMESTAMP 
         WHERE id = ?",
    )
    .bind(ingredient.name.unwrap_or(existing.name))
    .bind(ingredient.category_id.unwrap_or(existing.category_id))
    .bind(ingredient.primary_unit.unwrap_or(existing.primary_unit))
    .bind(ingredient.secondary_unit.unwrap_or(existing.secondary_unit))
    .bind(id)
    .execute(pool)
    .await?;

    get_ingredient(pool, id).await
}

pub async fn delete_ingredient(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM ingredients WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn merge_ingredients(
    pool: &SqlitePool,
    target_id: i64,
    ingredient_ids: Vec<i64>,
    name: String,
    category_id: i64,
    primary_unit: String,
    secondary_unit: Option<String>,
) -> Result<MergeIngredientsResult, sqlx::Error> {
    let name = name.trim().to_string();
    let primary_unit = primary_unit.trim().to_string();
    let secondary_unit = secondary_unit.and_then(|unit| {
        let unit = unit.trim().to_string();
        (!unit.is_empty()).then_some(unit)
    });

    if name.is_empty() || primary_unit.is_empty() {
        return Err(sqlx::Error::Decode(
            "Name and primary unit are required".into(),
        ));
    }

    let mut ingredient_ids = ingredient_ids;
    ingredient_ids.sort_unstable();
    ingredient_ids.dedup();

    if ingredient_ids.len() < 2 {
        return Err(sqlx::Error::Decode(
            "Select at least two ingredients to merge".into(),
        ));
    }

    if !ingredient_ids.contains(&target_id) {
        return Err(sqlx::Error::Decode(
            "Merge target must be one of the selected ingredients".into(),
        ));
    }

    let selected = get_ingredients_by_ids(pool, &ingredient_ids).await?;
    if selected.len() != ingredient_ids.len() {
        return Err(sqlx::Error::Decode(
            "One or more selected ingredients no longer exist".into(),
        ));
    }

    let first = &selected[0];
    if selected.iter().any(|ingredient| {
        ingredient.primary_unit != first.primary_unit
            || ingredient.secondary_unit != first.secondary_unit
    }) {
        return Err(sqlx::Error::Decode(
            "Selected ingredients must have the same primary and secondary units".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    sqlx::query(
        "UPDATE ingredients
         SET name = ?, category_id = ?, primary_unit = ?,
             secondary_unit = ?, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?",
    )
    .bind(&name)
    .bind(category_id)
    .bind(&primary_unit)
    .bind(&secondary_unit)
    .bind(target_id)
    .execute(&mut *tx)
    .await?;

    let source_ids: Vec<i64> = ingredient_ids
        .iter()
        .copied()
        .filter(|id| *id != target_id)
        .collect();

    let updated_recipe_rows = if source_ids.is_empty() {
        0
    } else {
        let mut update_query: QueryBuilder<Sqlite> =
            QueryBuilder::new("UPDATE recipe_ingredients SET ingredient_id = ");
        update_query.push_bind(target_id);
        update_query.push(" WHERE ingredient_id IN (");
        push_i64_list(&mut update_query, &source_ids);
        update_query.push(")");

        update_query
            .build()
            .execute(&mut *tx)
            .await?
            .rows_affected()
    };

    let combined_recipe_rows = combine_duplicate_recipe_ingredients(&mut tx, target_id).await?;

    let deleted_ingredient_count = if source_ids.is_empty() {
        0
    } else {
        let mut delete_query: QueryBuilder<Sqlite> =
            QueryBuilder::new("DELETE FROM ingredients WHERE id IN (");
        push_i64_list(&mut delete_query, &source_ids);
        delete_query.push(")");
        delete_query
            .build()
            .execute(&mut *tx)
            .await?
            .rows_affected()
    };

    tx.commit().await?;

    Ok(MergeIngredientsResult {
        ingredient: get_ingredient(pool, target_id).await?,
        merged_ingredient_count: ingredient_ids.len(),
        updated_recipe_rows,
        combined_recipe_rows,
        deleted_ingredient_count,
    })
}

async fn get_ingredients_by_ids(
    pool: &SqlitePool,
    ingredient_ids: &[i64],
) -> Result<Vec<Ingredient>, sqlx::Error> {
    let mut query: QueryBuilder<Sqlite> = QueryBuilder::new(
        "SELECT id, name, category_id, primary_unit, secondary_unit, created_at, updated_at
         FROM ingredients
         WHERE id IN (",
    );
    push_i64_list(&mut query, ingredient_ids);
    query.push(") ORDER BY id");

    query.build_query_as::<Ingredient>().fetch_all(pool).await
}

fn push_i64_list(query: &mut QueryBuilder<Sqlite>, values: &[i64]) {
    let mut separated = query.separated(", ");
    for value in values {
        separated.push_bind(value);
    }
}

async fn combine_duplicate_recipe_ingredients(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    ingredient_id: i64,
) -> Result<u64, sqlx::Error> {
    let groups = sqlx::query_as::<_, DuplicateRecipeIngredientGroup>(
        "SELECT
            recipe_id,
            unit,
            child_multiplier,
            teen_multiplier,
            adult_multiplier,
            notes,
            MIN(id) as keep_id,
            SUM(base_quantity) as total_quantity
         FROM recipe_ingredients
         WHERE ingredient_id = ?
         GROUP BY recipe_id, unit, child_multiplier, teen_multiplier, adult_multiplier, notes
         HAVING COUNT(*) > 1",
    )
    .bind(ingredient_id)
    .fetch_all(&mut **tx)
    .await?;

    let mut combined_rows = 0;

    for group in groups {
        sqlx::query("UPDATE recipe_ingredients SET base_quantity = ? WHERE id = ?")
            .bind(group.total_quantity)
            .bind(group.keep_id)
            .execute(&mut **tx)
            .await?;

        let deleted = sqlx::query(
            "DELETE FROM recipe_ingredients
             WHERE ingredient_id = ?
               AND recipe_id = ?
               AND unit = ?
               AND child_multiplier IS ?
               AND teen_multiplier IS ?
               AND adult_multiplier IS ?
               AND notes IS ?
               AND id <> ?",
        )
        .bind(ingredient_id)
        .bind(group.recipe_id)
        .bind(&group.unit)
        .bind(group.child_multiplier)
        .bind(group.teen_multiplier)
        .bind(group.adult_multiplier)
        .bind(&group.notes)
        .bind(group.keep_id)
        .execute(&mut **tx)
        .await?
        .rows_affected();

        combined_rows += deleted;
    }

    Ok(combined_rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    async fn setup_pool() -> SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        for migration in [
            include_str!("../../migrations/001_create_categories.sql"),
            include_str!("../../migrations/002_create_ingredients.sql"),
            include_str!("../../migrations/003_create_recipes.sql"),
            include_str!("../../migrations/007_rename_base_servings_to_portions.sql"),
        ] {
            sqlx::query(migration).execute(&pool).await.unwrap();
        }

        pool
    }

    async fn insert_ingredient(pool: &SqlitePool, name: &str, unit: &str) -> i64 {
        sqlx::query("INSERT INTO ingredients (name, category_id, primary_unit) VALUES (?, 3, ?)")
            .bind(name)
            .bind(unit)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    async fn insert_recipe(pool: &SqlitePool, name: &str) -> i64 {
        sqlx::query("INSERT INTO recipes (name, portions) VALUES (?, 10)")
            .bind(name)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    #[tokio::test]
    async fn get_ingredient_usage_returns_distinct_recipe_names() {
        let pool = setup_pool().await;
        let flour = insert_ingredient(&pool, "Mouka", "g").await;
        let breakfast = insert_recipe(&pool, "Palačinky").await;
        let dinner = insert_recipe(&pool, "Chleba").await;

        for recipe_id in [breakfast, breakfast, dinner] {
            sqlx::query(
                "INSERT INTO recipe_ingredients
                 (recipe_id, ingredient_id, base_quantity, unit)
                 VALUES (?, ?, 100.0, 'g')",
            )
            .bind(recipe_id)
            .bind(flour)
            .execute(&pool)
            .await
            .unwrap();
        }

        let usage = get_ingredient_usage(&pool).await.unwrap();

        assert_eq!(usage.len(), 1);
        assert_eq!(usage[0].ingredient_id, flour);
        assert_eq!(
            usage[0]
                .recipes
                .iter()
                .map(|recipe| recipe.name.as_str())
                .collect::<Vec<_>>(),
            vec!["Chleba", "Palačinky"]
        );
    }

    #[tokio::test]
    async fn merge_ingredients_repoints_and_combines_safe_recipe_rows() {
        let pool = setup_pool().await;
        let orange = insert_ingredient(&pool, "Pomeranč", "g").await;
        let oranges = insert_ingredient(&pool, "Pomeranče", "g").await;
        let recipe = insert_recipe(&pool, "Snack").await;

        for (ingredient_id, qty) in [(orange, 2.0), (oranges, 3.0)] {
            sqlx::query(
                "INSERT INTO recipe_ingredients
                 (recipe_id, ingredient_id, base_quantity, unit, child_multiplier, teen_multiplier, adult_multiplier)
                 VALUES (?, ?, ?, 'ks', 1.0, 1.0, 1.0)",
            )
            .bind(recipe)
            .bind(ingredient_id)
            .bind(qty)
            .execute(&pool)
            .await
            .unwrap();
        }

        let result = merge_ingredients(
            &pool,
            orange,
            vec![orange, oranges],
            "Pomeranč".to_string(),
            3,
            "g".to_string(),
            None,
        )
        .await
        .unwrap();

        assert_eq!(result.merged_ingredient_count, 2);
        assert_eq!(result.updated_recipe_rows, 1);
        assert_eq!(result.combined_recipe_rows, 1);
        assert_eq!(result.deleted_ingredient_count, 1);

        let rows: Vec<(i64, f64, String)> = sqlx::query_as(
            "SELECT ingredient_id, base_quantity, unit FROM recipe_ingredients WHERE recipe_id = ?",
        )
        .bind(recipe)
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(rows, vec![(orange, 5.0, "ks".to_string())]);

        let source_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ingredients WHERE id = ?")
            .bind(oranges)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(source_count, 0);
    }

    #[tokio::test]
    async fn merge_ingredients_rejects_different_unit_config() {
        let pool = setup_pool().await;
        let orange = insert_ingredient(&pool, "Pomeranč", "g").await;
        let juice = insert_ingredient(&pool, "Džus pomeranč", "l").await;

        let result = merge_ingredients(
            &pool,
            orange,
            vec![orange, juice],
            "Pomeranč".to_string(),
            3,
            "g".to_string(),
            None,
        )
        .await;

        assert!(result.is_err());
    }
}
