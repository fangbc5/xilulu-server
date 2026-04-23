#[derive(
    Debug, Default, Clone,
    sqlx::FromRow,
    serde::Serialize, serde::Deserialize,
    sqlxplus::ModelMeta, sqlxplus::CRUD,
)]
#[model(table = "media_tasks", pk = "id", table_comment = "媒体任务表")]
pub struct MediaTask {
    #[column(primary_key, length = 64, comment = "任务ID")]
    pub id: Option<String>,
    #[column(length = 64, comment = "源 Bucket")]
    pub source_bucket: Option<String>,
    #[column(length = 255, comment = "源路径")]
    pub source_key: Option<String>,
    #[column(length = 32, comment = "任务类型")]
    pub task_type: Option<String>,
    #[column(comment = "动态参数 JSON")]
    pub parameters: Option<String>,
    #[column(length = 20, default = "INIT", comment = "状态")]
    pub status: Option<String>,
    #[column(default = "0", comment = "重试次数")]
    pub retry_count: Option<i32>,
    #[column(default = "1", comment = "乐观锁")]
    pub version: Option<i32>,
    #[column(length = 255, comment = "衍生文件路径")]
    pub result_key: Option<String>,
    #[column(comment = "错误信息")]
    pub error_message: Option<String>,
    #[column(not_null, comment = "创建时间")]
    pub created_at: Option<i64>,
    #[column(not_null, comment = "更新时间")]
    pub updated_at: Option<i64>,
}

impl MediaTask {
    pub fn new_task(id: String, source_bucket: String, source_key: String, task_type: String, parameters: Option<serde_json::Value>) -> Self {
        Self {
            id: Some(id),
            source_bucket: Some(source_bucket),
            source_key: Some(source_key),
            task_type: Some(task_type),
            parameters: parameters.map(|p| p.to_string()),
            status: Some("INIT".to_string()),
            retry_count: Some(0),
            version: Some(1),
            result_key: None,
            error_message: None,
            created_at: None,
            updated_at: None,
        }
    }
}
