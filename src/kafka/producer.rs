use crate::kafka::KafkaConfig;
use crate::kafka::{create_kafka_config, DomainEvent, EventEnvelope, KafkaEventError};
use rdkafka::producer::{FutureProducer, FutureRecord};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};
use uuid::Uuid;

#[derive(Clone)]
pub struct EventProducer {
    producer: Option<Arc<FutureProducer>>,
    config: KafkaConfig,
}

impl EventProducer {
    pub async fn new(config: KafkaConfig) -> Result<Self, KafkaEventError> {
        if !config.enabled {
            info!("Kafka is disabled, event producer will be a no-op");
            return Ok(Self {
                producer: None,
                config,
            });
        }

        let kafka_config = create_kafka_config(&config);
        let producer: FutureProducer = kafka_config.create()?;

        info!(
            "Kafka producer initialized with brokers: {}",
            config.brokers
        );

        Ok(Self {
            producer: Some(Arc::new(producer)),
            config,
        })
    }

    pub async fn publish_event(
        &self,
        event: DomainEvent,
        user_id: Option<Uuid>,
    ) -> Result<(), KafkaEventError> {
        let Some(producer) = &self.producer else {
            debug!("Kafka disabled, skipping event publication");
            return Ok(());
        };

        let envelope = EventEnvelope::new(event, user_id);
        let topic = self.get_topic_for_event(&envelope.event);
        let key = self.get_key_for_event(&envelope.event);
        let payload = serde_json::to_string(&envelope)?;

        debug!(
            "Publishing event to topic '{}' with key '{}': {:?}",
            topic, key, envelope.event
        );

        let record = FutureRecord::to(&topic)
            .key(&key)
            .payload(&payload)
            .headers(self.create_headers(&envelope));

        match producer
            .send(
                record,
                Duration::from_millis(self.config.producer_timeout_ms),
            )
            .await
        {
            Ok((partition, offset)) => {
                debug!(
                    "Event published successfully to partition {} at offset {}",
                    partition, offset
                );
                Ok(())
            }
            Err((kafka_error, _)) => {
                error!("Failed to publish event: {:?}", kafka_error);
                Err(KafkaEventError::ProducerError(kafka_error))
            }
        }
    }

    fn get_topic_for_event(&self, event: &DomainEvent) -> String {
        let topic_suffix = match event {
            DomainEvent::UserRegistered(_) | DomainEvent::UserLoggedIn(_) => "users",
            DomainEvent::TodoCreated(_)
            | DomainEvent::TodoUpdated(_)
            | DomainEvent::TodoCompleted(_)
            | DomainEvent::TodoDeleted(_)
            | DomainEvent::TodosDeletedBatch(_)
            | DomainEvent::TodosUpdatedBatch(_) => "todos",
            DomainEvent::CategoryCreated(_)
            | DomainEvent::CategoryUpdated(_)
            | DomainEvent::CategoryDeleted(_) => "categories",
            DomainEvent::TagCreated(_) | DomainEvent::TagUpdated(_) | DomainEvent::TagDeleted(_) => {
                "tags"
            }
        };
        format!("{}.{}", self.config.topic_prefix, topic_suffix)
    }

    fn get_key_for_event(&self, event: &DomainEvent) -> String {
        match event {
            DomainEvent::UserRegistered(e) => format!("user.{}", e.user_id),
            DomainEvent::UserLoggedIn(e) => format!("user.{}", e.user_id),
            DomainEvent::TodoCreated(e) => format!("todo.{}", e.todo_id),
            DomainEvent::TodoUpdated(e) => format!("todo.{}", e.todo_id),
            DomainEvent::TodoCompleted(e) => format!("todo.{}", e.todo_id),
            DomainEvent::TodoDeleted(e) => format!("todo.{}", e.todo_id),
            DomainEvent::TodosDeletedBatch(_) => "batch.delete".to_string(),
            DomainEvent::TodosUpdatedBatch(_) => "batch.update".to_string(),
            DomainEvent::CategoryCreated(e) => format!("category.{}", e.category_id),
            DomainEvent::CategoryUpdated(e) => format!("category.{}", e.category_id),
            DomainEvent::CategoryDeleted(e) => format!("category.{}", e.category_id),
            DomainEvent::TagCreated(e) => format!("tag.{}", e.tag_id),
            DomainEvent::TagUpdated(e) => format!("tag.{}", e.tag_id),
            DomainEvent::TagDeleted(e) => format!("tag.{}", e.tag_id),
        }
    }

    fn create_headers(&self, envelope: &EventEnvelope) -> rdkafka::message::OwnedHeaders {
        let mut headers = rdkafka::message::OwnedHeaders::new();
        
        headers = headers.insert(rdkafka::message::Header {
            key: "event_id",
            value: Some(&envelope.metadata.event_id.to_string()),
        });

        headers = headers.insert(rdkafka::message::Header {
            key: "timestamp",
            value: Some(&envelope.metadata.timestamp.to_rfc3339()),
        });

        if let Some(user_id) = envelope.metadata.user_id {
            headers = headers.insert(rdkafka::message::Header {
                key: "user_id",
                value: Some(&user_id.to_string()),
            });
        }

        if let Some(correlation_id) = &envelope.metadata.correlation_id {
            headers = headers.insert(rdkafka::message::Header {
                key: "correlation_id",
                value: Some(correlation_id.as_str()),
            });
        }

        headers = headers.insert(rdkafka::message::Header {
            key: "content_type",
            value: Some("application/json"),
        });

        headers
    }

    pub fn is_enabled(&self) -> bool {
        self.producer.is_some()
    }
}

// Convenience methods for publishing specific event types
impl EventProducer {
    pub async fn publish_user_registered(&self, event: crate::kafka::UserRegisteredEvent) -> Result<(), KafkaEventError> {
        let user_id = event.user_id;
        self.publish_event(DomainEvent::UserRegistered(event), Some(user_id))
            .await
    }

    pub async fn publish_todo_created(&self, event: crate::kafka::TodoCreatedEvent) -> Result<(), KafkaEventError> {
        let user_id = event.user_id;
        self.publish_event(DomainEvent::TodoCreated(event), Some(user_id))
            .await
    }

    pub async fn publish_todo_updated(&self, event: crate::kafka::TodoUpdatedEvent, user_id: Uuid) -> Result<(), KafkaEventError> {
        self.publish_event(DomainEvent::TodoUpdated(event), Some(user_id))
            .await
    }

    pub async fn publish_todo_completed(&self, event: crate::kafka::TodoCompletedEvent, user_id: Uuid) -> Result<(), KafkaEventError> {
        self.publish_event(DomainEvent::TodoCompleted(event), Some(user_id))
            .await
    }

    pub async fn publish_todo_deleted(&self, event: crate::kafka::TodoDeletedEvent, user_id: Uuid) -> Result<(), KafkaEventError> {
        self.publish_event(DomainEvent::TodoDeleted(event), Some(user_id))
            .await
    }
}
