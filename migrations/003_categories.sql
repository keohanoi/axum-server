-- Add categories table
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    color VARCHAR(7), -- For hex color codes
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(name, user_id)
);

CREATE INDEX idx_categories_user_id ON categories(user_id);
CREATE INDEX idx_categories_name ON categories(name);

CREATE TRIGGER update_categories_updated_at BEFORE UPDATE
    ON categories FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- Add category_id to todos table
ALTER TABLE todos ADD COLUMN category_id UUID REFERENCES categories(id) ON DELETE SET NULL;
CREATE INDEX idx_todos_category_id ON todos(category_id);
