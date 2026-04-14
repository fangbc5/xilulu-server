use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

/// 员工实体
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "employee", pk = "id", soft_delete = "is_deleted")]
pub struct Employee {
    /// 主键ID
    pub id: Option<i64>,
    /// 租户ID
    pub tenant_id: i64,
    /// 所属组织ID
    pub org_id: i64,
    /// 关联的用户ID（ms-identity.user.id）
    pub user_id: i64,
    /// 员工工号
    pub employee_no: String,
    /// 员工姓名
    pub name: String,
    /// 员工头像
    pub avatar: Option<String>,
    /// 性别：0-未知 1-男 2-女
    pub gender: Option<i16>,
    /// 工作手机
    pub mobile: Option<String>,
    /// 工作邮箱
    pub email: Option<String>,
    /// 入职日期
    pub hire_date: Option<NaiveDate>,
    /// 离职日期
    pub leave_date: Option<NaiveDate>,
    /// 状态：0-离职 1-在职 2-试用期 3-停薪留职
    pub status: Option<i16>,
    /// 排序
    pub sort_order: Option<i32>,
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

/// 员工-部门关系实体
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "employee_department", pk = "id")]
pub struct EmployeeDepartment {
    /// 主键ID
    pub id: Option<i64>,
    /// 租户ID
    pub tenant_id: i64,
    /// 员工ID
    pub employee_id: i64,
    /// 部门ID
    pub department_id: i64,
    /// 是否主部门：0-否 1-是
    pub is_primary: Option<i16>,
    /// 是否部门负责人：0-否 1-是
    pub is_leader: Option<i16>,
    /// 加入部门日期
    pub join_date: Option<NaiveDate>,
    /// 离开部门日期
    pub leave_date: Option<NaiveDate>,
    /// 创建人
    pub created_by: Option<i64>,
    /// 创建时间
    pub created_at: Option<DateTime<Utc>>,
}

/// 员工-岗位关系实体
#[derive(Debug, Clone, Default, Serialize, Deserialize, sqlx::FromRow, sqlxplus::ModelMeta, sqlxplus::CRUD)]
#[model(table = "employee_position", pk = "id")]
pub struct EmployeePosition {
    /// 主键ID
    pub id: Option<i64>,
    /// 租户ID
    pub tenant_id: i64,
    /// 员工ID
    pub employee_id: i64,
    /// 岗位ID
    pub position_id: i64,
    /// 是否主岗位：0-否 1-是
    pub is_primary: Option<i16>,
    /// 任职开始日期
    pub start_date: Option<NaiveDate>,
    /// 任职结束日期
    pub end_date: Option<NaiveDate>,
    /// 创建人
    pub created_by: Option<i64>,
    /// 创建时间
    pub created_at: Option<DateTime<Utc>>,
}
