/// 枚举模块
///
/// 包含所有业务枚举定义

pub mod apply_deleted;
pub mod apply_read_status;
pub mod black_type;
pub mod feed;
pub mod feed_permission;
pub mod group_role;
pub mod group_role_app;
pub mod hot_flag;
pub mod item;
pub mod item_type;
pub mod merge_type;
pub mod message_mark_act_type;
pub mod message_status;
pub mod message_type;
pub mod notice_status;
pub mod notice_type;
pub mod oss_scene;
pub mod role_type;
pub mod room_type;

// 重新导出所有枚举
pub use apply_deleted::ApplyDeletedEnum;
pub use apply_read_status::ApplyReadStatusEnum;
pub use black_type::BlackTypeEnum;
pub use feed::FeedEnum;
pub use feed_permission::FeedPermissionEnum;
pub use group_role::GroupRoleEnum;
pub use group_role_app::GroupRoleAPPEnum;
pub use hot_flag::HotFlagEnum;
pub use item::ItemEnum;
pub use item_type::ItemTypeEnum;
pub use merge_type::MergeTypeEnum;
pub use message_mark_act_type::MessageMarkActTypeEnum;
pub use message_status::MessageStatusEnum;
pub use message_type::MessageTypeEnum;
pub use notice_status::NoticeStatusEnum;
pub use notice_type::NoticeTypeEnum;
pub use oss_scene::OssSceneEnum;
pub use role_type::RoleTypeEnum;
pub use room_type::RoomTypeEnum;

