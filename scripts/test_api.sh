#!/bin/bash

# API Testing Script for Axum Todo Server
# This script demonstrates all the new endpoints

BASE_URL="http://localhost:3000"
CONTENT_TYPE="Content-Type: application/json"

echo "=== Axum Todo Server API Test ==="

# Test user registration
echo -e "\n1. Registering a new user..."
REGISTER_RESPONSE=$(curl -s -X POST "$BASE_URL/api/users/register" \
  -H "$CONTENT_TYPE" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123",
    "full_name": "Test User"
  }')
echo "Register response: $REGISTER_RESPONSE"

# Test user login
echo -e "\n2. Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$BASE_URL/api/users/login" \
  -H "$CONTENT_TYPE" \
  -d '{
    "username": "testuser",
    "password": "password123"
  }')
echo "Login response: $LOGIN_RESPONSE"

# Extract token and user ID
TOKEN=$(echo $LOGIN_RESPONSE | jq -r '.token // empty')
USER_ID=$(echo $LOGIN_RESPONSE | jq -r '.user.id // empty')

if [ -z "$TOKEN" ] || [ "$TOKEN" = "null" ]; then
  echo "Failed to get token, trying without authentication..."
  AUTH_HEADER=""
else
  echo "Got token: $TOKEN"
  AUTH_HEADER="Authorization: Bearer $TOKEN"
fi

# Create a category
echo -e "\n3. Creating a category..."
CATEGORY_RESPONSE=$(curl -s -X POST "$BASE_URL/api/categories?user_id=$USER_ID" \
  -H "$CONTENT_TYPE" \
  $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\"") \
  -d '{
    "name": "Work",
    "description": "Work related tasks",
    "color": "#ff5733"
  }')
echo "Category response: $CATEGORY_RESPONSE"

CATEGORY_ID=$(echo $CATEGORY_RESPONSE | jq -r '.id // empty')

# Create tags
echo -e "\n4. Creating tags..."
TAG1_RESPONSE=$(curl -s -X POST "$BASE_URL/api/tags?user_id=$USER_ID" \
  -H "$CONTENT_TYPE" \
  $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\"") \
  -d '{
    "name": "urgent"
  }')
echo "Tag 1 response: $TAG1_RESPONSE"

TAG2_RESPONSE=$(curl -s -X POST "$BASE_URL/api/tags?user_id=$USER_ID" \
  -H "$CONTENT_TYPE" \
  $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\"") \
  -d '{
    "name": "important"
  }')
echo "Tag 2 response: $TAG2_RESPONSE"

# Create todos with new features
echo -e "\n5. Creating todos with categories and tags..."
TODO1_RESPONSE=$(curl -s -X POST "$BASE_URL/api/todos" \
  -H "$CONTENT_TYPE" \
  $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\"") \
  -d "{
    \"title\": \"Complete project proposal\",
    \"description\": \"Write and submit the Q1 project proposal\",
    \"category_id\": \"$CATEGORY_ID\",
    \"priority\": 4,
    \"due_date\": \"$(date -d '+7 days' -u +%Y-%m-%dT%H:%M:%SZ)\",
    \"tags\": [\"urgent\", \"important\"]
  }")
echo "Todo 1 response: $TODO1_RESPONSE"

TODO2_RESPONSE=$(curl -s -X POST "$BASE_URL/api/todos" \
  -H "$CONTENT_TYPE" \
  $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\"") \
  -d "{
    \"title\": \"Review code changes\",
    \"description\": \"Review pull requests from the team\",
    \"category_id\": \"$CATEGORY_ID\",
    \"priority\": 2,
    \"tags\": [\"urgent\"]
  }")
echo "Todo 2 response: $TODO2_RESPONSE"

TODO3_RESPONSE=$(curl -s -X POST "$BASE_URL/api/todos" \
  -H "$CONTENT_TYPE" \
  $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\"") \
  -d '{
    "title": "Buy groceries",
    "description": "Get milk, bread, and eggs",
    "priority": 1
  }')
echo "Todo 3 response: $TODO3_RESPONSE"

# Get todos with filtering
echo -e "\n6. Getting todos with filtering..."
TODOS_RESPONSE=$(curl -s -X GET "$BASE_URL/api/todos?page=1&per_page=10&priority=4" \
  $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\""))
echo "Filtered todos response: $TODOS_RESPONSE"

# Get categories
echo -e "\n7. Getting categories..."
CATEGORIES_RESPONSE=$(curl -s -X GET "$BASE_URL/api/categories?user_id=$USER_ID" \
  $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\""))
echo "Categories response: $CATEGORIES_RESPONSE"

# Get tags
echo -e "\n8. Getting tags..."
TAGS_RESPONSE=$(curl -s -X GET "$BASE_URL/api/tags?user_id=$USER_ID" \
  $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\""))
echo "Tags response: $TAGS_RESPONSE"

# Get statistics
echo -e "\n9. Getting todo statistics..."
STATS_RESPONSE=$(curl -s -X GET "$BASE_URL/api/stats/todos?user_id=$USER_ID" \
  $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\""))
echo "Statistics response: $STATS_RESPONSE"

# Test batch operations
echo -e "\n10. Testing batch operations..."
TODO_IDS=$(echo "$TODO1_RESPONSE $TODO2_RESPONSE $TODO3_RESPONSE" | jq -r '.id' | tr '\n' ',' | sed 's/,$//')

if [ -n "$TODO_IDS" ]; then
  # Convert comma-separated string to JSON array
  TODO_IDS_ARRAY=$(echo "[$TODO_IDS]" | sed 's/\([^,]*\)/"\1"/g')
  
  echo "Todo IDs for batch operation: $TODO_IDS_ARRAY"
  
  BATCH_UPDATE_RESPONSE=$(curl -s -X PATCH "$BASE_URL/api/todos/batch" \
    -H "$CONTENT_TYPE" \
    $([ -n "$AUTH_HEADER" ] && echo "-H \"$AUTH_HEADER\"") \
    -d "{
      \"todo_ids\": $TODO_IDS_ARRAY,
      \"completed\": true
    }")
  echo "Batch update response: $BATCH_UPDATE_RESPONSE"
fi

# Health check
echo -e "\n11. Health check..."
HEALTH_RESPONSE=$(curl -s -X GET "$BASE_URL/health")
echo "Health check response: $HEALTH_RESPONSE"

echo -e "\n=== API Test Complete ==="
