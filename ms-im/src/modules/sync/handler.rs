use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use std::sync::Arc;

use fbc_starter::{RequestContext, R};
use crate::error::ImError;
use crate::state::ImState;

use super::model::{SyncRequest, SyncResponse};

/// 增量同步 GET /api/v1/im/sync?since_ts=1700000000000
async fn pull_sync(
    State(state): State<Arc<ImState>>,
    context: RequestContext,
    Query(req): Query<SyncRequest>,
) -> Result<Json<R<SyncResponse>>, ImError> {
    let resp = state.sync_service.pull_sync(context.user_id, &req).await?;
    Ok(Json(R::ok_with_data(resp)))
}

/// 同步模块路由
pub fn sync_routes() -> Router<Arc<ImState>> {
    Router::new().route("/", get(pull_sync))
}
