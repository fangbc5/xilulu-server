use serde::{Deserialize, Serialize};

/// 组织实体
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "organization", pk = "id", soft_delete = "is_deleted")]
pub struct Organization {
    /// 主键ID
    pub id: Option<i64>,
    /// 租户ID
    pub tenant_id: i64,
    /// 上级组织ID
    pub parent_id: Option<i64>,
    /// 组织编码
    pub code: String,
    /// 组织名称
    pub name: String,
    /// 简称
    pub short_name: Option<String>,
    /// 组织类型：1-集团 2-公司 3-分公司 4-子公司
    #[sqlx(rename = "type")]
    #[serde(rename = "type")]
    pub r#type: Option<i16>,
    /// 组织Logo
    pub logo: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 排序
    pub sort_order: Option<i32>,
    /// 状态：0-禁用 1-启用
    pub status: Option<i16>,
    /// 创建人
    pub created_by: Option<i64>,
    /// 创建时间（毫秒时间戳）
    pub created_at: Option<i64>,
    /// 更新人
    pub updated_by: Option<i64>,
    /// 更新时间（毫秒时间戳）
    pub updated_at: Option<i64>,
    /// 是否删除
    pub is_deleted: Option<i16>,
}
