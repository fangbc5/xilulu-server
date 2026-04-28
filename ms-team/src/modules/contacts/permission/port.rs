use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

/// 当前查看者身份信息（从请求上下文中构建）
#[derive(Debug, Clone)]
pub struct ContactsViewer {
    /// 用户 ID
    pub user_id: i64,
    /// 租户 ID
    pub tenant_id: i64,
    /// 组织 ID
    pub org_id: i64,
    /// 员工 ID（可选）
    pub employee_id: Option<i64>,
    /// 查看者所属部门 ID 列表
    /// Phase 1 中为空（DefaultPermission 不使用）
    /// Phase 2+ 中从缓存加载（不在每次请求时查 DB）
    pub department_ids: Vec<i64>,
    /// 是否管理员
    pub is_admin: bool,
}

/// 字段限制策略
#[derive(Debug, Clone)]
pub struct FieldRestrictions {
    /// 手机号
    pub mobile: FieldAction,
    /// 邮箱
    pub email: FieldAction,
    /// 入职日期
    pub hire_date: FieldAction,
    /// 工号
    pub employee_no: FieldAction,
}

impl FieldRestrictions {
    /// 所有字段可见（Phase 1 默认策略）
    pub fn all_visible() -> Self {
        Self {
            mobile: FieldAction::Visible,
            email: FieldAction::Visible,
            hire_date: FieldAction::Visible,
            employee_no: FieldAction::Visible,
        }
    }
}

/// 字段操作类型
#[derive(Debug, Clone, PartialEq)]
pub enum FieldAction {
    /// 完整展示
    Visible,
    /// 脱敏（如 138****8888）
    Masked,
    /// 完全隐藏（返回 null）
    Hidden,
}

/// 通讯录权限引擎
/// 所有通讯录 API 在查询原始数据后、组装响应前，经过此引擎进行可见性过滤
#[async_trait]
pub trait ContactsPermission: Send + Sync {
    // ==================== Layer 1: 部门可见性 ====================

    /// 批量过滤部门列表，返回当前用户可见的部门 ID 集合
    async fn filter_visible_departments(
        &self,
        viewer: &ContactsViewer,
        department_ids: &[i64],
    ) -> HashSet<i64>;

    /// 判断单个部门是否对当前用户可见
    async fn is_department_visible(
        &self,
        viewer: &ContactsViewer,
        department_id: i64,
    ) -> bool;

    // ==================== Layer 2: 人员可见性 ====================

    /// 批量过滤员工列表，返回当前用户可见的员工 ID 集合
    async fn filter_visible_employees(
        &self,
        viewer: &ContactsViewer,
        employee_ids: &[i64],
    ) -> HashSet<i64>;

    /// 判断单个员工是否对当前用户可见
    async fn is_employee_visible(
        &self,
        viewer: &ContactsViewer,
        target_employee_id: i64,
    ) -> bool;

    // ==================== Layer 3: 字段可见性 ====================

    /// 返回当前用户查看目标员工时的字段限制（单条）
    async fn get_field_restrictions(
        &self,
        viewer: &ContactsViewer,
        target_employee_id: i64,
    ) -> FieldRestrictions;

    /// 批量获取字段限制（用于列表场景，避免 N 次调用）
    async fn get_batch_field_restrictions(
        &self,
        viewer: &ContactsViewer,
        target_employee_ids: &[i64],
    ) -> HashMap<i64, FieldRestrictions>;

    // ==================== 计数 ====================

    /// 计算某部门对当前用户可见的直属成员数
    /// Phase 1：直接返回 actual_count
    /// Phase 2+：从快照中的 hidden_count_by_dept 计算
    async fn count_visible_members(
        &self,
        viewer: &ContactsViewer,
        department_id: i64,
        actual_count: i64,
    ) -> i64;
}
