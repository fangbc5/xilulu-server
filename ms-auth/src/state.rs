use std::sync::Arc;

use fbc_starter::AppState as FbcAppState;
use sa_token_core::refresh::RefreshTokenManager;
use sa_token_plugin_axum::SaTokenState;

/// 应用状态
#[derive(Clone)]
pub struct AppState {
    pub fbc_app_state: Arc<FbcAppState>,
    pub sa_token: SaTokenState,
    /// sa-token RefreshTokenManager（管理 refresh token）
    pub refresh_token_mgr: RefreshTokenManager,
}
