use std::sync::Arc;
use axum::routing::{delete, get, post};
use axum::Router;
use crate::state::OssState;
use crate::modules::file::handler;

/// 创建路由
pub fn create_router(state: Arc<OssState>) -> Router {
    Router::new()
        .nest(
            "/api/v1",
            Router::new()
                // 预签名接口
                .nest(
                    "/oss/presign",
                    Router::new()
                        .route("/upload", post(handler::presign_upload))
                        .route("/download", post(handler::presign_download)),
                )
                // 上传回调
                .route("/oss/callback", post(handler::upload_callback))
                // 文件元数据 CRUD
                .nest(
                    "/oss/files",
                    Router::new()
                        .route("/{id}", get(handler::get_file))
                        .route("/{id}", delete(handler::delete_file)),
                ),
        )
        .with_state(state)
}
