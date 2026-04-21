use serde::{Deserialize, Serialize};
use sqlx;

/// 文件元数据实体
#[derive(
    Debug, Default, Clone, Serialize, Deserialize, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD,
)]
#[model(table = "file_meta", pk = "id")]
pub struct FileMeta {
    #[column(primary_key, auto_increment)]
    pub id: Option<i64>,
    /// 对象存储 Key（唯一路径）
    pub file_key: Option<String>,
    /// Bucket 名称
    pub bucket: Option<String>,
    /// 原始文件名
    pub original_name: Option<String>,
    /// 文件 MIME 类型
    pub content_type: Option<String>,
    /// 文件大小（字节）
    pub size: Option<i64>,
    /// 业务场景（avatar / chat_image / logo / document 等）
    pub scene: Option<String>,
    /// 上传者用户 ID
    pub uploader_id: Option<i64>,
    /// OSS 厂商（rustfs / aliyun / tencent / aws）
    pub provider: Option<String>,
    /// 状态：0=待上传 1=已上传 2=已删除
    #[column(not_null)]
    pub status: Option<i16>,
    /// 水印状态：0=无 1=已添加
    #[column(not_null)]
    pub watermark: Option<i16>,
    /// 缩略图 Key
    pub thumbnail_key: Option<String>,
    /// 创建时间
    #[column(not_null)]
    pub created_at: Option<i64>,
    /// 更新时间
    #[column(not_null)]
    pub updated_at: Option<i64>,
}
