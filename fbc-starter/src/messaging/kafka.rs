// Kafka 消息代理实现（分离 Producer 和 Consumer）

use super::{Message, MessageConsumer, MessageProducer};
use async_trait::async_trait;
use rdkafka::admin::{AdminClient, AdminOptions, NewTopic, TopicReplication};
use rdkafka::config::ClientConfig;
use rdkafka::consumer::base_consumer::BaseConsumer;
use rdkafka::consumer::stream_consumer::StreamConsumer;
use rdkafka::consumer::{CommitMode, Consumer, ConsumerContext, Rebalance};
use rdkafka::error::KafkaError;
use rdkafka::message::Message as RdKafkaMessage;
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::{ClientContext, TopicPartitionList};
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

// 导出 config 中的配置类型，避免重复定义
pub use crate::config::{KafkaConsumerConfig, KafkaProducerConfig};

/// Kafka 消费者上下文
struct KafkaConsumerContext;

impl ClientContext for KafkaConsumerContext {}

impl ConsumerContext for KafkaConsumerContext {
    fn pre_rebalance(&self, _consumer: &BaseConsumer<KafkaConsumerContext>, rebalance: &Rebalance) {
        info!("Kafka rebalance: {:?}", rebalance);
    }

    fn post_rebalance(
        &self,
        _consumer: &BaseConsumer<KafkaConsumerContext>,
        rebalance: &Rebalance,
    ) {
        info!("Kafka rebalance completed: {:?}", rebalance);
    }

    fn commit_callback(
        &self,
        result: rdkafka::error::KafkaResult<()>,
        _offsets: &TopicPartitionList,
    ) {
        if let Err(e) = result {
            warn!("Kafka commit error: {}", e);
        }
    }
}

/// Kafka 生产者实现
pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    /// 创建新的 Kafka 生产者
    ///
    /// # 参数
    /// - `brokers`: Kafka 集群地址
    /// - `config`: 生产者配置
    pub fn new(brokers: &str, config: &KafkaProducerConfig) -> Result<Self, KafkaError> {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("acks", &config.acks)
            .set("retries", config.retries.to_string())
            .set("enable.idempotence", config.enable_idempotence.to_string())
            .set("compression.type", "snappy")
            .set("linger.ms", "10") // 批量发送延迟
            .set("batch.size", "32768") // 批量大小
            .create()?;

        info!("✅ Kafka Producer 创建成功");
        Ok(Self { producer })
    }
}

#[async_trait]
impl MessageProducer for KafkaProducer {
    async fn publish(
        &self,
        topic: &str,
        message: Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::to_vec(&message)?;
        let key = message.topic.as_bytes();

        let record = FutureRecord::to(topic).key(key).payload(&payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        Ok(())
    }
}

/// Kafka 消费者实现
pub struct KafkaConsumer {
    consumer: Arc<StreamConsumer<KafkaConsumerContext>>,
}

impl KafkaConsumer {
    /// 使用指定的 group_id 创建新的 Kafka 消费者
    ///
    /// # 参数
    /// - `brokers`: Kafka 集群地址
    /// - `config`: 消费者配置（用于其他配置项）
    /// - `group_id`: 消费者组ID
    pub fn new_with_group_id(
        brokers: &str,
        config: &KafkaConsumerConfig,
        group_id: &str,
    ) -> Result<Self, KafkaError> {
        let consumer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("group.id", group_id)
            .set("enable.auto.commit", config.enable_auto_commit.to_string())
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "30000")
            .set("heartbeat.interval.ms", "10000")
            .create_with_context(KafkaConsumerContext)?;

        info!("✅ Kafka Consumer 创建成功 (group: {})", group_id);
        Ok(Self {
            consumer: Arc::new(consumer),
        })
    }
}

#[async_trait]
impl MessageConsumer for KafkaConsumer {
    async fn subscribe(
        &self,
        topic: &str,
        handler: Arc<dyn Fn(Message) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.subscribe_topics(vec![topic.to_string()], handler)
            .await
    }

