use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct KafkaConfig {
    pub bootstrap_servers: String,
    pub client_id: String,
    pub group_id: String,
    pub todo_events_topic: String,
    pub user_events_topic: String,
    pub category_events_topic: String,
    pub tag_events_topic: String,
    pub enable_auto_commit: bool,
    pub session_timeout_ms: u64,
    pub auto_offset_reset: String,
    pub enabled: bool,
    pub brokers: String,
    pub topic_prefix: String,
    pub producer_timeout_ms: u64,
}

impl Default for KafkaConfig {
    fn default() -> Self {
        Self {
            bootstrap_servers: "localhost:9092".to_string(),
            client_id: "axum-server".to_string(),
            group_id: "axum-server-group".to_string(),
            todo_events_topic: "todo-events".to_string(),
            user_events_topic: "user-events".to_string(),
            category_events_topic: "category-events".to_string(),
            tag_events_topic: "tag-events".to_string(),
            enable_auto_commit: true,
            session_timeout_ms: 6000,
            auto_offset_reset: "earliest".to_string(),
            enabled: true,
            brokers: "localhost:9092".to_string(),
            topic_prefix: "axum-server".to_string(),
            producer_timeout_ms: 5000,
        }
    }
}