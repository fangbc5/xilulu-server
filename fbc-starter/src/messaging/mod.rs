use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// 子模块
#[cfg(feature = "kafka")]
pub mod kafka;

#[cfg(feature = "consumer")]
mod router;

// 导出具体实现
#[cfg(feature = "kafka")]
pub use kafka::{KafkaConsumer, KafkaProducer};

/// 消息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// 消息主题/房间 ID
    pub topic: String,
    /// 消息来源
    pub from: String,
    /// 消息数据（JSON 格式）
    pub data: serde_json::Value,
    /// 时间戳（毫秒）
    pub timestamp: i64,
}

impl Message {
    /// 创建新消息
    pub fn new(topic: impl Into<String>, from: impl Into<String>, data: serde_json::Value) -> Self {
        Self {
            topic: topic.into(),
            from: from.into(),
            data,
            timestamp: chrono::Utc::now().timestamp_millis(),
        }
    }
}

/// 消息生产者 trait
#[async_trait]
pub trait MessageProducer: Send + Sync {
    /// 发布消息到指定 topic
    async fn publish(
        &self,
        topic: &str,
        message: Message,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 批量发布消息
    async fn publish_batch(
        &self,
        messages: Vec<(String, Message)>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        for (topic, message) in messages {
            self.publish(&topic, message).await?;
        }
        Ok(())
    }
}

/// 消息消费者 trait
#[async_trait]
pub trait MessageConsumer: Send + Sync {
    /// 订阅单个 topic
    async fn subscribe(
        &self,
        topic: &str,
        handler: Arc<dyn Fn(Message) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;

    /// 订阅多个 topic
    async fn subscribe_topics(
        &self,
        topics: Vec<String>,
        handler: Arc<dyn Fn(Message) + Send + Sync>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// 消息生产者类型别名
pub type MessageProducerType = Arc<dyn MessageProducer + Send + Sync>;

/// 消息消费者类型别名
pub type MessageConsumerType = Arc<dyn MessageConsumer + Send + Sync>;

/// Kafka 消息处理器 trait
#[cfg(feature = "consumer")]
#[async_trait]
pub trait KafkaMessageHandler: Send + Sync {
    /// 返回此 handler 处理的 topic 列表
    fn topics(&self) -> Vec<String>;

    /// 返回此 handler 使用的消费者组ID
    fn group_id(&self) -> String;

    /// 处理接收到的消息
    async fn handle(&self, message: Message);
}

#[cfg(feature = "consumer")]
pub use router::KafkaMessageRouter;
