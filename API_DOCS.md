# Axum Todo Server API Documentation

A comprehensive todo management API built with Axum, featuring users, categories, tags, and analytics.

## Base URL
```
http://localhost:3000
```

## Authentication
Most endpoints support JWT authentication via the `Authorization` header:
```
Authorization: Bearer <jwt_token>
```

## Endpoints

### Health Check
- **GET** `/health` - Server health check

### User Management

#### Register User
- **POST** `/api/users/register`
- **Body:**
```json
{
  "username": "john_doe",
  "email": "john@example.com", 
  "password": "secret123",
  "full_name": "John Doe"
}
```

#### Login User
- **POST** `/api/users/login`
- **Body:**
```json
{
  "username": "john_doe",
  "password": "secret123"
}
```
- **Response:**
```json
{
  "user": {
    "id": "uuid",
    "username": "john_doe",
    "email": "john@example.com",
    "full_name": "John Doe",
    "is_active": true,
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  },
  "token": "jwt_token_here"
}
```

#### Get User Profile
- **GET** `/api/users/{id}`

#### Update User Profile
- **PATCH** `/api/users/{id}`
- **Body:**
```json
{
  "email": "newemail@example.com",
  "full_name": "Updated Name",
  "is_active": true
}
```

#### Delete User
- **DELETE** `/api/users/{id}`

### Todo Management

#### Create Todo
- **POST** `/api/todos`
- **Body:**
```json
{
  "title": "Buy groceries",
  "description": "Get milk, bread, and eggs",
  "category_id": "uuid",
  "priority": 2,
  "due_date": "2024-12-31T23:59:59Z",
  "tags": ["shopping", "food"]
}
```

#### Get Todos (with filtering)
- **GET** `/api/todos?page=1&per_page=10&completed=false&category_id=uuid&priority=2&tag=work&search=grocery&overdue=true`

Query parameters:
- `page` (optional): Page number (default: 1)
- `per_page` (optional): Items per page (default: 10, max: 100)
- `completed` (optional): Filter by completion status
- `category_id` (optional): Filter by category
- `priority` (optional): Filter by priority (0-4)
- `tag` (optional): Filter by tag name
- `search` (optional): Search in title and description
- `overdue` (optional): Show only overdue incomplete todos

#### Get Single Todo
- **GET** `/api/todos/{id}`

#### Update Todo
- **PATCH** `/api/todos/{id}`
- **Body:**
```json
{
  "title": "Updated title",
  "completed": true,
  "priority": 3,
  "tags": ["updated", "tags"]
}
```

#### Delete Todo
- **DELETE** `/api/todos/{id}`

### Batch Operations

#### Batch Update Todos
- **PATCH** `/api/todos/batch`
- **Body:**
```json
{
  "todo_ids": ["uuid1", "uuid2", "uuid3"],
  "completed": true,
  "category_id": "uuid",
  "priority": 1
}
```

#### Batch Delete Todos
- **DELETE** `/api/todos/batch`
- **Body:**
```json
["uuid1", "uuid2", "uuid3"]
```

### Category Management

#### Create Category
- **POST** `/api/categories?user_id=uuid`
- **Body:**
```json
{
  "name": "Work",
  "description": "Work related tasks",
  "color": "#ff5733"
}
```

#### Get Categories
- **GET** `/api/categories?user_id=uuid`

#### Get Single Category
- **GET** `/api/categories/{id}`

#### Update Category
- **PATCH** `/api/categories/{id}`
- **Body:**
```json
{
  "name": "Updated Work",
  "description": "Updated description",
  "color": "#33ff57"
}
```

#### Delete Category
- **DELETE** `/api/categories/{id}`

### Tag Management

#### Create Tag
- **POST** `/api/tags?user_id=uuid`
- **Body:**
```json
{
  "name": "urgent"
}
```

#### Get Tags
- **GET** `/api/tags?user_id=uuid`

#### Get Single Tag
- **GET** `/api/tags/{id}`

#### Delete Tag
- **DELETE** `/api/tags/{id}`

#### Assign Tag to Todo
- **PUT** `/api/todos/{todo_id}/tags/{tag_id}`

#### Remove Tag from Todo
- **DELETE** `/api/todos/{todo_id}/tags/{tag_id}`

### Statistics & Analytics

#### Get Todo Statistics
- **GET** `/api/stats/todos?user_id=uuid`

Response includes:
- Total todos count
- Completed todos count  
- Pending todos count
- Overdue todos count
- Breakdown by priority
- Breakdown by category

**Example Response:**
```json
{
  "total_todos": 25,
  "completed_todos": 15,
  "pending_todos": 10,
  "overdue_todos": 3,
  "todos_by_priority": [
    {"priority": 0, "count": 5},
    {"priority": 1, "count": 8},
    {"priority": 2, "count": 7},
    {"priority": 3, "count": 3},
    {"priority": 4, "count": 2}
  ],
  "todos_by_category": [
    {"category_id": "uuid1", "category_name": "Work", "count": 12},
    {"category_id": "uuid2", "category_name": "Personal", "count": 8},
    {"category_id": null, "category_name": null, "count": 5}
  ]
}
```

## Data Models

### Priority Levels
- `0`: No priority
- `1`: Low priority
- `2`: Medium priority
- `3`: High priority
- `4`: Critical priority

### Todo Response Model
```json
{
  "id": "uuid",
  "title": "Task title",
  "description": "Task description",
  "completed": false,
  "user_id": "uuid",
  "category": {
    "id": "uuid",
    "name": "Category Name",
    "description": "Category description",
    "color": "#ff5733",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  },
  "priority": 2,
  "due_date": "2024-12-31T23:59:59Z",
  "tags": [
    {
      "id": "uuid",
      "name": "tag1",
      "created_at": "2024-01-01T00:00:00Z"
    }
  ],
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

## Error Responses

All errors follow this format:
```json
{
  "error": "Error message describing what went wrong"
}
```

Common HTTP status codes:
- `200 OK`: Success
- `201 Created`: Resource created successfully
- `204 No Content`: Success with no response body
- `400 Bad Request`: Invalid request data
- `401 Unauthorized`: Authentication required
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource already exists
- `500 Internal Server Error`: Server error

## Getting Started

1. Start the server:
```bash
cargo run
```

2. Create a user:
```bash
curl -X POST http://localhost:3000/api/users/register \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","email":"test@example.com","password":"password123"}'
```

3. Login to get a token:
```bash
curl -X POST http://localhost:3000/api/users/login \
  -H "Content-Type: application/json" \
  -d '{"username":"testuser","password":"password123"}'
```

4. Create a todo:
```bash
curl -X POST http://localhost:3000/api/todos \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  -d '{"title":"My first todo","description":"Get started with the API"}'
```
