use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx;

/// 文件元数据实体
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FileMeta {
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
    pub status: Option<i8>,
    /// 水印状态：0=无 1=已添加
    pub watermark: Option<i8>,
    /// 缩略图 Key
    pub thumbnail_key: Option<String>,
    /// 创建时间
    pub created_at: Option<NaiveDateTime>,
    /// 更新时间
    pub updated_at: Option<NaiveDateTime>,
}
