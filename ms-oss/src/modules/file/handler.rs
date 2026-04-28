//! 对标阿里云 OSS 的 RESTful Handler
//!
//! 所有 handler 遵循阿里云 OSS API 语义：
//! - 签名服务：POST /oss/signature
//! - 长效分享：GET /oss/share/{token}
//! - 对象操作：PUT/POST/GET/HEAD/DELETE /oss/{bucket}/*key

use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use fbc_starter::{RequestContext, R};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::IntoParams;

use super::model::dto::*;
use crate::error::OssError;
use crate::state::OssState;

// ============================================
// 签名服务
// ============================================

/// 统一签名服务
///
/// 根据 method 字段分发：put → 上传签名、get → 下载签名、share → 长效分享
#[utoipa::path(
    post,
    path = "/api/v1/oss/signature",
    tag = "签名服务",
    request_body = SignatureRequest,
    responses(
        (status = 200, description = "put → 上传签名", body = R<SignatureUploadResponse>),
        (status = 400, description = "请求参数错误"),
    )
)]
pub async fn create_signature(
    State(state): State<Arc<OssState>>,
    ctx: Option<RequestContext>,
    Json(req): Json<SignatureRequest>,
) -> Result<Response, OssError> {
    let uploader_id = ctx.map(|c| c.user_id);

    match req.method.to_lowercase().as_str() {
        "put" => {
            let resp = state
                .file_service
                .signature_upload(req, uploader_id)
                .await?;
            Ok(Json(R::ok_with_data(resp)).into_response())
        }
        "get" => {
            let resp = state.file_service.signature_download(req).await?;
            Ok(Json(R::ok_with_data(resp)).into_response())
        }
        "share" => {
            let resp = state.file_service.signature_share(req).await?;
            Ok(Json(R::ok_with_data(resp)).into_response())
        }
        other => Err(OssError::BadRequest(format!(
            "不支持的 method: {}，可选值: put/get/share",
            other
        ))),
    }
}

// ============================================
// 长效分享
// ============================================

/// 长效分享链接 302 入口
#[utoipa::path(
    get,
    path = "/api/v1/oss/share/{token}",
    tag = "分享服务",
    params(
        ("token" = String, Path, description = "JWT 分享 Token")
    ),
    responses(
        (status = 302, description = "重定向到实际文件地址"),
        (status = 401, description = "Token 无效或过期"),
    )
)]
pub async fn share_redirect(
    State(state): State<Arc<OssState>>,
    Path(token): Path<String>,
) -> Result<Response, OssError> {
    let redirect_url = state.file_service.resolve_share(&token).await?;
    Ok(Redirect::temporary(&redirect_url).into_response())
}

// ============================================
// PutObject — 预签名上传
// ============================================

/// PutObject — 预签名上传
///
/// 校验 scene 规则 → 生成预签名 URL → 返回
#[utoipa::path(
    put,
    path = "/api/v1/oss/{bucket}/{key}",
    tag = "对象操作",
    params(
        ("bucket" = String, Path, description = "Bucket 名称"),
        ("key" = String, Path, description = "对象 Key（路径）"),
    ),
    responses(
        (status = 200, description = "预签名上传 URL", body = R<PutObjectResponse>),
        (status = 400, description = "场景规则校验失败"),
    )
)]
pub async fn put_object(
    State(state): State<Arc<OssState>>,
    Path((bucket, key)): Path<(String, String)>,
    ctx: Option<RequestContext>,
    headers: HeaderMap,
) -> Result<Json<R<PutObjectResponse>>, OssError> {
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let scene = headers
        .get("x-oss-meta-scene")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let original_name = headers
        .get("x-oss-meta-original-name")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());
    let content_length: Option<i64> = headers
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok());
    let uploader_id = ctx.map(|c| c.user_id);

    let resp = state
        .file_service
        .put_object(
            &bucket,
            &key,
            content_type.as_deref(),
            scene.as_deref(),
            original_name.as_deref(),
            content_length,
            uploader_id,
        )
        .await?;

    Ok(Json(R::ok_with_data(resp)))
}

// ============================================
// PostObject — 上传确认 / 分片上传
// ============================================

/// PostObject Query 参数
#[derive(Debug, Deserialize, Default, IntoParams)]
pub struct PostObjectQuery {
    /// 存在则为初始化分片上传
    pub uploads: Option<String>,
    /// 存在则为完成分片上传
    #[serde(rename = "uploadId")]
    #[param(rename = "uploadId")]
    pub upload_id: Option<String>,
}

