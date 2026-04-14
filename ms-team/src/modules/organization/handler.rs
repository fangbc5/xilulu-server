use super::model::dto::{
    CreateOrganizationRequest, ListOrganizationsQuery, OrganizationResponse, OrganizationTreeNode,
    UpdateOrganizationRequest,
};
use super::model::entity::Organization;
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
fn to_response(org: Organization) -> OrganizationResponse {
    OrganizationResponse {
        id: org.id.unwrap_or(0),
        tenant_id: org.tenant_id,
        parent_id: org.parent_id,
        code: org.code,
        name: org.name,
        short_name: org.short_name,
        r#type: org.r#type,
        logo: org.logo,
        description: org.description,
        sort_order: org.sort_order,
        status: org.status,
    }
}

/// 创建组织
pub async fn create_organization(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Json(req): Json<CreateOrganizationRequest>,
) -> Result<Json<R<i64>>, OrganizationError> {
    req.validate()
        .map_err(|e| OrganizationError::ParamFormatError(e.to_string()))?;

    let id = state
        .organization_service
        .create(current_user.tenant_id, req, Some(current_user.user_id))
        .await?;

    Ok(Json(R::ok_with_data(id)))
}

/// 获取组织详情
pub async fn get_organization(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<OrganizationResponse>>, OrganizationError> {
    let org = state
        .organization_service
        .get_by_id(id)
        .await?;

    Ok(Json(R::ok_with_data(to_response(org))))
}

/// 获取组织树
pub async fn get_organization_tree(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
) -> Result<Json<R<Vec<OrganizationTreeNode>>>, OrganizationError> {
    let tree = state
        .organization_service
        .get_tree(current_user.tenant_id)
        .await?;

    Ok(Json(R::ok_with_data(tree)))
}

/// 获取组织列表
pub async fn list_organizations(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Query(query): Query<ListOrganizationsQuery>,
) -> Result<Json<R<CursorPageBaseResp<OrganizationResponse>>>, OrganizationError> {
    // 记录分页参数用于计算响应
    let page = query.page.cursor.unwrap_or(1);
    let page_size = query.page.page_size;

    let (orgs, total) = state
        .organization_service
        .find_page(current_user.tenant_id, query)
        .await?;

    let responses: Vec<OrganizationResponse> = orgs.into_iter().map(to_response).collect();

    // 计算游标和是否最后一页
    // 这里假设 cursor 是页码
    let has_next = (page as i64 * page_size as i64) < total;
    let next_cursor = if has_next { Some(page + 1) } else { None };

    Ok(Json(R::ok_with_data(CursorPageBaseResp::init(
        next_cursor,
        has_next,
        responses,
        total,
    ))))
}

/// 更新组织
pub async fn update_organization(
    State(state): State<Arc<AppState>>,
    current_user: CurrentUser,
    Path(id): Path<i64>,
    Json(req): Json<UpdateOrganizationRequest>,
) -> Result<Json<R<()>>, OrganizationError> {
    req.validate()
        .map_err(|e| OrganizationError::ParamFormatError(e.to_string()))?;

    state
        .organization_service
        .update(id, req, Some(current_user.user_id))
        .await?;

    Ok(Json(R::ok()))
}

/// 删除组织
pub async fn delete_organization(
    State(state): State<Arc<AppState>>,
    Path(id): Path<i64>,
) -> Result<Json<R<()>>, OrganizationError> {
    state
        .organization_service
        .delete(id)
        .await?;

    Ok(Json(R::ok()))
}
