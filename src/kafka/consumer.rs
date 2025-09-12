use crate::kafka::KafkaConfig;
use crate::kafka::{create_consumer_config, DomainEvent, EventEnvelope, KafkaEventError};
use futures::StreamExt;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::message::Message;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, error, info, warn};

pub type EventReceiver = broadcast::Receiver<EventEnvelope>;

#[derive(Clone)]
pub struct EventConsumer {
    consumer: Option<Arc<StreamConsumer>>,
    config: KafkaConfig,
    event_sender: broadcast::Sender<EventEnvelope>,
}

impl EventConsumer {
    pub async fn new(config: KafkaConfig) -> Result<Self, KafkaEventError> {
        let (event_sender, _) = broadcast::channel(1000);

        if !config.enabled {
            info!("Kafka is disabled, event consumer will be a no-op");
            return Ok(Self {
                consumer: None,
                config,
                event_sender,
            });
        }

        let kafka_config = create_consumer_config(&config);
        let consumer: StreamConsumer = match kafka_config.create() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to create Kafka consumer: {:?}. Consumer will be disabled.", e);
                return Ok(Self {
                    consumer: None,
                    config,
                    event_sender,
                });
            }
        };

        // Subscribe to all topics
        let topics = vec![
            format!("{}.users", config.topic_prefix),
            format!("{}.todos", config.topic_prefix),
            format!("{}.categories", config.topic_prefix),
            format!("{}.tags", config.topic_prefix),
        ];

        if let Err(e) = consumer.subscribe(&topics.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
            warn!("Failed to subscribe to Kafka topics: {:?}. Consumer will be disabled.", e);
            return Ok(Self {
                consumer: None,
                config,
                event_sender,
            });
        }

        info!(
            "Kafka consumer initialized and subscribed to topics: {:?}",
            topics
        );

        Ok(Self {
            consumer: Some(Arc::new(consumer)),
            config,
            event_sender,
        })
    }

    pub fn subscribe(&self) -> EventReceiver {
        self.event_sender.subscribe()
    }

    pub async fn start_consuming(&self) -> Result<(), KafkaEventError> {
        let Some(consumer) = &self.consumer else {
            debug!("Kafka disabled, skipping event consumption");
            return Ok(());
        };

        info!("Starting Kafka event consumption...");

        let mut stream = consumer.stream();
        
        while let Some(message) = stream.next().await {
            match message {
                Ok(m) => {
                    if let Err(e) = self.process_message(&m).await {
                        error!("Error processing message: {:?}", e);
                    }
                }
                Err(e) => {
                    // Check if it's a broker transport failure - these are expected when Kafka is down
                    if let rdkafka::error::KafkaError::MessageConsumption(rdkafka::error::RDKafkaErrorCode::BrokerTransportFailure) = e {
                        debug!("Kafka broker unavailable, will retry when available");
                        // Sleep briefly to avoid tight loop
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    } else {
                        error!("Error receiving message: {:?}", e);
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_message(&self, message: &rdkafka::message::BorrowedMessage<'_>) -> Result<(), KafkaEventError> {
        let payload = match message.payload() {
            Some(p) => p,
            None => {
                warn!("Received message with no payload");
                return Ok(());
            }
        };

        let payload_str = std::str::from_utf8(payload)
            .map_err(|e| KafkaEventError::ConsumerError(e.to_string()))?;

        let envelope: EventEnvelope = serde_json::from_str(payload_str)?;

        debug!(
            "Received event from topic '{}', partition {}, offset {}: {:?}",
            message.topic(),
            message.partition(),
            message.offset(),
            envelope.event
        );

        // Process the event based on its type
        self.handle_event(&envelope).await?;

        // Broadcast the event to subscribers
        if let Err(e) = self.event_sender.send(envelope) {
            warn!("No active event subscribers: {:?}", e);
        }

        Ok(())
    }

    async fn handle_event(&self, envelope: &EventEnvelope) -> Result<(), KafkaEventError> {
        match &envelope.event {
            DomainEvent::UserRegistered(event) => {
                info!("User registered: {} ({})", event.username, event.user_id);
                // Add custom processing logic here (e.g., send welcome email)
            }
            DomainEvent::UserLoggedIn(event) => {
                debug!("User logged in: {} ({})", event.username, event.user_id);
                // Add custom processing logic here (e.g., update last login)
            }
            DomainEvent::TodoCreated(event) => {
                info!("Todo created: '{}' for user {}", event.title, event.user_id);
                // Add custom processing logic here (e.g., send notifications)
            }
            DomainEvent::TodoCompleted(event) => {
                info!("Todo completed: {}", event.todo_id);
                // Add custom processing logic here (e.g., achievement tracking)
            }
            DomainEvent::TodoDeleted(event) => {
                info!("Todo deleted: {}", event.todo_id);
                // Add custom processing logic here (e.g., cleanup related data)
            }
            DomainEvent::TodosDeletedBatch(event) => {
                info!("Batch deleted {} todos", event.deleted_count);
                // Add custom processing logic here
            }
            DomainEvent::TodosUpdatedBatch(event) => {
                info!("Batch updated {} todos", event.updated_count);
                // Add custom processing logic here
            }
            DomainEvent::CategoryCreated(event) => {
                info!("Category created: '{}' for user {}", event.name, event.user_id);
                // Add custom processing logic here
            }
            DomainEvent::TagCreated(event) => {
                info!("Tag created: '{}' for user {}", event.name, event.user_id);
                // Add custom processing logic here
            }
            _ => {
                debug!("Received event: {:?}", envelope.event);
                // Handle other event types
            }
        }

        Ok(())
    }

    pub fn is_enabled(&self) -> bool {
        self.consumer.is_some()
    }

    pub fn get_config(&self) -> &KafkaConfig {
        &self.config
    }
}

/// Background task to run the event consumer
pub async fn run_event_consumer(consumer: EventConsumer) {
    if !consumer.is_enabled() {
        info!("Kafka consumer is disabled, not starting background task");
        return;
    }

    tokio::spawn(async move {
        if let Err(e) = consumer.start_consuming().await {
            error!("Event consumer error: {:?}", e);
        }
    });

    info!("Event consumer background task started");
}
