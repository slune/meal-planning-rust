-- Create categories table
CREATE TABLE IF NOT EXISTS categories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default categories (ignore if already exists)
INSERT OR IGNORE INTO categories (id, name, sort_order) VALUES
    (1, 'Meat & Fish', 1),
    (2, 'Vegetables', 2),
    (3, 'Fruits', 3),
    (4, 'Dairy', 4),
    (5, 'Baking Goods', 5),
    (6, 'Grains & Pasta', 6),
    (7, 'Spices & Condiments', 7),
    (8, 'Beverages', 8),
    (9, 'Canned Goods', 9),
    (99, 'Other', 99);
