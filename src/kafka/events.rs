use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub event_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub user_id: Option<Uuid>,
    pub correlation_id: Option<String>,
}

impl EventMetadata {
    pub fn new(user_id: Option<Uuid>) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            user_id,
            correlation_id: None,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: String) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", content = "data")]
pub enum DomainEvent {
    // User Events
    UserRegistered(UserRegisteredEvent),
    UserLoggedIn(UserLoggedInEvent),
    
    // Todo Events
    TodoCreated(TodoCreatedEvent),
    TodoUpdated(TodoUpdatedEvent),
    TodoCompleted(TodoCompletedEvent),
    TodoDeleted(TodoDeletedEvent),
    TodosDeletedBatch(TodosDeletedBatchEvent),
    TodosUpdatedBatch(TodosUpdatedBatchEvent),
    
    // Category Events
    CategoryCreated(CategoryCreatedEvent),
    CategoryUpdated(CategoryUpdatedEvent),
    CategoryDeleted(CategoryDeletedEvent),
    
    // Tag Events
    TagCreated(TagCreatedEvent),
    TagUpdated(TagUpdatedEvent),
    TagDeleted(TagDeletedEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub metadata: EventMetadata,
    pub event: DomainEvent,
}

impl EventEnvelope {
    pub fn new(event: DomainEvent, user_id: Option<Uuid>) -> Self {
        Self {
            metadata: EventMetadata::new(user_id),
            event,
        }
    }
}

// User Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegisteredEvent {
    pub user_id: Uuid,
    pub username: String,
    pub email: String,
    pub full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLoggedInEvent {
    pub user_id: Uuid,
    pub username: String,
    pub login_timestamp: DateTime<Utc>,
}

// Todo Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoCreatedEvent {
    pub todo_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub user_id: Uuid,
    pub category_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoUpdatedEvent {
    pub todo_id: Uuid,
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub category_id: Option<Uuid>,
    pub priority: Option<i32>,
    pub due_date: Option<DateTime<Utc>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoCompletedEvent {
    pub todo_id: Uuid,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoDeletedEvent {
    pub todo_id: Uuid,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodosDeletedBatchEvent {
    pub todo_ids: Vec<Uuid>,
    pub deleted_count: usize,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodosUpdatedBatchEvent {
    pub todo_ids: Vec<Uuid>,
    pub updated_count: usize,
    pub updated_at: DateTime<Utc>,
    pub changes: TodoUpdatedEvent, // What was changed
}

// Category Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryCreatedEvent {
    pub category_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryUpdatedEvent {
    pub category_id: Uuid,
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryDeletedEvent {
    pub category_id: Uuid,
    pub deleted_at: DateTime<Utc>,
}

// Tag Events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagCreatedEvent {
    pub tag_id: Uuid,
    pub name: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagUpdatedEvent {
    pub tag_id: Uuid,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDeletedEvent {
    pub tag_id: Uuid,
    pub deleted_at: DateTime<Utc>,
}
