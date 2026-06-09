-- Create editable meal types and remove the hardcoded planned_meals CHECK.

CREATE TABLE IF NOT EXISTS meal_types (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

INSERT OR IGNORE INTO meal_types (key, name, sort_order) VALUES
    ('breakfast', 'Breakfast', 1),
    ('morning_snack', 'Morning Snack', 2),
    ('lunch', 'Lunch', 3),
    ('afternoon_snack', 'Afternoon Snack', 4),
    ('dinner', 'Dinner', 5);

-- SQLite cannot drop a CHECK constraint in place, so rebuild planned_meals.
PRAGMA foreign_keys = OFF;

BEGIN TRANSACTION;

DROP TABLE IF EXISTS planned_meals_new;

CREATE TABLE planned_meals_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    meal_plan_id INTEGER NOT NULL,
    recipe_id INTEGER NOT NULL,
    meal_type TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE RESTRICT
);

INSERT INTO planned_meals_new SELECT * FROM planned_meals;

DROP TABLE planned_meals;

ALTER TABLE planned_meals_new RENAME TO planned_meals;

CREATE INDEX IF NOT EXISTS idx_planned_meals_plan ON planned_meals(meal_plan_id);
CREATE INDEX IF NOT EXISTS idx_planned_meals_recipe ON planned_meals(recipe_id);
CREATE INDEX IF NOT EXISTS idx_planned_meals_type ON planned_meals(meal_type);

COMMIT;

PRAGMA foreign_keys = ON;
