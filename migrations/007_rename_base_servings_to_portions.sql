-- Rename recipe serving-count terminology to match the UI/domain language.
ALTER TABLE recipes RENAME COLUMN base_servings TO portions;
