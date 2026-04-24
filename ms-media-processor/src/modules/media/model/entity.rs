/// 媒体任务数据库实体
#[derive(
    Debug,
    Default,
    Clone,
    sqlx::FromRow,
    serde::Serialize,
    serde::Deserialize,
    sqlxplus::ModelMeta,
    sqlxplus::CRUD,
)]
#[model(table = "media_task", pk = "id", table_comment = "媒体处理任务表")]
pub struct MediaTask {
    #[column(primary_key, auto_increment, comment = "自增主键")]
    pub id: Option<i64>,
    #[column(length = 64, not_null, comment = "任务ID")]
    pub task_id: Option<String>,
    #[column(length = 64, not_null, comment = "源文件 Bucket")]
    pub source_bucket: Option<String>,
    #[column(length = 512, not_null, comment = "源文件路径")]
    pub source_key: Option<String>,
    #[column(length = 32, not_null, comment = "任务类型")]
    pub task_type: Option<String>,
    #[column(comment = "任务参数 JSON")]
    pub parameters: Option<String>,
    #[column(length = 20, not_null, default = "INIT", comment = "任务状态")]
    pub status: Option<String>,
    #[column(not_null, default = "0", comment = "优先级")]
    pub priority: Option<i32>,
    #[column(not_null, default = "0", comment = "已重试次数")]
    pub retry_count: Option<i32>,
    #[column(not_null, default = "3", comment = "最大重试次数")]
    pub max_retry: Option<i32>,
    #[column(not_null, default = "1", comment = "乐观锁版本")]
    pub version: Option<i32>,
    #[column(length = 512, comment = "主产物路径")]
    pub result_key: Option<String>,
    #[column(comment = "产物元信息 JSON")]
    pub result_meta: Option<String>,
    #[column(comment = "错误信息")]
    pub error_message: Option<String>,
    #[column(length = 128, comment = "完成回调 Kafka topic")]
    pub callback_topic: Option<String>,
    #[column(length = 64, comment = "提交方服务名")]
    pub created_by: Option<String>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
    #[column(not_null, comment = "更新时间")]
    pub updated_at: Option<i64>,
}

/// 媒体任务产物数据库实体
#[derive(
    Debug,
    Default,
    Clone,
    sqlx::FromRow,
    serde::Serialize,
    serde::Deserialize,
    sqlxplus::ModelMeta,
    sqlxplus::CRUD,
)]
#[model(
    table = "media_task_output",
    pk = "id",
    table_comment = "媒体任务产物表"
)]
pub struct MediaTaskOutput {
    #[column(primary_key, auto_increment, comment = "自增主键")]
    pub id: Option<i64>,
    #[column(length = 64, not_null, comment = "关联任务ID")]
    pub task_id: Option<String>,
    #[column(length = 512, not_null, comment = "产物 S3 路径")]
    pub output_key: Option<String>,
    #[column(length = 32, comment = "产物类型")]
    pub output_type: Option<String>,
    #[column(length = 64, comment = "MIME 类型")]
    pub content_type: Option<String>,
    #[column(comment = "文件大小（字节）")]
    pub file_size: Option<i64>,
    #[column(comment = "额外元信息 JSON")]
    pub metadata: Option<String>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
}
