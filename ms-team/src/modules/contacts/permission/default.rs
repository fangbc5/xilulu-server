use async_trait::async_trait;
use std::collections::{HashMap, HashSet};

use super::port::{ContactsPermission, ContactsViewer, FieldRestrictions};

/// 默认权限实现 — Phase 1 全开放策略
/// 所有部门可见、所有人员可见、所有字段可见
/// 所有方法为纯内存操作，无 I/O 开销
pub struct DefaultPermission;

#[async_trait]
impl ContactsPermission for DefaultPermission {
    async fn filter_visible_departments(
        &self,
        _viewer: &ContactsViewer,
        department_ids: &[i64],
    ) -> HashSet<i64> {
        department_ids.iter().cloned().collect()
    }

    async fn is_department_visible(
        &self,
        _viewer: &ContactsViewer,
        _department_id: i64,
    ) -> bool {
        true
    }

    async fn filter_visible_employees(
        &self,
        _viewer: &ContactsViewer,
        employee_ids: &[i64],
    ) -> HashSet<i64> {
        employee_ids.iter().cloned().collect()
    }

    async fn is_employee_visible(
        &self,
        _viewer: &ContactsViewer,
        _target_employee_id: i64,
    ) -> bool {
        true
    }

    async fn get_field_restrictions(
        &self,
        _viewer: &ContactsViewer,
        _target_employee_id: i64,
    ) -> FieldRestrictions {
        FieldRestrictions::all_visible()
    }

    async fn get_batch_field_restrictions(
        &self,
        _viewer: &ContactsViewer,
        target_employee_ids: &[i64],
    ) -> HashMap<i64, FieldRestrictions> {
        target_employee_ids
            .iter()
            .map(|id| (*id, FieldRestrictions::all_visible()))
            .collect()
    }

    async fn count_visible_members(
        &self,
        _viewer: &ContactsViewer,
        _department_id: i64,
        actual_count: i64,
    ) -> i64 {
        actual_count
    }
}
