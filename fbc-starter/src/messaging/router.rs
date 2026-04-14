// Kafka 消息路由器

use super::{KafkaMessageHandler, Message};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, warn};

/// Kafka 消息路由器
///
/// 根据消息的 topic 将消息分发到对应的 handlers
/// 支持一个 topic 对应多个 handlers（并发执行）
pub struct KafkaMessageRouter {
    /// topic -> 处理该 topic 的所有 handlers
    routes: HashMap<String, Vec<Arc<dyn KafkaMessageHandler>>>,
    /// 所有需要订阅的 topics
    all_topics: Vec<String>,
}

impl KafkaMessageRouter {
    /// 创建新的路由器
    ///
    /// # 参数
    /// - `handlers`: 所有消息处理器
    pub fn new(handlers: Vec<Arc<dyn KafkaMessageHandler>>) -> Self {
        let mut routes: HashMap<String, Vec<Arc<dyn KafkaMessageHandler>>> = HashMap::new();
        let mut all_topics = HashSet::new();

        // 构建路由表
        for handler in handlers {
            for topic in handler.topics() {
                routes
                    .entry(topic.clone())
                    .or_insert_with(Vec::new)
                    .push(handler.clone());
                all_topics.insert(topic);
            }
        }

        debug!(
            "Kafka 路由器初始化完成，共 {} 个 topics，{} 条路由",
            all_topics.len(),
            routes.values().map(|v| v.len()).sum::<usize>()
        );

        Self {
            routes,
            all_topics: all_topics.into_iter().collect(),
        }
    }

    /// 分发消息到对应的 handlers（并发执行）
    ///
    /// # 并发策略
    /// - 如果一个 topic 有多个 handlers，会并发调用所有 handlers
    /// - 开发者需要自行处理并发安全问题（锁、事务等）
    pub async fn dispatch(&self, message: Message) {
        let topic = &message.topic;

        if let Some(handlers) = self.routes.get(topic) {
            debug!(
                "分发消息到 {} 个 handler (topic: {})",
                handlers.len(),
                topic
            );

            // 并发执行所有 handlers
            let mut tasks = Vec::new();
            for handler in handlers {
                let msg = message.clone();
                let h = handler.clone();
                let task = tokio::spawn(async move {
                    h.handle(msg).await;
                });
                tasks.push(task);
            }

            // 等待所有 handler 完成
            for task in tasks {
                if let Err(e) = task.await {
                    warn!("Handler 执行失败: {}", e);
                }
            }
        } else {
            // 收到了配置中订阅的 topic，但没有 handler 处理
            // 这是正常情况，可能是其他服务实例的 handler
            debug!("收到消息但没有 handler 处理 topic: {}", topic);
        }
    }

    /// 获取所有需要订阅的 topics
    pub fn get_subscribe_topics(&self) -> Vec<String> {
        self.all_topics.clone()
    }

    /// 检查是否有 handler 处理指定的 topic
    pub fn has_handler_for(&self, topic: &str) -> bool {
        self.routes.contains_key(topic)
    }

    /// 获取处理指定 topic 的 handler 数量
    pub fn handler_count_for(&self, topic: &str) -> usize {
        self.routes.get(topic).map(|v| v.len()).unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct TestHandler {
        topics: Vec<String>,
        counter: Arc<AtomicUsize>,
    }

    #[async_trait]
    impl KafkaMessageHandler for TestHandler {
        fn topics(&self) -> Vec<String> {
            self.topics.clone()
        }

        fn group_id(&self) -> String {
            "test-group".to_string()
        }

        async fn handle(&self, _message: Message) {
            self.counter.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn test_router_dispatch() {
        let counter1 = Arc::new(AtomicUsize::new(0));
        let counter2 = Arc::new(AtomicUsize::new(0));

        let handler1 = Arc::new(TestHandler {
            topics: vec!["topic1".to_string()],
            counter: counter1.clone(),
        }) as Arc<dyn KafkaMessageHandler>;

        let handler2 = Arc::new(TestHandler {
            topics: vec!["topic1".to_string(), "topic2".to_string()],
            counter: counter2.clone(),
        }) as Arc<dyn KafkaMessageHandler>;

        let router = KafkaMessageRouter::new(vec![handler1, handler2]);

        assert_eq!(router.get_subscribe_topics().len(), 2);
        assert_eq!(router.handler_count_for("topic1"), 2);
        assert_eq!(router.handler_count_for("topic2"), 1);

        let msg = Message::new("topic1", "test", serde_json::json!({}));
        router.dispatch(msg).await;

        assert_eq!(counter1.load(Ordering::SeqCst), 1);
        assert_eq!(counter2.load(Ordering::SeqCst), 1);
    }
}
