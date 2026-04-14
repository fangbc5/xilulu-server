// Kafka 消息发送模块

use crate::state::AppState;
use fbc_starter::Message;
use serde_json::json;

/// 发送验证码通知消息到 Kafka
pub struct NotificationSender;

impl NotificationSender {
    /// 发送验证码通知
    ///
    /// # 参数
    /// - `app_state`: 应用状态
    /// - `account`: 账号（手机号或邮箱）
    /// - `code`: 验证码
    /// - `channel`: 渠道类型（"sms" 或 "email"）
    pub async fn send_verify_code(
        app_state: &AppState,
        account: &str,
        code: &str,
        channel: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let producer = app_state
            .fbc_app_state
            .message_producer()
            .map_err(|e| format!("Kafka producer未初始化: {}", e))?;

        // 构建通知消息数据
        let notification_data = match channel {
            "sms" => {
                // 短信格式：JSON 模板参数字符串
                json!({
                    "to": account,
                    "channel": "sms",
                    "body": json!({
                        "code": code
                    }).to_string(),
                    "subject": "短信验证码",
                    "from": "乌拉科技",
                    "biz_id": account,
                    "biz_type": "verify_code"
                })
            }
            "email" => {
                // 邮件格式（from 留空使用 ms-notify 配置的默认发件人）
                json!({
                    "to": account,
                    "channel": "email",
                    "subject": "邮箱验证码",
                    "body": format!("您的验证码是：{}，有效期5分钟。", code),
                    "from": "fangbc5@163.com",
                    "biz_id": account,
                    "biz_type": "verify_code"
                })
            }
            _ => {
                return Err(format!("不支持的渠道类型: {}", channel).into());
            }
        };

        // 创建消息
        let message = Message::new(
            "ms-notify-topic",  // topic
            "ms-auth", // from
            notification_data,
        );

        // 发送消息
        producer
            .publish("ms-notify-topic", message)
            .await
            .map_err(|e| format!("发送Kafka消息失败: {}", e))?;

        Ok(())
    }
}
