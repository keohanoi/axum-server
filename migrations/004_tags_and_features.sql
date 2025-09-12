-- Add tags table
CREATE TABLE tags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(50) NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(name, user_id)
);

CREATE INDEX idx_tags_user_id ON tags(user_id);
CREATE INDEX idx_tags_name ON tags(name);

-- Add todo_tags junction table
CREATE TABLE todo_tags (
    todo_id UUID NOT NULL REFERENCES todos(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (todo_id, tag_id)
);

CREATE INDEX idx_todo_tags_todo_id ON todo_tags(todo_id);
CREATE INDEX idx_todo_tags_tag_id ON todo_tags(tag_id);

-- Add priority and due_date to todos
ALTER TABLE todos ADD COLUMN priority INTEGER DEFAULT 0 CHECK (priority >= 0 AND priority <= 4);
ALTER TABLE todos ADD COLUMN due_date TIMESTAMPTZ;

CREATE INDEX idx_todos_priority ON todos(priority);
CREATE INDEX idx_todos_due_date ON todos(due_date);
