#!/bin/bash

# Script to create a todo via the API

# Default values
SERVER_URL="http://127.0.0.1:3000"
TITLE=""
DESCRIPTION=""

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -t|--title)
            TITLE="$2"
            shift 2
            ;;
        -d|--description)
            DESCRIPTION="$2"
            shift 2
            ;;
        -u|--url)
            SERVER_URL="$2"
            shift 2
            ;;
        -h|--help)
            echo "Usage: $0 -t 'Todo Title' [-d 'Description'] [-u 'http://server:port']"
            echo "Options:"
            echo "  -t, --title        Todo title (required)"
            echo "  -d, --description  Todo description (optional)"
            echo "  -u, --url          Server URL (default: http://127.0.0.1:3000)"
            echo "  -h, --help         Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option $1"
            exit 1
            ;;
    esac
done

# Check if title is provided
if [ -z "$TITLE" ]; then
    echo "Error: Title is required"
    echo "Usage: $0 -t 'Todo Title' [-d 'Description']"
    exit 1
fi

# Build JSON payload
if [ -z "$DESCRIPTION" ]; then
    JSON_PAYLOAD="{\"title\": \"$TITLE\"}"
else
    JSON_PAYLOAD="{\"title\": \"$TITLE\", \"description\": \"$DESCRIPTION\"}"
fi

echo "Creating todo..."
echo "Title: $TITLE"
if [ ! -z "$DESCRIPTION" ]; then
    echo "Description: $DESCRIPTION"
fi
echo "Server: $SERVER_URL"
echo ""

# Make the API request
response=$(curl -s -w "\nHTTP_CODE:%{http_code}" \
    -X POST \
    -H "Content-Type: application/json" \
    -d "$JSON_PAYLOAD" \
    "$SERVER_URL/api/todos")

# Extract HTTP status code and response body
http_code=$(echo "$response" | tail -n1 | sed 's/.*HTTP_CODE://')
response_body=$(echo "$response" | sed '$d')

# Check if request was successful
if [ "$http_code" -eq 201 ]; then
    echo "✅ Todo created successfully!"
    echo "$response_body" | jq . 2>/dev/null || echo "$response_body"
else
    echo "❌ Failed to create todo (HTTP $http_code)"
    echo "$response_body" | jq . 2>/dev/null || echo "$response_body"
fi