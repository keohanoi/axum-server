use axum::{
    extract::{Query, State},
    Json,
};
use chrono::Utc;
use uuid::Uuid;

use crate::{
    db::DbPool,
    error::Result,
    models::{
        TodoStatsResponse, PriorityCount, CategoryCount,
    },
};

#[derive(serde::Deserialize)]
pub struct StatsQuery {
    pub user_id: Option<Uuid>,
}

pub async fn get_todo_statistics(
    State(pool): State<DbPool>,
    Query(query): Query<StatsQuery>,
) -> Result<Json<TodoStatsResponse>> {
    let user_filter = if let Some(user_id) = query.user_id {
        format!("WHERE user_id = '{}'", user_id)
    } else {
        String::new()
    };

    // Get basic counts
    let total_todos: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM todos {}",
        user_filter
    ))
    .fetch_one(&pool)
    .await?;

    let completed_filter = if user_filter.is_empty() {
        "WHERE".to_string()
    } else {
        format!("{} AND", user_filter)
    };

    let completed_todos: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM todos {} completed = true",
        completed_filter
    ))
    .fetch_one(&pool)
    .await?;

    let pending_todos = total_todos - completed_todos;

    let overdue_filter = if user_filter.is_empty() {
        "WHERE".to_string()
    } else {
        format!("{} AND", user_filter)
    };

    let overdue_todos: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM todos {} due_date < $1 AND completed = false",
        overdue_filter
    ))
    .bind(Utc::now())
    .fetch_one(&pool)
    .await?;

    // Get todos by priority
    let priority_query = format!(
        "SELECT priority, COUNT(*) as count FROM todos {} GROUP BY priority ORDER BY priority",
        user_filter
    );
    
    let priority_rows: Vec<(Option<i32>, i64)> = sqlx::query_as(&priority_query)
        .fetch_all(&pool)
        .await?;

    let todos_by_priority: Vec<PriorityCount> = priority_rows
        .into_iter()
        .map(|(priority, count)| PriorityCount {
            priority: priority.unwrap_or(0),
            count,
        })
        .collect();

    // Get todos by category
    let category_query = format!(
        r#"
        SELECT 
            t.category_id, 
            c.name as category_name, 
            COUNT(*) as count
        FROM todos t
        LEFT JOIN categories c ON t.category_id = c.id
        {}
        GROUP BY t.category_id, c.name
        ORDER BY count DESC
        "#,
        user_filter
    );
    
    let category_rows: Vec<(Option<Uuid>, Option<String>, i64)> = sqlx::query_as(&category_query)
        .fetch_all(&pool)
        .await?;

    let todos_by_category: Vec<CategoryCount> = category_rows
        .into_iter()
        .map(|(category_id, category_name, count)| CategoryCount {
            category_id,
            category_name,
            count,
        })
        .collect();

    let stats = TodoStatsResponse {
        total_todos,
        completed_todos,
        pending_todos,
        overdue_todos,
        todos_by_priority,
        todos_by_category,
    };

    Ok(Json(stats))
}
