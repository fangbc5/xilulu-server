use fbc_starter::AppState;
use sqlxplus::DbPool;
use std::sync::Arc;

pub struct ProcessState {
    pub fbc_app_state: Arc<AppState>,
    pub db: Arc<DbPool>,
    pub task_repo: Arc<crate::modules::task::repository::TaskRepository>,
}
