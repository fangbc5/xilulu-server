use sqlxplus::DbPool;
use std::sync::Arc;

use crate::models::Notification;
use super::entity::NotifyLog;
use super::repository::NotifyLogRepo;

/// 通知日志服务
///
/// 封装"记录→发送→更新"流程
pub struct NotifyLogService {
    db: Arc<DbPool>,
}

impl NotifyLogService {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// 发送前：入库（直接以"发送中"状态插入），返回日志 ID
    pub async fn before_send(&self, notification: &Notification) -> anyhow::Result<i64> {
        let channel_str = serde_json::to_value(&notification.channel)
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| format!("{:?}", notification.channel));

        let mut log = NotifyLog {
            id: None,
            channel: Some(channel_str),
            sender: if notification.from.is_empty() { None } else { Some(notification.from.clone()) },
            receiver: if notification.to.is_empty() { None } else { Some(notification.to.clone()) },
            subject: if notification.subject.is_empty() { None } else { Some(notification.subject.clone()) },
            body: if notification.body.is_empty() { None } else { Some(notification.body.clone()) },
            status: Some(1), // 直接设为发送中，省去一次 UPDATE
            error_msg: None,
            retry_count: Some(0),
            biz_type: notification.biz_type.clone(),
            biz_id: notification.biz_id.clone(),
            created_at: None,
            updated_at: None,
        };

        let id = NotifyLogRepo::insert(self.db.mysql_pool(), &mut log).await?;
        Ok(id)
    }

    /// 发送成功：更新状态
    pub async fn on_success(&self, log_id: i64) {
        if let Err(e) = NotifyLogRepo::mark_success(self.db.mysql_pool(), log_id).await {
            tracing::error!("更新通知日志状态失败: id={}, err={}", log_id, e);
        }
    }

    /// 发送失败：记录错误
    pub async fn on_failure(&self, log_id: i64, error: &str) {
        if let Err(e) = NotifyLogRepo::mark_failed(self.db.mysql_pool(), log_id, error).await {
            tracing::error!("更新通知日志状态失败: id={}, err={}", log_id, e);
        }
    }
}
