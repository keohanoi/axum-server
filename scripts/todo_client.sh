#!/bin/bash

# Comprehensive todo client script

SERVER_URL="http://127.0.0.1:3000"

show_help() {
    echo "Todo Client - Interact with the Todo API"
    echo ""
    echo "Usage: $0 <command> [options]"
    echo ""
    echo "Commands:"
    echo "  create -t 'title' [-d 'description']  Create a new todo"
    echo "  list [-p page] [-l limit]             List todos with pagination"
    echo "  get <id>                              Get a specific todo"
    echo "  update <id> [options]                 Update a todo"
    echo "  delete <id>                           Delete a todo"
    echo "  health                                Check server health"
    echo ""
    echo "Options:"
    echo "  -t, --title        Todo title"
    echo "  -d, --description  Todo description"
    echo "  -c, --completed    Set completed status (true/false)"
    echo "  -p, --page         Page number (default: 1)"
    echo "  -l, --limit        Items per page (default: 10)"
    echo "  -s, --search       Search term"
    echo "  -f, --filter       Filter by completed status (true/false)"
    echo "  -u, --url          Server URL (default: http://127.0.0.1:3000)"
    echo ""
    echo "Examples:"
    echo "  $0 create -t 'Buy groceries' -d 'Milk, bread, eggs'"
    echo "  $0 list -p 1 -l 5"
    echo "  $0 get 123e4567-e89b-12d3-a456-426614174000"
    echo "  $0 update 123e4567-e89b-12d3-a456-426614174000 -c true"
    echo "  $0 delete 123e4567-e89b-12d3-a456-426614174000"
}

make_request() {
    local method="$1"
    local endpoint="$2"
    local data="$3"
    
    if [ -z "$data" ]; then
        response=$(curl -s -w "\nHTTP_CODE:%{http_code}" -X "$method" "$SERVER_URL$endpoint")
    else
        response=$(curl -s -w "\nHTTP_CODE:%{http_code}" \
            -X "$method" \
            -H "Content-Type: application/json" \
            -d "$data" \
            "$SERVER_URL$endpoint")
    fi
    
    http_code=$(echo "$response" | tail -n1 | sed 's/.*HTTP_CODE://')
    response_body=$(echo "$response" | sed '$d')
    
    echo "$response_body" | jq . 2>/dev/null || echo "$response_body"
    
    if [ "$http_code" -ge 400 ]; then
        echo "HTTP Error: $http_code" >&2
        return 1
    fi
}

create_todo() {
    local title=""
    local description=""
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -t|--title) title="$2"; shift 2 ;;
            -d|--description) description="$2"; shift 2 ;;
            *) echo "Unknown option: $1" >&2; return 1 ;;
        esac
    done
    
    if [ -z "$title" ]; then
        echo "Error: Title is required for creating a todo" >&2
        return 1
    fi
    
    local json="{\"title\": \"$title\""
    if [ ! -z "$description" ]; then
        json="$json, \"description\": \"$description\""
    fi
    json="$json}"
    
    echo "Creating todo..."
    make_request "POST" "/api/todos" "$json"
}

list_todos() {
    local page=1
    local per_page=10
    local search=""
    local completed=""
    local query_params=""
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -p|--page) page="$2"; shift 2 ;;
            -l|--limit) per_page="$2"; shift 2 ;;
            -s|--search) search="$2"; shift 2 ;;
            -f|--filter) completed="$2"; shift 2 ;;
            *) echo "Unknown option: $1" >&2; return 1 ;;
        esac
    done
    
    query_params="?page=$page&per_page=$per_page"
    if [ ! -z "$search" ]; then
        query_params="$query_params&search=$search"
    fi
    if [ ! -z "$completed" ]; then
        query_params="$query_params&completed=$completed"
    fi
    
    echo "Listing todos..."
    make_request "GET" "/api/todos$query_params"
}

get_todo() {
    local id="$1"
    if [ -z "$id" ]; then
        echo "Error: Todo ID is required" >&2
        return 1
    fi
    
    echo "Getting todo $id..."
    make_request "GET" "/api/todos/$id"
}

update_todo() {
    local id="$1"
    shift
    
    if [ -z "$id" ]; then
        echo "Error: Todo ID is required" >&2
        return 1
    fi
    
    local title=""
    local description=""
    local completed=""
    local updates=""
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            -t|--title) title="$2"; shift 2 ;;
            -d|--description) description="$2"; shift 2 ;;
            -c|--completed) completed="$2"; shift 2 ;;
            *) echo "Unknown option: $1" >&2; return 1 ;;
        esac
    done
    
    local json="{"
    local first=true
    
    if [ ! -z "$title" ]; then
        json="$json\"title\": \"$title\""
        first=false
    fi
    
    if [ ! -z "$description" ]; then
        [ "$first" = false ] && json="$json, "
        json="$json\"description\": \"$description\""
        first=false
    fi
    
    if [ ! -z "$completed" ]; then
        [ "$first" = false ] && json="$json, "
        json="$json\"completed\": $completed"
        first=false
    fi
    
    json="$json}"
    
    if [ "$json" = "{}" ]; then
        echo "Error: No updates provided" >&2
        return 1
    fi
    
    echo "Updating todo $id..."
    make_request "PATCH" "/api/todos/$id" "$json"
}

delete_todo() {
    local id="$1"
    if [ -z "$id" ]; then
        echo "Error: Todo ID is required" >&2
        return 1
    fi
    
    echo "Deleting todo $id..."
    make_request "DELETE" "/api/todos/$id"
    echo "Todo deleted successfully"
}

check_health() {
    echo "Checking server health..."
    make_request "GET" "/health"
}

# Parse global options
while [[ $# -gt 0 ]] && [[ $1 == -* ]]; do
    case $1 in
        -u|--url) SERVER_URL="$2"; shift 2 ;;
        -h|--help) show_help; exit 0 ;;
        *) break ;;
    esac
done

# Get command
command="$1"
shift

case $command in
    create) create_todo "$@" ;;
    list) list_todos "$@" ;;
    get) get_todo "$@" ;;
    update) update_todo "$@" ;;
    delete) delete_todo "$@" ;;
    health) check_health ;;
    *) 
        echo "Unknown command: $command" >&2
        show_help
        exit 1
        ;;
esac