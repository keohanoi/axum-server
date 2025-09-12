-- Add users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);

CREATE TRIGGER update_users_updated_at BEFORE UPDATE
    ON users FOR EACH ROW EXECUTE PROCEDURE update_updated_at_column();

-- Add user_id to todos table
ALTER TABLE todos ADD COLUMN user_id UUID REFERENCES users(id) ON DELETE CASCADE;
CREATE INDEX idx_todos_user_id ON todos(user_id);
