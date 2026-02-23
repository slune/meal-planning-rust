-- Remove UNIQUE(meal_plan_id, meal_type) constraint to allow multiple meals of the same type per day
CREATE TABLE planned_meals_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    meal_plan_id INTEGER NOT NULL,
    recipe_id INTEGER NOT NULL,
    meal_type TEXT NOT NULL CHECK(meal_type IN ('breakfast', 'morning_snack', 'lunch', 'afternoon_snack', 'dinner')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE RESTRICT
);

INSERT INTO planned_meals_new SELECT * FROM planned_meals;

DROP TABLE planned_meals;

ALTER TABLE planned_meals_new RENAME TO planned_meals;

CREATE INDEX IF NOT EXISTS idx_planned_meals_plan ON planned_meals(meal_plan_id);
CREATE INDEX IF NOT EXISTS idx_planned_meals_recipe ON planned_meals(recipe_id);
