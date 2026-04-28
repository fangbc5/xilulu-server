use sqlxplus::DbPool;
use std::sync::Arc;

use crate::modules::content::search::port::SearchPort;
use crate::modules::content::service::ContentService;

/// ms-content 服务整体状态
///
/// 持有所有业务 Service 实例，供 Handler 层通过 Axum State 注入使用
#[derive(Clone)]
pub struct ContentState {
    /// 内容服务
    pub content_service: Arc<ContentService>,
}

impl ContentState {
    /// 构建应用状态
    pub fn new(
        db_pool: Arc<DbPool>,
        search_port: Arc<dyn SearchPort>,
    ) -> Self {
        let content_service = Arc::new(ContentService::new(db_pool, search_port));
        Self { content_service }
    }
}
