/// ms-content 服务配置（业务专属，基础设施由 fbc-starter 自动加载）
#[derive(Debug, Clone)]
pub struct ContentConfig {
    /// Meilisearch 服务地址
    pub meilisearch_url: String,
    /// Meilisearch API Key
    pub meilisearch_api_key: String,
    /// XXL-JOB Admin 地址
    pub xxl_admin_addr: String,
    /// XXL-JOB 访问令牌
    pub xxl_access_token: String,
    /// XXL-JOB Executor 监听端口
    pub xxl_executor_port: u16,
}

impl ContentConfig {
    /// 从环境变量加载配置（遵循 APP__CONTENT__ 命名规范）
    pub fn from_env() -> Self {
        Self {
            meilisearch_url: std::env::var("APP__CONTENT__MEILISEARCH_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:7700".to_string()),
            meilisearch_api_key: std::env::var("APP__CONTENT__MEILISEARCH_API_KEY")
                .expect("缺少 APP__CONTENT__MEILISEARCH_API_KEY"),
            xxl_admin_addr: std::env::var("APP__CONTENT__XXL_ADMIN_ADDR")
                .unwrap_or_else(|_| "http://127.0.0.1:8725/xxl-job-admin".to_string()),
            xxl_access_token: std::env::var("APP__CONTENT__XXL_ACCESS_TOKEN")
                .unwrap_or_else(|_| String::new()),
            xxl_executor_port: std::env::var("APP__CONTENT__XXL_EXECUTOR_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(31106),
        }
    }
}
