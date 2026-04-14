use axum::extract::{Path, State};
use axum::Json;
use fbc_starter::{R, RequestContext};
use std::sync::Arc;

use super::model::dto::*;
use crate::error::OssError;
use crate::state::OssState;

/// 预签名上传
///
/// POST /oss/presign/upload
pub async fn presign_upload(
    State(state): State<Arc<OssState>>,
    ctx: Option<RequestContext>,
    Json(req): Json<PresignUploadRequest>,
) -> Result<Json<R<PresignUploadResponse>>, OssError> {
    let uploader_id = ctx.map(|c| c.user_id);
    let resp = state.file_service.presign_upload(req, uploader_id).await
        .map_err(|e| OssError::PresignFailed(e.to_string()))?;
    Ok(Json(R::ok_with_data(resp)))
}

/// 预签名下载
///
/// POST /oss/presign/download
pub async fn presign_download(
    State(state): State<Arc<OssState>>,
    Json(req): Json<PresignDownloadRequest>,
) -> Result<Json<R<PresignDownloadResponse>>, OssError> {
    let resp = state.file_service.presign_download(req).await
        .map_err(|e| OssError::PresignFailed(e.to_string()))?;
    Ok(Json(R::ok_with_data(resp)))
}

/// 上传完成回调
///
/// POST /oss/callback
pub async fn upload_callback(
    State(state): State<Arc<OssState>>,
    Json(req): Json<UploadCallbackRequest>,
) -> Result<Json<R<FileMetaResponse>>, OssError> {
    let resp = state.file_service.upload_callback(req).await
        .map_err(|e| OssError::CallbackFailed(e.to_string()))?;
    Ok(Json(R::ok_with_data(resp)))
}

/// 查询文件元数据
///
/// GET /oss/files/:id
pub async fn get_file(
    State(state): State<Arc<OssState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<FileMetaResponse>>, OssError> {
    let resp = state.file_service.get_file_meta(id).await
        .map_err(|e| OssError::InternalError(e.to_string()))?
        .ok_or_else(|| OssError::FileNotFound(format!("文件不存在: {}", id)))?;
    Ok(Json(R::ok_with_data(resp)))
}

/// 删除文件
///
/// DELETE /oss/files/:id
pub async fn delete_file(
    State(state): State<Arc<OssState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<String>>, OssError> {
    match state.file_service.delete_file(id).await {
        Ok(Some(())) => Ok(Json(R::ok())),
        Ok(None) => Err(OssError::FileNotFound(format!("文件不存在: {}", id))),
        Err(e) => Err(OssError::InternalError(e.to_string())),
    }
}
