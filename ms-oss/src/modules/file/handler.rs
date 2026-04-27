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

use super::model::dto::*;
use crate::error::OssError;
use crate::state::OssState;

// ============================================
// 签名服务
// ============================================

/// 统一签名服务
///
/// POST /oss/signature
/// 根据 method 字段分发：put → 上传签名、get → 下载签名、share → 长效分享
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
///
/// GET /oss/share/{token}
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
/// PUT /oss/{bucket}/*key
/// 校验 scene 规则 → 生成预签名 URL → 返回
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
#[derive(Debug, Deserialize, Default)]
pub struct PostObjectQuery {
    /// 存在则为初始化分片上传
    pub uploads: Option<String>,
    /// 存在则为完成分片上传
    #[serde(rename = "uploadId")]
    pub upload_id: Option<String>,
}

/// PostObject — 上传确认 / 分片上传系列
///
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
                urlencoding::decode(s).map(|c| c.into_owned()).unwrap_or_else(|_| s.to_string())
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
#[derive(Debug, Deserialize, Default)]
pub struct GetObjectQuery {
    /// 图片处理参数
    #[serde(rename = "x-oss-process")]
    pub x_oss_process: Option<String>,
    /// 分片上传 ID（ListParts）
    #[serde(rename = "uploadId")]
    pub upload_id: Option<String>,
}

/// GetObject — 下载 / 图片处理 / 视频产物 / ListParts
///
/// GET /oss/{bucket}/*key
/// - 无 query → 原文件下载（302 → S3）
/// - ?x-oss-process=image/... → 图片实时处理（302 → imgproxy）
/// - ?x-oss-process=video/... → 视频截帧产物
/// - ?x-oss-process=style/... → Style 预设
/// - ?uploadId=xxx → ListParts
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
/// HEAD /oss/{bucket}/*key
/// 返回文件元信息（在 Header 中）
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
#[derive(Debug, Deserialize, Default)]
pub struct DeleteObjectQuery {
    /// 分片上传 ID（AbortMultipartUpload）
    #[serde(rename = "uploadId")]
    pub upload_id: Option<String>,
}

/// DeleteObject — 删除文件 / AbortMultipart
///
/// DELETE /oss/{bucket}/*key
/// - 无 query → 删除文件
/// - ?uploadId=xxx → 取消分片上传
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
