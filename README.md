# Axum Todo Server

A comprehensive todo management API built with Rust and Axum, featuring user management, categories, tags, analytics, and batch operations.

## Features

### 🔐 User Management
- User registration and authentication
- JWT-based authentication
- User profile management
- Secure password hashing with bcrypt

### 📝 Enhanced Todo Management
- Create, read, update, delete todos
- Priority levels (0-4: None, Low, Medium, High, Critical)
- Due dates with overdue detection
- Rich filtering and search capabilities
- Pagination support

### 🏷️ Organization
- **Categories**: Organize todos into colored categories
- **Tags**: Flexible tagging system with many-to-many relationships
- **Batch Operations**: Update or delete multiple todos at once

### 📊 Analytics & Statistics
- Todo completion statistics
- Priority distribution analysis
- Category breakdown
- Overdue todos tracking

### 🔍 Advanced Filtering
- Filter by completion status
- Filter by category
- Filter by priority level
- Filter by tags
- Search in title and description
- Show only overdue todos

## Getting Started

### Prerequisites
- Rust (latest stable version)
- PostgreSQL database
- Environment variables configured

### Environment Setup
Create a `.env` file in the project root:

```env
DATABASE_URL=postgres://username:password@localhost/todo_db
RUST_LOG=info
HOST=127.0.0.1
PORT=3000
```

### Database Setup
1. Create a PostgreSQL database
2. Run migrations:
```bash
cargo run --bin migrate
```

### Running the Server
```bash
cargo run
```

The server will start at `http://localhost:3000`

## API Documentation

See [API_DOCS.md](./API_DOCS.md) for complete API documentation with examples.

## Quick Examples

### 1. Register a User
```bash
curl -X POST http://localhost:3000/api/users/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe",
    "email": "john@example.com",
    "password": "secure123",
    "full_name": "John Doe"
  }'
```

### 2. Login
```bash
curl -X POST http://localhost:3000/api/users/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "john_doe", 
    "password": "secure123"
  }'
```

### 3. Create a Category
```bash
curl -X POST "http://localhost:3000/api/categories?user_id=YOUR_USER_ID" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "name": "Work",
    "description": "Work related tasks",
    "color": "#ff5733"
  }'
```

### 4. Create a Todo with Priority and Tags
```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "title": "Complete project proposal",
    "description": "Write and submit Q1 proposal",
    "category_id": "YOUR_CATEGORY_ID",
    "priority": 4,
    "due_date": "2024-12-31T23:59:59Z",
    "tags": ["urgent", "important"]
  }'
```

### 5. Get Filtered Todos
```bash
# Get high priority todos that are overdue
curl "http://localhost:3000/api/todos?priority=4&overdue=true" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Search todos by text
curl "http://localhost:3000/api/todos?search=project" \
  -H "Authorization: Bearer YOUR_TOKEN"

# Get todos by category
curl "http://localhost:3000/api/todos?category_id=YOUR_CATEGORY_ID" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

### 6. Batch Update Todos
```bash
curl -X PATCH http://localhost:3000/api/todos/batch \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -d '{
    "todo_ids": ["uuid1", "uuid2", "uuid3"],
    "completed": true,
    "priority": 1
  }'
```

### 7. Get Statistics
```bash
curl "http://localhost:3000/api/stats/todos?user_id=YOUR_USER_ID" \
  -H "Authorization: Bearer YOUR_TOKEN"
```

## Testing Scripts

The project includes several testing scripts:

### Comprehensive API Test
```bash
./scripts/test_api.sh
```

### Interactive Todo Client
```bash
# Create a todo
./scripts/todo_client.sh create -t "Buy groceries" -d "Milk, bread, eggs"

# List todos with pagination
./scripts/todo_client.sh list -p 1 -l 10

# Search todos
./scripts/todo_client.sh list -s "groceries"

# Update a todo
./scripts/todo_client.sh update TODO_ID -c true

# Delete a todo
./scripts/todo_client.sh delete TODO_ID
```

## Architecture

### Database Schema
- **users**: User accounts and authentication
- **todos**: Todo items with relations to users and categories
- **categories**: User-defined categories with colors
- **tags**: Flexible tagging system
- **todo_tags**: Many-to-many relationship between todos and tags

### Key Components
- **Handlers**: Request processing logic for each endpoint type
- **Models**: Data structures and validation rules
- **Middleware**: Authentication, CORS, and logging
- **Error Handling**: Comprehensive error types and responses

### Authentication
- JWT tokens with configurable expiration
- Bcrypt password hashing
- Middleware-based authentication (optional for backwards compatibility)

## Priority Levels

| Level | Name     | Description           |
|-------|----------|-----------------------|
| 0     | None     | No specific priority  |
| 1     | Low      | Can wait              |
| 2     | Medium   | Normal priority       |
| 3     | High     | Important task        |
| 4     | Critical | Urgent, do immediately|

## Development

### Project Structure
```
src/
├── config/          # Configuration management
├── db/              # Database connection and migrations
├── error/           # Error types and handling
├── handlers/        # Request handlers
│   ├── mod.rs       # Todo handlers
│   ├── users.rs     # User management
│   ├── categories.rs # Category management
│   ├── tags.rs      # Tag management
│   ├── stats.rs     # Analytics
│   └── batch.rs     # Batch operations
├── middleware/      # Auth, CORS, logging
├── models/          # Data models and validation
├── routes/          # Route definitions
├── lib.rs           # Library root
└── main.rs          # Application entry point
```

### Adding New Features
1. Define models in `src/models/mod.rs`
2. Create handlers in appropriate handler files
3. Add routes in `src/routes/mod.rs`
4. Update database schema with new migrations
5. Add tests and documentation

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Update documentation
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## TODO

### Future Enhancements
- [ ] File attachments for todos
- [ ] Todo templates
- [ ] Recurring todos
- [ ] Todo sharing between users
- [ ] Email notifications
- [ ] Calendar integration
- [ ] Real-time updates with WebSockets
- [ ] Export/import functionality
- [ ] Advanced reporting
- [ ] Mobile app support
