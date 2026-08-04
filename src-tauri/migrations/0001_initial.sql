CREATE TABLE IF NOT EXISTS foods (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    normalized_name TEXT NOT NULL,
    brand TEXT,
    category TEXT,
    serving_size REAL NOT NULL DEFAULT 100,
    serving_unit TEXT NOT NULL DEFAULT 'g',
    servings_per_container REAL NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS nutrients (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    normalized_name TEXT NOT NULL UNIQUE,
    unit TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS food_nutrients (
    id INTEGER PRIMARY KEY,
    food_id INTEGER NOT NULL REFERENCES foods(id) ON DELETE CASCADE,
    nutrient_id INTEGER NOT NULL REFERENCES nutrients(id) ON DELETE CASCADE,
    amount REAL NOT NULL,
    UNIQUE(food_id, nutrient_id)
);

CREATE INDEX IF NOT EXISTS idx_foods_normalized_name ON foods(normalized_name);
CREATE INDEX IF NOT EXISTS idx_food_nutrients_food_id ON food_nutrients(food_id);
CREATE INDEX IF NOT EXISTS idx_food_nutrients_nutrient_id ON food_nutrients(nutrient_id);