/// PostObject — 分片上传系列
///
/// - `?uploads` → 初始化分片上传
/// - `?uploadId=xxx` + body → 完成分片上传
#[utoipa::path(
    post,
    path = "/api/v1/oss/{bucket}/{key}",
    tag = "分片上传",
    params(
        ("bucket" = String, Path, description = "Bucket 名称"),
        ("key" = String, Path, description = "对象 Key（路径）"),
        PostObjectQuery,
    ),
    request_body(content = Option<CompleteMultipartRequest>, description = "完成分片上传时提供已上传分片列表"),
    responses(
        (status = 200, description = "分片上传初始化 / 完成", body = R<MultipartInitResponse>),
        (status = 400, description = "请求参数错误"),
    )
)]
pub async fn post_object(
    State(state): State<Arc<OssState>>,
    Path((bucket, key)): Path<(String, String)>,
    ctx: Option<RequestContext>,
    headers: HeaderMap,
    Query(query): Query<PostObjectQuery>,
    body: Option<Json<CompleteMultipartRequest>>,
) -> Result<Response, OssError> {
    if query.uploads.is_some() {
        // 初始化分片上传
        let scene = headers
            .get("x-oss-meta-scene")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("default");
        let content_type = headers
            .get("x-oss-meta-content-type")
            .and_then(|v| v.to_str().ok());
        let total_size: Option<i64> = headers
            .get("x-oss-meta-total-size")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());
        let part_size: Option<i64> = headers
            .get("x-oss-meta-part-size")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse().ok());
        let original_name = headers
            .get("x-oss-meta-original-name")
            .and_then(|v| v.to_str().ok())
            .map(|s| {
                // 如果前端传的是 URL 编码的中文，这里做一次解码尝试
                urlencoding::decode(s)
                    .map(|c| c.into_owned())
                    .unwrap_or_else(|_| s.to_string())
            });
        let uploader_id = ctx.map(|c| c.user_id);

        let resp = state
            .file_service
            .initiate_multipart(
                &bucket,
                &key,
                scene,
                content_type,
                total_size,
                part_size,
                original_name.as_deref(),
                uploader_id,
            )
            .await?;
        Ok(Json(R::ok_with_data(resp)).into_response())
    } else if let Some(upload_id) = query.upload_id {
        // 完成分片上传
        let parts = body
            .ok_or_else(|| OssError::BadRequest("缺少 parts 请求体".into()))?
            .0;
        let resp = state
            .file_service
            .complete_multipart(&bucket, &key, &upload_id, parts.parts)
            .await?;
        Ok(Json(R::ok_with_data(resp)).into_response())
    } else {
        // 单文件上传已彻底改为监听 MinIO 本地 Kafka Event 落库，不再接受 HTTP 手动确认
        Err(OssError::BadRequest(
            "单文件请直接直传至 MinIO（由 Kafka 接管确认）。仅分片上传可调用本接口。".into(),
        ))
    }
}

// ============================================
// GetObject — 下载 / 图片处理 / 视频产物 / ListParts
// ============================================

/// GetObject Query 参数
#[derive(Debug, Deserialize, Default, IntoParams)]
pub struct GetObjectQuery {
    /// 图片/视频处理参数（如 image/resize,m_fill,w_128,h_128）
    #[serde(rename = "x-oss-process")]
    #[param(rename = "x-oss-process")]
    pub x_oss_process: Option<String>,
    /// 分片上传 ID（用于 ListParts 查询）
    #[serde(rename = "uploadId")]
    #[param(rename = "uploadId")]
    pub upload_id: Option<String>,
}

