use serde::Deserialize;
use utoipa::ToSchema;

/// 创建群聊请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateGroupRequest {
    pub name: String,
    pub member_uids: Vec<i64>,
}

/// 添加群成员请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct AddMemberRequest {
    /// 被添加的目标用户 ID
    pub uid: i64,
}

/// 更新群信息请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub notice: Option<String>,
}

/// 转让群主请求
#[derive(Debug, Deserialize, ToSchema)]
pub struct TransferOwnerRequest {
    /// 新群主的用户 ID
    pub new_owner_uid: i64,
}
