use super::model::dto::{
    CreatePositionRequest, ListPositionsQuery, PositionResponse, UpdatePositionRequest,
};
use super::model::entity::Position;
use crate::error::OrganizationError;
use crate::middleware::CurrentUser;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::Json;
use fbc_starter::base::CursorPageBaseResp;
use fbc_starter::R;
use std::sync::Arc;
use validator::Validate;

/// 转换实体为响应
fn to_response(pos: Position) -> PositionResponse {
    PositionResponse {
        id: pos.id.unwrap_or(0),
        tenant_id: pos.tenant_id,
        org_id: pos.org_id,
        code: pos.code,
        name: pos.name,
        category: pos.category,
        level: pos.level,
        description: pos.description,
        requirements: pos.requirements,
        sort_order: pos.sort_order,
        status: pos.status,
    }
}

/// 创建岗位
pub async fn create_position(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(req): Json<CreatePositionRequest>,
) -> Result<Json<R<i64>>, OrganizationError> {
    req.validate()
        .map_err(|e| OrganizationError::ParamFormatError(e.to_string()))?;

    let id = state
        .position_service
        .create(current_user.tenant_id, req, Some(current_user.user_id))
        .await?;

    Ok(Json(R::ok_with_data(id)))
}

/// 获取岗位详情
pub async fn get_position(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<PositionResponse>>, OrganizationError> {
    let pos = state
        .position_service
        .get_by_id(id)
        .await?;

    Ok(Json(R::ok_with_data(to_response(pos))))
}

/// 获取岗位列表
pub async fn list_positions(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<ListPositionsQuery>,
) -> Result<Json<R<CursorPageBaseResp<PositionResponse>>>, OrganizationError> {
    let page = query.page.cursor.unwrap_or(1);
    let page_size = query.page.page_size;

    let (positions, total) = state
        .position_service
        .find_page(current_user.tenant_id, query)
        .await?;

    let responses: Vec<PositionResponse> = positions.into_iter().map(to_response).collect();
    let has_next = (page as i64 * page_size as i64) < total;
    let next_cursor = if has_next { Some(page + 1) } else { None };

    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(
        next_cursor,
        has_next,
        responses,
        total,
    ))))
}


/// 更新岗位
pub async fn update_position(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdatePositionRequest>,
) -> Result<Json<R<()>>, OrganizationError> {
    req.validate()
        .map_err(|e| OrganizationError::ParamFormatError(e.to_string()))?;

    state
        .position_service
        .update(id, req, Some(current_user.user_id))
        .await?;

    Ok(Json(R::ok()))
}

/// 删除岗位
pub async fn delete_position(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, OrganizationError> {
    state
        .position_service
        .delete(id)
        .await?;

    Ok(Json(R::ok()))
}
