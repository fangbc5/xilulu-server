use fbc_starter::AppState;
use sqlxplus::DbPool;
use std::sync::Arc;

use crate::config::OssConfig;
use crate::modules::file::process::FileProcessor;
use crate::modules::file::service::FileService;
use crate::provider::OssProvider;

/// OSS 服务整体状态
///
/// 持有基础的 `AppState`（Redis 等）、数据库连接池、OSS Provider、业务 Service
#[derive(Clone)]
pub struct OssState {
    /// 启动器提供的应用状态
    pub app_state: Arc<AppState>,
    /// 数据库连接池
    pub db_pool: Arc<DbPool>,
    /// OSS 配置
    pub config: OssConfig,
    /// 文件服务
    pub file_service: Arc<FileService>,
    /// 文件处理器列表（按配置启用）
    pub processors: Vec<Arc<dyn FileProcessor>>,
}

impl OssState {
    pub fn new(
        app_state: Arc<AppState>,
        db_pool: Arc<DbPool>,
        config: OssConfig,
        provider: Arc<dyn OssProvider>,
        processors: Vec<Arc<dyn FileProcessor>>,
    ) -> Self {
        let producer = app_state.message_producer().ok().cloned();
        let file_service = Arc::new(FileService::new(
            db_pool.clone(),
            config.clone(),
            provider,
            producer,
        ));

        Self {
            app_state,
            db_pool,
            config,
            file_service,
            processors,
        }
    }
}
