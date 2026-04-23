use fbc_starter::AppState;
use sqlxplus::DbPool;
use std::sync::Arc;

use crate::config::OssConfig;
use crate::modules::file::service::FileService;
use crate::provider::OssProvider;

/// OSS 服务整体状态
///
/// 持有基础的 `AppState`（Redis 等）、数据库连接池、OSS Provider、业务 Service
#[derive(Clone)]
pub struct OssState {
    /// 文件服务
    pub file_service: Arc<FileService>,
}

impl OssState {
    pub fn new(
        app_state: Arc<AppState>,
        db_pool: Arc<DbPool>,
        config: OssConfig,
        provider: Arc<dyn OssProvider>,
    ) -> Self {
        let producer = app_state.message_producer().ok().cloned();
        let file_service = Arc::new(FileService::new(
            db_pool,
            config,
            provider,
            producer,
        ));

        Self { file_service }
    }
}