/// GetObject — 下载 / 图片处理 / 视频产物 / ListParts
///
/// - 无 query → 原文件下载（302 → S3）
/// - `?x-oss-process=image/...` → 图片实时处理（302 → imgproxy）
/// - `?x-oss-process=video/...` → 视频截帧产物
/// - `?x-oss-process=style/...` → Style 预设
/// - `?uploadId=xxx` → ListParts
#[utoipa::path(
    get,
    path = "/api/v1/oss/{bucket}/{key}",
    tag = "对象操作",
    params(
        ("bucket" = String, Path, description = "Bucket 名称"),
        ("key" = String, Path, description = "对象 Key（路径）"),
        GetObjectQuery,
    ),
    responses(
        (status = 302, description = "重定向到 S3 / imgproxy / 视频产物"),
        (status = 200, description = "ListParts 查询结果", body = R<ListPartsResponse>),
    )
)]
pub async fn get_object(
    State(state): State<Arc<OssState>>,
    Path((bucket, key)): Path<(String, String)>,
    Query(query): Query<GetObjectQuery>,
) -> Result<Response, OssError> {
    if let Some(upload_id) = query.upload_id {
        // ListParts
        let resp = state
            .file_service
            .list_parts(&bucket, &key, &upload_id)
            .await?;
        Ok(Json(R::ok_with_data(resp)).into_response())
    } else {
        // GetObject 分发（原文件 / imgproxy / video / style）
        let redirect_url = state
            .file_service
            .dispatch_get(&bucket, &key, query.x_oss_process.clone())
            .await?;
        Ok(Redirect::temporary(&redirect_url).into_response())
    }
}

// ============================================
// HeadObject — 获取元数据
// ============================================

/// HeadObject — 获取元数据
///
/// 返回文件元信息（在 HTTP Header 中）
#[utoipa::path(
    head,
    path = "/api/v1/oss/{bucket}/{key}",
    tag = "对象操作",
    params(
        ("bucket" = String, Path, description = "Bucket 名称"),
        ("key" = String, Path, description = "对象 Key（路径）"),
    ),
    responses(
        (status = 200, description = "元信息在 Header 中返回",
            headers(
                ("content-type" = String, description = "文件 MIME 类型"),
                ("content-length" = i64, description = "文件大小（字节）"),
                ("x-oss-meta-original-name" = String, description = "原始文件名"),
                ("x-oss-meta-scene" = String, description = "业务场景"),
                ("x-oss-meta-thumbnail-key" = String, description = "缩略图 Key"),
            )
        ),
        (status = 404, description = "文件不存在"),
    )
)]
pub async fn head_object(
    State(state): State<Arc<OssState>>,
    Path((bucket, key)): Path<(String, String)>,
) -> Result<Response, OssError> {
    let meta = state.file_service.head_object(&bucket, &key).await?;

    let mut headers = HeaderMap::new();
    if let Some(ct) = &meta.content_type {
        if let Ok(val) = ct.parse() {
            headers.insert("content-type", val);
        }
    }
    if let Some(size) = meta.size {
        if let Ok(val) = size.to_string().parse() {
            headers.insert("content-length", val);
        }
    }
    if let Some(name) = &meta.original_name {
        if let Ok(val) = name.parse() {
            headers.insert("x-oss-meta-original-name", val);
        }
    }
    if let Some(scene) = &meta.scene {
        if let Ok(val) = scene.parse() {
            headers.insert("x-oss-meta-scene", val);
        }
    }
    if let Some(thumb) = &meta.thumbnail_key {
        if let Ok(val) = thumb.parse() {
            headers.insert("x-oss-meta-thumbnail-key", val);
        }
    }

    Ok((StatusCode::OK, headers).into_response())
}

// ============================================
// DeleteObject — 删除文件 / AbortMultipart
// ============================================

/// DeleteObject Query 参数
#[derive(Debug, Deserialize, Default, IntoParams)]
pub struct DeleteObjectQuery {
    /// 分片上传 ID（AbortMultipartUpload）
    #[serde(rename = "uploadId")]
    #[param(rename = "uploadId")]
    pub upload_id: Option<String>,
}

/// DeleteObject — 删除文件 / AbortMultipart
///
/// - 无 query → 软删除文件（status=2）
/// - `?uploadId=xxx` → 取消分片上传
#[utoipa::path(
    delete,
    path = "/api/v1/oss/{bucket}/{key}",
    tag = "对象操作",
    params(
        ("bucket" = String, Path, description = "Bucket 名称"),
        ("key" = String, Path, description = "对象 Key（路径）"),
        DeleteObjectQuery,
    ),
    responses(
        (status = 204, description = "删除成功"),
        (status = 404, description = "文件不存在"),
    )
)]
pub async fn delete_object(
    State(state): State<Arc<OssState>>,
    Path((bucket, key)): Path<(String, String)>,
    Query(query): Query<DeleteObjectQuery>,
) -> Result<Response, OssError> {
    if let Some(upload_id) = query.upload_id {
        // AbortMultipartUpload
        state
            .file_service
            .abort_multipart(&bucket, &key, &upload_id)
            .await?;
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        // DeleteObject
        state.file_service.delete_object(&bucket, &key).await?;
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}
