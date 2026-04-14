use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 岗位实体
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "position", pk = "id", soft_delete = "is_deleted")]
pub struct Position {
    /// 主键ID
    pub id: Option<i64>,
    /// 租户ID
    pub tenant_id: i64,
    /// 所属组织ID
    pub org_id: i64,
    /// 岗位编码
    pub code: String,
    /// 岗位名称
    pub name: String,
    /// 岗位类别
    pub category: Option<String>,
    /// 岗位级别
    pub level: Option<i32>,
    /// 岗位职责描述
    pub description: Option<String>,
    /// 任职要求
    pub requirements: Option<String>,
    /// 排序
    pub sort_order: Option<i32>,
    /// 状态：0-禁用 1-启用
    pub status: Option<i16>,
    /// 创建人
    pub created_by: Option<i64>,
    /// 创建时间
    pub created_at: Option<DateTime<Utc>>,
    /// 更新人
    pub updated_by: Option<i64>,
    /// 更新时间
    pub updated_at: Option<DateTime<Utc>>,
    /// 是否删除
    pub is_deleted: Option<i16>,
}
