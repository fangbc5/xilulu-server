use serde::{Deserialize, Serialize};

// ============================================
// 签名服务 DTO
// ============================================

/// 统一签名请求
#[derive(Debug, Deserialize)]
pub struct SignatureRequest {
    /// 操作方法：put / get / share
    pub method: String,
    /// Bucket（可选，默认按 scene 规则或 default_bucket）
    pub bucket: Option<String>,
    /// 对象 Key（put 可选——为空时自动生成；get/share 必填）
    pub key: Option<String>,
    /// 文件 MIME 类型（put 时需要）
    pub content_type: Option<String>,
    /// 业务场景（put 时需要）
    pub scene: Option<String>,
    /// 文件大小（字节，put 时可选校验）
    pub size: Option<i64>,
    /// 原始文件名（put 时，用于提取扩展名和审计）
    pub filename: Option<String>,
    /// 过期时间（秒，get/share 时可选）
    pub expires_in: Option<u64>,
    /// 绑定处理参数（share 时可选，防篡改）
    pub x_oss_process: Option<String>,
}

/// 上传签名响应
#[derive(Debug, Serialize)]
pub struct SignatureUploadResponse {
    /// 预签名上传 URL
    pub url: String,
    /// 生成的对象 Key
    pub object_key: String,
    /// HTTP 方法
    pub method: String,
    /// 过期时间（秒）
    pub expires_in: u64,
}

/// 下载签名响应
#[derive(Debug, Serialize)]
pub struct SignatureDownloadResponse {
    /// 预签名下载 URL
    pub url: String,
    /// HTTP 方法
    pub method: String,
    /// 过期时间（秒）
    pub expires_in: u64,
}

/// 分享链接响应
#[derive(Debug, Serialize)]
pub struct SignatureShareResponse {
    /// 分享 URL
    pub url: String,
    /// 过期时间（秒）
    pub expires_in: u64,
}

// ============================================
// 对象操作 DTO
// ============================================

/// PutObject 响应
#[derive(Debug, Serialize)]
pub struct PutObjectResponse {
    /// 预签名上传 URL
    pub upload_url: String,
    /// 生成的对象 Key
    pub object_key: String,
    /// 过期时间（秒）
    pub expires_in: u64,
}

/// PostObject（上传确认）响应
#[derive(Debug, Serialize)]
pub struct ObjectInfoResponse {
    /// Bucket 名称
    pub bucket: String,
    /// 对象 Key
    pub key: String,
    /// Content-Type
    pub content_type: Option<String>,
    /// 文件大小
    pub size: Option<i64>,
}

/// 分片上传初始化响应
#[derive(Debug, Serialize)]
pub struct MultipartInitResponse {
    /// 分片上传 ID
    pub upload_id: String,
    /// 生成的对象 Key
    pub object_key: String,
    /// 分片数量
    pub part_count: usize,
    /// 各分片预签名 URL
    pub part_urls: Vec<PartUrlInfo>,
    /// 过期时间（秒）
    pub expires_in: u64,
}

/// 分片 URL 信息
#[derive(Debug, Serialize)]
pub struct PartUrlInfo {
    /// 分片序号
    pub part_number: u32,
    /// 预签名上传 URL
    pub upload_url: String,
}

/// 完成分片上传请求
#[derive(Debug, Deserialize)]
pub struct CompleteMultipartRequest {
    /// 已上传的分片列表
    pub parts: Vec<PartInfo>,
}

/// 分片信息
#[derive(Debug, Deserialize, Serialize)]
pub struct PartInfo {
    /// 分片序号
    pub part_number: u32,
    /// ETag
    pub etag: String,
}

/// 查询已上传分片响应
#[derive(Debug, Serialize)]
pub struct ListPartsResponse {
    /// 分片上传 ID
    pub upload_id: String,
    /// 已上传分片列表
    pub parts: Vec<PartDetail>,
    /// 下一个待上传的分片序号
    pub next_part_number: u32,
}

/// 分片详情
#[derive(Debug, Serialize)]
pub struct PartDetail {
    /// 分片序号
    pub part_number: u32,
    /// ETag
    pub etag: String,
    /// 分片大小
    pub size: i64,
}

// ============================================
// 旧接口 DTO（供 gRPC 兼容用，标记废弃）
// ============================================

/// 预签名上传请求（旧）
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

/// 预签名上传响应（旧）
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

/// 预签名下载请求（旧）
#[derive(Debug, Deserialize)]
pub struct PresignDownloadRequest {
    /// Bucket（可选，默认使用 default_bucket）
    pub bucket: Option<String>,
    /// 对象 Key
    pub object_key: String,
}

/// 预签名下载响应（旧）
#[derive(Debug, Serialize)]
pub struct PresignDownloadResponse {
    /// 预签名下载 URL
    pub download_url: String,
    /// URL 过期时间（秒）
    pub expires_in: u64,
}

/// 上传完成回调请求（旧）
#[derive(Debug, Deserialize)]
pub struct UploadCallbackRequest {
    /// 文件元数据记录 ID
    pub file_id: i64,
    /// 文件大小（字节）
    pub size: Option<i64>,
}

/// 文件元数据响应（旧，gRPC 用）
#[derive(Debug, Serialize)]
pub struct FileMetaResponse {
    pub id: i64,
    pub file_key: String,
    pub bucket: String,
    pub original_name: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<i64>,
    pub scene: String,
    pub status: i16,
    pub created_at: i64,
}
