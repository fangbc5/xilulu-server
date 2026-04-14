use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 部门实体
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "department", pk = "id", soft_delete = "is_deleted")]
pub struct Department {
    /// 主键ID
    pub id: Option<i64>,
    /// 租户ID
    pub tenant_id: i64,
    /// 所属组织ID
    pub org_id: i64,
    /// 上级部门ID
    pub parent_id: Option<i64>,
    /// 部门编码
    pub code: String,
    /// 部门名称
    pub name: String,
    /// 部门全称
    pub full_name: Option<String>,
    /// 部门路径（如：/1/2/3/）
    pub path: Option<String>,
    /// 层级深度
    pub level: Option<i32>,
    /// 部门负责人（员工ID）
    pub leader_employee_id: Option<i64>,
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
