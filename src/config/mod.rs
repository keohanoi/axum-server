use serde::Deserialize;
use std::env;
use crate::kafka::KafkaConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    pub rust_log: String,
    pub kafka: KafkaConfig,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        dotenvy::dotenv().ok();

        let bootstrap_servers = env::var("KAFKA_BOOTSTRAP_SERVERS")
            .unwrap_or_else(|_| "localhost:9092".to_string());
        
        let kafka_config = KafkaConfig {
            bootstrap_servers: bootstrap_servers.clone(),
            client_id: env::var("KAFKA_CLIENT_ID")
                .unwrap_or_else(|_| "axum-server".to_string()),
            group_id: env::var("KAFKA_GROUP_ID")
                .unwrap_or_else(|_| "axum-server-group".to_string()),
            todo_events_topic: env::var("KAFKA_TODO_EVENTS_TOPIC")
                .unwrap_or_else(|_| "todo-events".to_string()),
            user_events_topic: env::var("KAFKA_USER_EVENTS_TOPIC")
                .unwrap_or_else(|_| "user-events".to_string()),
            category_events_topic: env::var("KAFKA_CATEGORY_EVENTS_TOPIC")
                .unwrap_or_else(|_| "category-events".to_string()),
            tag_events_topic: env::var("KAFKA_TAG_EVENTS_TOPIC")
                .unwrap_or_else(|_| "tag-events".to_string()),
            enable_auto_commit: env::var("KAFKA_ENABLE_AUTO_COMMIT")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            session_timeout_ms: env::var("KAFKA_SESSION_TIMEOUT_MS")
                .unwrap_or_else(|_| "6000".to_string())
                .parse()
                .unwrap_or(6000),
            auto_offset_reset: env::var("KAFKA_AUTO_OFFSET_RESET")
                .unwrap_or_else(|_| "earliest".to_string()),
            enabled: env::var("KAFKA_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            brokers: bootstrap_servers,
            topic_prefix: env::var("KAFKA_TOPIC_PREFIX")
                .unwrap_or_else(|_| "axum-server".to_string()),
            producer_timeout_ms: env::var("KAFKA_PRODUCER_TIMEOUT_MS")
                .unwrap_or_else(|_| "5000".to_string())
                .parse()
                .unwrap_or(5000),
        };

        Ok(Config {
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/todos".to_string()),
            server_host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            server_port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            kafka: kafka_config,
        })
    }

    pub fn server_address(&self) -> String {
        format!("{}:{}", self.server_host, self.server_port)
    }
}