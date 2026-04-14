use crate::adapters::Sender;
use crate::adapters::{DingdingSender, EmailSender, FeishuSender, SmsSender};
use crate::config::NotifyConfig;
use crate::error::NotifyError;
use crate::models::{ChannelType, Notification};
use crate::modules::notify_log::NotifyLogService;
use async_trait::async_trait;
use fbc_starter::{KafkaMessageHandler, Message as KafkaMessage};
use std::sync::Arc;
use tracing::{error, info, warn};

/// Kafka 消息处理器上下文
/// 包含所有消息发送器和日志服务
pub struct NotificationHandlerContext {
    email_sender: Option<EmailSender>,
    sms_sender: Option<SmsSender>,
    feishu_sender: Option<FeishuSender>,
    dingding_sender: Option<DingdingSender>,
    /// 邮件配置（用于获取默认发件人）
    email_config: Option<crate::config::EmailConfig>,
    /// 通知日志服务（可选，无 DB 时降级为不记录）
    log_service: Option<Arc<NotifyLogService>>,
}

impl NotificationHandlerContext {
    /// 创建处理器上下文
    pub fn new(config: &NotifyConfig) -> Self {
        Self {
            email_sender: config
                .notify
                .email
                .as_ref()
                .map(|cfg| EmailSender::new(cfg)),
            sms_sender: config.notify.sms.clone().map(|cfg| SmsSender::new(cfg)),
            feishu_sender: config
                .notify
                .feishu
                .clone()
                .map(|cfg| FeishuSender::new(cfg)),
            dingding_sender: config
                .notify
                .dingding
                .clone()
                .map(|cfg| DingdingSender::new(cfg)),
            email_config: config.notify.email.clone(),
            log_service: None,
        }
    }

    /// 注入日志服务
    pub fn with_log_service(mut self, service: Arc<NotifyLogService>) -> Self {
        self.log_service = Some(service);
        self
    }