    async fn subscribe_topics(
        &self,
        topics: Vec<String>,
        handler: Arc<dyn Fn(Message) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let consumer = self.consumer.clone();
        let topics_str: Vec<&str> = topics.iter().map(|s| s.as_str()).collect();

        // 订阅 topics
        consumer.subscribe(&topics_str)?;
        info!("✅ Kafka Consumer 开始订阅 topics: {:?}", topics);

        // 启动消费循环
        tokio::spawn(async move {
            loop {
                match consumer.recv().await {
                    Ok(kafka_message) => {
                        if let Some(payload) = kafka_message.payload() {
                            let message = match serde_json::from_slice::<Message>(payload) {
                                Ok(m) => Some(m),
                                Err(e) => {
                                    // 回退：如果不是标准的 fbc-starter Message，尝试将其作为普通 JSON 解析（比如 MinIO 原生 Webhook 消息）
                                    match serde_json::from_slice::<serde_json::Value>(payload) {
                                        Ok(raw_json) => {
                                            Some(Message {
                                                topic: kafka_message.topic().to_string(),
                                                from: "external".to_string(),
                                                data: raw_json,
                                                timestamp: chrono::Utc::now().timestamp_millis(),
                                            })
                                        }
                                        Err(je) => {
                                            error!(
                                                "Failed to deserialize message: {} \n Raw JSON fallback also failed: {}", 
                                                e, je
                                            );
                                            None
                                        }
                                    }
                                }
                            };

                            if let Some(message) = message {
                                handler(message);
                                // 手动提交偏移量
                                if let Err(e) =
                                    consumer.store_offset_from_message(&kafka_message)
                                {
                                    error!("Failed to store offset: {}", e);
                                }
                                if let Err(e) = consumer.commit_consumer_state(CommitMode::Sync)
                                {
                                    error!("Failed to commit offset: {}", e);
                                }
                            }
                        } else {
                            warn!("Received empty Kafka message");
                        }
                    }
                    Err(e) => {
                        error!("Kafka consumer error: {}", e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }
        });

        Ok(())
    }
}

/// 根据 handlers 创建多个按 group_id 分组的消费者
///
/// # 参数
/// - `brokers`: Kafka 集群地址
/// - `config`: 消费者配置
/// - `handlers`: Kafka 消息处理器列表
///
/// # 返回
/// 返回一个映射：group_id -> (consumer, topics)
/// 会为每个不同的 group_id 创建独立的 consumer
#[cfg(feature = "consumer")]
pub fn create_consumers_from_handlers(
    brokers: &str,
    config: &KafkaConsumerConfig,
    handlers: &[std::sync::Arc<dyn crate::messaging::KafkaMessageHandler>],
) -> Result<std::collections::HashMap<String, (KafkaConsumer, Vec<String>)>, KafkaError> {
    use std::collections::HashMap;

    let mut result: HashMap<String, (KafkaConsumer, Vec<String>)> = HashMap::new();
    let mut group_topics: HashMap<String, Vec<String>> = HashMap::new();

    // 如果没有 handlers，返回空结果
    if handlers.is_empty() {
        return Ok(result);
    }

    // 根据 handlers 的 group_id，将 topics 分组
    for handler in handlers {
        let group_id = handler.group_id();
        for topic in handler.topics() {
            group_topics
                .entry(group_id.clone())
                .or_insert_with(Vec::new)
                .push(topic);
        }
    }

    // 为每个 group_id 创建独立的 consumer
    for (group_id, topics) in group_topics {
        if topics.is_empty() {
            continue;
        }
        let consumer = KafkaConsumer::new_with_group_id(brokers, config, &group_id)?;
        result.insert(group_id, (consumer, topics));
    }

    Ok(result)
}

/// 确保 Topic 存在的辅助函数
pub async fn ensure_topic_exists(
    brokers: &str,
    topic: &str,
    partitions: i32,
    replication_factor: i32,
) -> Result<(), KafkaError> {
    let admin_client: AdminClient<_> = ClientConfig::new()
        .set("bootstrap.servers", brokers)
        .create()?;

    let new_topic = NewTopic::new(
        topic,
        partitions,
        TopicReplication::Fixed(replication_factor),
    );

    let results = admin_client
        .create_topics(&[new_topic], &AdminOptions::default())
        .await?;

    for result in results {
        match result {
            Ok(topic_name) => {
                info!("✅ Kafka Topic 创建成功: {}", topic_name);
            }
            Err((topic_name, err)) => {
                // 如果 topic 已存在，忽略错误
                if err.to_string().contains("already exists") {
                    info!("ℹ️ Kafka Topic 已存在: {}", topic_name);
                } else {
                    warn!("⚠️ Kafka Topic 创建失败: {} - {}", topic_name, err);
                }
            }
        }
    }

    Ok(())
}
