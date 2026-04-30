use crate::modules::contact::handler::contact_routes;
use crate::modules::friend::handler::friend_routes;
use crate::modules::message::handler::message_routes;
use crate::modules::room::handler::room_routes;
use crate::modules::sync::handler::sync_routes;
use crate::state::ImState;
use axum::Router;
use std::sync::Arc;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

// DTO 导入（用于 OpenAPI schemas 注册）
use crate::modules::friend::model::{
    ApplyRequest, ApplyVO, DeleteFriendRequest, FriendSearchVO, FriendVO, SearchFriendRequest,
};
use crate::modules::room::model::{
    AddMemberRequest, CreateGroupRequest, GroupMember, RoomGroup, TransferOwnerRequest,
    UpdateGroupRequest,
};
use crate::modules::contact::model::{
    ContactRequest, ContactSettingRequest, ContactVO, MarkReadRequest,
};
use crate::modules::message::model::{
    BatchLatestMessageRequest, MarkRequest, Message, MessageCursorQuery,
    SendMessageRequest,
};
use crate::modules::sync::model::{SyncRequest, SyncResponse};
use crate::modules::room::model::{Room, RoomFriend};
use crate::modules::friend::model::UserFriend;
use crate::modules::contact::model::Contact;
use crate::client::identity::UserBrief;

/// OpenAPI 文档定义
///
/// 自动收集所有标注了 `#[utoipa::path]` 的 handler
#[derive(OpenApi)]
#[openapi(
    info(
        title = "ms-im 即时通讯服务",
        version = "0.1.0",
        description = "Xilulu 即时通讯微服务 API。\n\n支持好友管理、群聊管理、会话管理、消息收发、增量同步等能力。",
        contact(name = "fangbc5@gmail.com"),
    ),
    tags(
        (name = "好友管理", description = "好友申请/同意/拒绝/删除/搜索"),
        (name = "群聊管理", description = "群聊创建/更新/成员管理/解散/转让"),
        (name = "会话管理", description = "会话列表/置顶/免打扰/已读/删除"),
        (name = "消息管理", description = "消息发送/列表/撤回/标记/批量拉取"),
        (name = "数据同步", description = "客户端增量同步"),
    ),
    paths(
        // 好友管理
        crate::modules::friend::handler::list_friends,
        crate::modules::friend::handler::delete_friend,
        crate::modules::friend::handler::apply,
        crate::modules::friend::handler::list_applies,
        crate::modules::friend::handler::approve,
        crate::modules::friend::handler::reject,
        crate::modules::friend::handler::search_user,
        // 群聊管理
        crate::modules::room::handler::create_group,
        crate::modules::room::handler::group_info,
        crate::modules::room::handler::update_group,
        crate::modules::room::handler::list_members,
        crate::modules::room::handler::add_member,
        crate::modules::room::handler::remove_member,
        crate::modules::room::handler::quit_group,
        crate::modules::room::handler::dissolve_group,
        crate::modules::room::handler::transfer_owner,
        // 会话管理
        crate::modules::contact::handler::list_contacts,
        crate::modules::contact::handler::set_top,
        crate::modules::contact::handler::set_mute,
        crate::modules::contact::handler::mark_read,
        crate::modules::contact::handler::mark_unread,
        crate::modules::contact::handler::delete_contact,
        // 消息管理
        crate::modules::message::handler::send_message,
        crate::modules::message::handler::list_messages,
        crate::modules::message::handler::recall_message,
        crate::modules::message::handler::mark_message,
        crate::modules::message::handler::batch_latest_messages,
        // 数据同步
        crate::modules::sync::handler::pull_sync,
    ),
    components(schemas(
        // 好友
        ApplyRequest,
        DeleteFriendRequest,
        SearchFriendRequest,
        FriendVO,
        ApplyVO,
        FriendSearchVO,
        UserFriend,
        // 房间/群聊
        CreateGroupRequest,
        UpdateGroupRequest,
        AddMemberRequest,
        TransferOwnerRequest,
        Room,
        RoomFriend,
        RoomGroup,
        GroupMember,
        // 会话
        ContactSettingRequest,
        ContactRequest,
        MarkReadRequest,
        ContactVO,
        Contact,
        // 消息
        SendMessageRequest,
        MessageCursorQuery,
        MarkRequest,
        BatchLatestMessageRequest,
        Message,
        // 同步
        SyncRequest,
        SyncResponse,
        UserBrief,
    ))
)]
pub struct ApiDoc;

/// 创建应用路由
pub fn create_routes(im_state: Arc<ImState>) -> Router {
    Router::new()
        // ---- Swagger UI（内嵌资源，无需 CDN）----
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api/v1/im", Router::new()
            .nest("/friends", friend_routes())
            .nest("/rooms", room_routes())
            .nest("/contacts", contact_routes())
            .nest("/messages", message_routes())
            .nest("/sync", sync_routes())
        )
        .with_state(im_state)
}