    /// 发送通知消息（带日志记录）
    pub async fn send(&self, notification: &Notification) -> Result<(), NotifyError> {
        // 1. 入库记录
        let log_id = if let Some(ref log_svc) = self.log_service {
            match log_svc.before_send(notification).await {
                Ok(id) => Some(id),
                Err(e) => {
                    warn!("通知日志入库失败（不影响发送）: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // 2. 实际发送
        let result = self.do_send(notification).await;

        // 3. 异步更新日志状态（fire-and-forget，不阻塞下一条消息处理）
        if let Some(ref log_svc) = self.log_service {
            if let Some(id) = log_id {
                let log_svc = Arc::clone(log_svc);
                let error_msg = result.as_ref().err().map(|e| e.to_string());
                tokio::spawn(async move {
                    match error_msg {
                        None => log_svc.on_success(id).await,
                        Some(msg) => log_svc.on_failure(id, &msg).await,
                    }
                });
            }
        }

        result
    }

    /// 实际发送逻辑（从原 send 提取）
    async fn do_send(&self, notification: &Notification) -> Result<(), NotifyError> {
        match notification.channel {
            ChannelType::Email => {
                let sender = self.email_sender.as_ref().ok_or_else(|| {
                    NotifyError::Config("Email sender not configured".to_string())
                })?;

                let notification = if notification.from.is_empty() {
                    let from = self
                        .email_config
                        .as_ref()
                        .map(|cfg| cfg.smtp_user.clone())
                        .unwrap_or_else(|| "noreply@example.com".to_string());
                    Notification {
                        from,
                        ..notification.clone()
                    }
                } else {
                    notification.clone()
                };

                sender.send(&notification).await?;
            }
            ChannelType::Sms => {
                let sender = self
                    .sms_sender
                    .as_ref()
                    .ok_or_else(|| NotifyError::Config("SMS sender not configured".to_string()))?;
                sender.send(notification).await?;
            }
            ChannelType::ImFeishu => {
                let sender = self.feishu_sender.as_ref().ok_or_else(|| {
                    NotifyError::Config("Feishu sender not configured".to_string())
                })?;
                sender.send(notification).await?;
            }
            ChannelType::ImDingding => {
                let sender = self.dingding_sender.as_ref().ok_or_else(|| {
                    NotifyError::Config("Dingding sender not configured".to_string())
                })?;
                sender.send(notification).await?;
            }
            _ => {
                return Err(NotifyError::Config(format!(
                    "Unsupported channel type: {:?}",
                    notification.channel
                )));
            }
        }
        Ok(())
    }
}

/// Kafka 通知消息处理器
pub struct NotificationHandler {
    context: Arc<NotificationHandlerContext>,
}

impl NotificationHandler {
    pub fn new(context: Arc<NotificationHandlerContext>) -> Self {
        Self { context }
    }
}

#[async_trait]
impl KafkaMessageHandler for NotificationHandler {
    fn topics(&self) -> Vec<String> {
        vec!["ms-notify-topic".to_string()]
    }

    fn group_id(&self) -> String {
        "ms-notify-group-1".to_string()
    }

    async fn handle(&self, message: KafkaMessage) {
        info!(
            "Received Kafka message: topic={}, from={}",
            message.topic, message.from
        );

        match serde_json::from_value::<Notification>(message.data.clone()) {
            Ok(notification) => {
                if let Err(e) = dispatch(&self.context, notification).await {
                    error!("Failed to dispatch notification: {}", e);
                }
            }
            Err(_) => match parse_flare_format(&message.data) {
                Ok(notification) => {
                    if let Err(e) = dispatch(&self.context, notification).await {
                        error!("Failed to dispatch notification: {}", e);
                    }
                }
                Err(e) => {
                    warn!(
                        "Failed to parse notification message: {}, data: {}",
                        e, message.data
                    );
                }
            },
        }
    }
}

/// 解析 flare-worker 格式的消息
fn parse_flare_format(data: &serde_json::Value) -> Result<Notification, NotifyError> {
    let channel = data
        .get("channel")
        .and_then(|v| serde_json::from_value::<ChannelType>(v.clone()).ok())
        .ok_or_else(|| NotifyError::Config("missing or invalid 'channel' field".to_string()))?;

    let payload = data
        .get("payload")
        .ok_or_else(|| NotifyError::Config("missing 'payload' field".to_string()))?;

    let biz_type = data
        .get("biz_type")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let biz_id = data
        .get("biz_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    match channel {
        ChannelType::Email => {
            let from = payload
                .get("from")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "noreply@example.com".to_string());
            let to = require_str(payload, "to")?;
            let subject = require_str(payload, "subject")?;
            let body = require_str(payload, "body")?;
            Ok(Notification {
                from,
                to,
                subject,
                body,
                channel,
                biz_type,
                biz_id,
            })
        }
        ChannelType::Sms => {
            let to = require_str(payload, "to")?;
            let body = require_str(payload, "param").or_else(|_| require_str(payload, "body"))?;
            Ok(Notification {
                from: String::new(),
                to,
                subject: String::new(),
                body,
                channel,
                biz_type,
                biz_id,
            })
        }
        ChannelType::ImFeishu | ChannelType::ImDingding => {
            let body = require_str(payload, "text").or_else(|_| require_str(payload, "body"))?;
            Ok(Notification {
                from: String::new(),
                to: String::new(),
                subject: String::new(),
                body,
                channel,
                biz_type,
                biz_id,
            })
        }
        _ => Err(NotifyError::Config(format!(
            "Unsupported channel type: {:?}",
            channel
        ))),
    }
}

/// 分发消息到对应的处理器
async fn dispatch(
    ctx: &NotificationHandlerContext,
    notification: Notification,
) -> Result<(), NotifyError> {
    ctx.send(&notification).await?;
    info!(
        "Notification sent successfully: channel={:?}, to={}",
        notification.channel, notification.to
    );
    Ok(())
}

/// 从 payload 中获取字符串字段
fn require_str(payload: &serde_json::Value, key: &str) -> Result<String, NotifyError> {
    payload
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            NotifyError::Config(format!("missing or invalid '{}' field in payload", key))
        })
}
