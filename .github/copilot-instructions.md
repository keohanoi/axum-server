# Axum Todo Server - AI Coding Instructions

## Architecture Overview

This is a **comprehensive todo management API** built with Axum, featuring user authentication, categories, tags, priorities, and analytics. The codebase follows a **modular service-oriented architecture** with clear separation of concerns:

- **Database-first design**: PostgreSQL with SQLx for type-safe queries and migrations
- **JWT authentication**: Optional middleware-based auth with bcrypt password hashing  
- **Rich domain model**: Users → Categories/Tags → Todos with many-to-many relationships
- **Comprehensive error handling**: Custom `AppError` enum with proper HTTP status mapping

## Key Project Patterns

### Handler Pattern
All handlers follow this signature pattern:
```rust
pub async fn handler_name(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>, // for specific resources
    Query(params): Query<QueryStruct>, // for filtering
    Json(payload): Json<RequestStruct>, // for mutations
) -> Result<(StatusCode, Json<ResponseStruct>)>
```

### Database Relations Helper
Complex queries use the `get_todo_with_relations()` helper pattern to build full responses with nested category/tag data. When adding new relations, follow this pattern in `src/handlers/mod.rs`.

### Migration-Driven Development
Database changes require **sequential migrations** in `migrations/`. Current schema supports:
- `001_init.sql`: Base todos table
- `002_users.sql`: User management + todos.user_id FK
- `003_categories.sql`: Categories with colors + todos.category_id FK  
- `004_tags_and_features.sql`: Tags + junction table + priority/due_date fields

## Critical Development Commands

```bash
# Build and run (auto-runs migrations)
cargo run

# Database operations
cargo install sqlx-cli  # First time only
sqlx migrate add new_feature  # Create new migration
sqlx migrate run  # Apply pending migrations

# Testing
./scripts/test_api.sh  # Full API integration test
./scripts/todo_client.sh create -t "Test" -d "Description"  # CLI client
```

## Request/Response Patterns

### Validation
Use `#[validate()]` attributes on request structs. Example:
```rust
#[derive(Debug, Deserialize, Validate)]
pub struct CreateTodoRequest {
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    #[validate(range(min = 0, max = 4))]
    pub priority: Option<i32>,
}
```

### Error Handling  
Always return `Result<T>` (alias for `Result<T, AppError>`). Use specific error types:
```rust
AppError::NotFound(format!("Todo with id {} not found", id))
AppError::Conflict("Username already exists".to_string())
AppError::Validation(e.to_string())
```

### Filtering & Pagination
Query parameters follow this pattern in `TodoQuery` struct:
- `page`/`per_page`: Standard pagination
- `search`: ILIKE search across title/description  
- `completed`/`priority`/`category_id`: Exact filtering
- `tag`: Tag name filtering via junction table
- `overdue`: Boolean for due_date < now AND completed = false

## Authentication Integration

JWT middleware in `src/middleware/auth.rs` is **optional by default** for backwards compatibility. To require auth:
1. Update middleware to return `Err(StatusCode::UNAUTHORIZED)` instead of allowing passthrough
2. Extract user from request extensions: `request.extensions().get::<Claims>()`
3. Use `user_id` for filtering user-specific resources

## Configuration & Environment

- **Required**: `DATABASE_URL` for PostgreSQL connection
- **Optional**: `SERVER_HOST` (default: 127.0.0.1), `SERVER_PORT` (default: 3000), `RUST_LOG` (default: info)
- **Deployment**: Auto-runs migrations on startup, supports connection pooling (max 10 connections)

## Testing & Integration

The `scripts/` directory contains production-ready testing tools:
- `test_api.sh`: Full workflow test (register → login → create todos → batch operations → stats)
- `todo_client.sh`: Interactive CLI with help system and error handling
- Both scripts handle JWT token extraction and passing automatically

## Adding New Features

1. **Database**: Create migration in `migrations/XXX_feature.sql`
2. **Models**: Add structs to `src/models/mod.rs` with proper validation
3. **Handlers**: Create new handler file in `src/handlers/` following existing patterns
4. **Routes**: Add routes to `src/routes/mod.rs` with appropriate HTTP methods
5. **Relations**: Update `get_todo_with_relations()` if adding todo associations
6. **Documentation**: Update `API_DOCS.md` with new endpoints and examples
