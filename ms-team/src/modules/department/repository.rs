use super::model::entity::Department;
use crate::error::{OrganizationError, Result};
use sqlx::{MySql, Pool};
use sqlxplus::Crud;

/// 部门 Repository
pub struct DepartmentRepo;

impl DepartmentRepo {
    /// 根据组织ID和编码查找部门
    pub async fn find_by_org_and_code(
        pool: &Pool<MySql>,
        org_id: i64,
        code: &str,
    ) -> Result<Option<Department>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM department")
            .and_eq("org_id", org_id)
            .and_eq("code", code);

        let dept = Department::find_one(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(dept)
    }

    /// 根据组织ID查找所有部门
    pub async fn find_by_org_id(pool: &Pool<MySql>, org_id: i64) -> Result<Vec<Department>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM department")
            .and_eq("org_id", org_id)
            .order_by("sort_order", true)
            .order_by("id", true);

        let depts = Department::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(depts)
    }

    /// 根据上级部门ID查找子部门
    pub async fn find_by_parent_id(pool: &Pool<MySql>, parent_id: i64) -> Result<Vec<Department>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM department")
            .and_eq("parent_id", parent_id)
            .order_by("sort_order", true)
            .order_by("id", true);

        let depts = Department::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(depts)
    }

    /// 根据组织ID查找顶级部门
    pub async fn find_root_by_org_id(pool: &Pool<MySql>, org_id: i64) -> Result<Vec<Department>> {
        // sqlxplus 会自动处理 is_deleted
        // 注意：这里手动添加了 WHERE 子句，sqlxplus 应该会正确追加 AND is_deleted = 0
        let builder =
            sqlxplus::QueryBuilder::new("SELECT * FROM department WHERE parent_id IS NULL")
                .and_eq("org_id", org_id)
                .order_by("sort_order", true)
                .order_by("id", true);

        let depts = Department::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(depts)
    }

    /// 检查是否有子部门
    pub async fn has_children(pool: &Pool<MySql>, dept_id: i64) -> Result<bool> {
        let builder =
            sqlxplus::QueryBuilder::new("SELECT * FROM department").and_eq("parent_id", dept_id);

        let count = Department::count(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(count > 0)
    }

    /// 根据路径前缀查找所有子部门（包括孙子部门）
    pub async fn find_by_path_prefix(
        pool: &Pool<MySql>,
        path_prefix: &str,
    ) -> Result<Vec<Department>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM department")
            .and_like("path", format!("{}%", path_prefix))
            .order_by("path", true);

        let depts = Department::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(depts)
    }

    /// 获取部门数量
    pub async fn count_by_org_id(pool: &Pool<MySql>, org_id: i64) -> Result<i64> {
        let builder =
            sqlxplus::QueryBuilder::new("SELECT * FROM department").and_eq("org_id", org_id);

        let count = Department::count(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(count as i64)
    }

    /// 统计某部门及其所有下属部门的员工总数
    /// 利用 path 索引快速查询所有下属部门，然后统计员工数
    pub async fn count_employees_by_dept_id(
        pool: &Pool<MySql>,
        _dept_id: i64,
        dept_path: &str,
        tenant_id: i64,
    ) -> Result<i64> {
        // 获取该部门及所有下属部门的 ID
        // 利用 path = '/123/' 查询得到路径为 /123/* 的所有部门
        let path_prefix = format!("{}%", dept_path.trim_end_matches('/'));

        let sql = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(DISTINCT ed.employee_id)
            FROM employee_department ed
            INNER JOIN department d ON ed.dept_id = d.id
            WHERE d.path LIKE ?
              AND ed.status = 1
              AND d.tenant_id = ?
            "#,
        )
        .bind(&path_prefix)
        .bind(tenant_id);

        let count = sql
            .fetch_one(pool)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(count)
    }

    /// 统计部门的直属员工数
    pub async fn count_direct_employees(pool: &Pool<MySql>, dept_id: i64) -> Result<i64> {
        let sql = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM employee_department
            WHERE dept_id = ? AND status = 1
            "#,
        )
        .bind(dept_id);

        let count = sql
            .fetch_one(pool)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(count)
    }
}
