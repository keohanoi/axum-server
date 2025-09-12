pub mod config;
pub mod events;
pub mod producer;
pub mod consumer;

use rdkafka::config::ClientConfig;
use thiserror::Error;

pub use config::KafkaConfig;
pub use events::*;
pub use producer::EventProducer;
pub use consumer::{EventConsumer, EventReceiver};

#[derive(Debug, Error)]
pub enum KafkaEventError {
    #[error("Kafka producer error: {0}")]
    ProducerError(#[from] rdkafka::error::KafkaError),
    
    #[error("Kafka consumer error: {0}")]
    ConsumerError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// Create Kafka producer configuration
pub fn create_kafka_config(config: &KafkaConfig) -> ClientConfig {
    let mut client_config = ClientConfig::new();
    
    client_config
        .set("bootstrap.servers", &config.brokers)
        .set("client.id", &config.client_id)
        .set("message.timeout.ms", config.producer_timeout_ms.to_string())
        .set("batch.size", "16384")
        .set("linger.ms", "10")
        .set("compression.type", "snappy");
    
    client_config
}

/// Create Kafka consumer configuration  
pub fn create_consumer_config(config: &KafkaConfig) -> ClientConfig {
    let mut client_config = ClientConfig::new();
    
    client_config
        .set("bootstrap.servers", &config.brokers)
        .set("group.id", &config.group_id)
        .set("client.id", &config.client_id)
        .set("auto.offset.reset", &config.auto_offset_reset)
        .set("session.timeout.ms", config.session_timeout_ms.to_string())
        .set("enable.auto.commit", config.enable_auto_commit.to_string())
        .set("auto.commit.interval.ms", "1000");
    
    client_config
}