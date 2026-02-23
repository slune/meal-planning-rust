-- Create meal_plans table
CREATE TABLE IF NOT EXISTS meal_plans (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    camp_id INTEGER NOT NULL,
    date DATE NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (camp_id) REFERENCES camps(id) ON DELETE CASCADE,
    UNIQUE(camp_id, date)
);

-- Create planned_meals table
CREATE TABLE IF NOT EXISTS planned_meals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    meal_plan_id INTEGER NOT NULL,
    recipe_id INTEGER NOT NULL,
    meal_type TEXT NOT NULL CHECK(meal_type IN ('breakfast', 'morning_snack', 'lunch', 'afternoon_snack', 'dinner')),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (meal_plan_id) REFERENCES meal_plans(id) ON DELETE CASCADE,
    FOREIGN KEY (recipe_id) REFERENCES recipes(id) ON DELETE RESTRICT,
    UNIQUE(meal_plan_id, meal_type)
);

-- Create meal_attendance table
CREATE TABLE IF NOT EXISTS meal_attendance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    planned_meal_id INTEGER NOT NULL,
    children INTEGER NOT NULL DEFAULT 0,
    teens INTEGER NOT NULL DEFAULT 0,
    adults INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (planned_meal_id) REFERENCES planned_meals(id) ON DELETE CASCADE,
    UNIQUE(planned_meal_id)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_meal_plans_camp ON meal_plans(camp_id);
CREATE INDEX IF NOT EXISTS idx_meal_plans_date ON meal_plans(date);
CREATE INDEX IF NOT EXISTS idx_planned_meals_plan ON planned_meals(meal_plan_id);
CREATE INDEX IF NOT EXISTS idx_planned_meals_recipe ON planned_meals(recipe_id);
CREATE INDEX IF NOT EXISTS idx_meal_attendance_meal ON meal_attendance(planned_meal_id);
