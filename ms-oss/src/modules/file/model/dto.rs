use serde::{Deserialize, Serialize};

/// 预签名上传请求
#[derive(Debug, Deserialize)]
pub struct PresignUploadRequest {
    /// Bucket（可选，默认按 SceneRule 或 default_bucket）
    pub bucket: Option<String>,
    /// 原始文件名
    pub filename: String,
    /// 文件 MIME 类型
    pub content_type: Option<String>,
    /// 业务场景（avatar / chat_image / logo / document 等）
    pub scene: String,
    /// 文件大小（字节，可选，用于签名前校验 + 回调时校验）
    pub size: Option<i64>,
}

/// 预签名上传响应
#[derive(Debug, Serialize)]
pub struct PresignUploadResponse {
    /// 预签名上传 URL
    pub upload_url: String,
    /// 生成的 Object Key
    pub object_key: String,
    /// 文件元数据记录 ID
    pub file_id: i64,
    /// URL 过期时间（秒）
    pub expires_in: u64,
}

/// 预签名下载请求
#[derive(Debug, Deserialize)]
pub struct PresignDownloadRequest {
    /// Bucket（可选，默认使用 default_bucket）
    pub bucket: Option<String>,
    /// 对象 Key
    pub object_key: String,
}

/// 预签名下载响应
#[derive(Debug, Serialize)]
pub struct PresignDownloadResponse {
    /// 预签名下载 URL
    pub download_url: String,
    /// URL 过期时间（秒）
    pub expires_in: u64,
}

/// 上传完成回调请求
#[derive(Debug, Deserialize)]
pub struct UploadCallbackRequest {
    /// 文件元数据记录 ID
    pub file_id: i64,
    /// 文件大小（字节）
    pub size: Option<i64>,
}

/// 文件元数据响应
#[derive(Debug, Serialize)]
pub struct FileMetaResponse {
    pub id: i64,
    pub file_key: String,
    pub bucket: String,
    pub original_name: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<i64>,
    pub scene: String,
    pub status: i8,
    pub created_at: String,
}
