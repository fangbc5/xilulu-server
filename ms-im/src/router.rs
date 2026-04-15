use crate::modules::contact::handler::contact_routes;
use crate::modules::friend::handler::friend_routes;
use crate::modules::message::handler::message_routes;
use crate::modules::room::handler::room_routes;
use crate::modules::sync::handler::sync_routes;
use crate::state::ImState;
use axum::Router;
use std::sync::Arc;

/// 创建应用路由
pub fn create_routes(im_state: Arc<ImState>) -> Router {
    Router::new()
        .nest("/api/v1/im", Router::new()
            .nest("/friends", friend_routes())
            .nest("/rooms", room_routes())
            .nest("/contacts", contact_routes())
            .nest("/messages", message_routes())
            .nest("/sync", sync_routes())
        )
        .with_state(im_state)
}
