// 设备模块 HTTP 处理器

use crate::context::RequestContext;
use crate::error::{IdentityError, to_err};
use crate::modules::device::model::dto::{DeviceInfo, RegisterDeviceRequest, UnregisterDeviceRequest};
use crate::state::AppState;
use axum::{extract::State, response::Json};
use fbc_starter::R;
use std::sync::Arc;

/// 注册/更新设备推送 Token
pub async fn register_device(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::Json(req): axum::Json<RegisterDeviceRequest>,
) -> Result<Json<R<i64>>, IdentityError> {
    let uid = context.user_id;
    let device_id = app_state
        .device_service
        .register_device(uid, &req)
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok_with_data(device_id)))
}

/// 注销设备
pub async fn unregister_device(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
    axum::Json(req): axum::Json<UnregisterDeviceRequest>,
) -> Result<Json<R<()>>, IdentityError> {
    let uid = context.user_id;
    app_state
        .device_service
        .unregister_device(uid, &req.client_id)
        .await
        .map_err(to_err)?;
    Ok(Json(R::ok()))
}

/// 获取当前用户的所有有效设备
pub async fn get_my_devices(
    State(app_state): State<Arc<AppState>>,
    context: RequestContext,
) -> Result<Json<R<Vec<DeviceInfo>>>, IdentityError> {
    let uid = context.user_id;
    let devices = app_state
        .device_service
        .get_active_devices(uid)
        .await
        .map_err(to_err)?;
    let list: Vec<DeviceInfo> = devices.into_iter().map(DeviceInfo::from).collect();
    Ok(Json(R::ok_with_data(list)))
}
