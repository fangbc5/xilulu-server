use super::model::entity::{Employee, EmployeeDepartment, EmployeePosition};
use crate::error::{OrganizationError, Result};
use sqlx::{MySql, Pool};
use sqlxplus::Crud;

/// 员工 Repository
pub struct EmployeeRepo;

impl EmployeeRepo {
    /// 根据组织ID和用户ID查找员工
    pub async fn find_by_org_and_user(
        pool: &Pool<MySql>,
        org_id: i64,
        user_id: i64,
    ) -> Result<Option<Employee>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee")
            .and_eq("org_id", org_id)
            .and_eq("user_id", user_id);

        let emp = Employee::find_one(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(emp)
    }

    /// 根据组织ID和工号查找员工
    pub async fn find_by_org_and_employee_no(
        pool: &Pool<MySql>,
        org_id: i64,
        employee_no: &str,
    ) -> Result<Option<Employee>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee")
            .and_eq("org_id", org_id)
            .and_eq("employee_no", employee_no);

        let emp = Employee::find_one(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(emp)
    }

    /// 根据用户ID查找所有员工身份（跨组织）
    pub async fn find_by_user_id(pool: &Pool<MySql>, user_id: i64) -> Result<Vec<Employee>> {
        let builder =
            sqlxplus::QueryBuilder::new("SELECT * FROM employee").and_eq("user_id", user_id);

        let emps = Employee::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(emps)
    }

    /// 根据租户ID和用户ID查找员工
    pub async fn find_by_tenant_and_user(
        pool: &Pool<MySql>,
        tenant_id: i64,
        user_id: i64,
    ) -> Result<Vec<Employee>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee")
            .and_eq("tenant_id", tenant_id)
            .and_eq("user_id", user_id);

        let emps = Employee::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(emps)
    }

    /// 根据组织ID查找所有员工
    pub async fn find_by_org_id(pool: &Pool<MySql>, org_id: i64) -> Result<Vec<Employee>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee")
            .and_eq("org_id", org_id)
            .order_by("sort_order", true)
            .order_by("id", true);

        let emps = Employee::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(emps)
    }

    /// 获取员工数量
    pub async fn count_by_org_id(pool: &Pool<MySql>, org_id: i64) -> Result<i64> {
        let builder =
            sqlxplus::QueryBuilder::new("SELECT * FROM employee").and_eq("org_id", org_id);

        let count = Employee::count(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(count as i64)
    }

    /// 按月统计入职人数
    ///
    /// `start_ts` / `end_ts` 为毫秒时间戳，对应月份第一天 00:00:00 UTC。
    /// SQL 使用 `DATE_FORMAT(FROM_UNIXTIME(hire_date/1000), '%Y-%m')` 做按月分组。
    pub async fn count_hires_by_month(
        pool: &Pool<MySql>,
        org_id: i64,
        start_ts: Option<i64>,
        end_ts: Option<i64>,
    ) -> Result<Vec<(String, i64)>> {
        let mut sql = String::from(
            "SELECT DATE_FORMAT(FROM_UNIXTIME(hire_date/1000), '%Y-%m') AS month, COUNT(*) AS cnt \
             FROM employee \
             WHERE org_id = ? AND hire_date IS NOT NULL AND is_deleted = 0",
        );

        if start_ts.is_some() {
            sql.push_str(" AND hire_date >= ?");
        }
        if end_ts.is_some() {
            sql.push_str(" AND hire_date < ?");
        }
        sql.push_str(" GROUP BY month ORDER BY month");

        let mut q = sqlx::query_as::<_, (String, i64)>(&sql).bind(org_id);

        if let Some(ts) = start_ts {
            q = q.bind(ts);
        }
        if let Some(ts) = end_ts {
            q = q.bind(ts);
        }

        let rows = q
            .fetch_all(pool)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(rows)
    }
}

/// 员工部门关系 Repository
pub struct EmployeeDepartmentRepo;

impl EmployeeDepartmentRepo {
    /// 根据员工ID查找所有部门关系
    pub async fn find_by_employee_id(
        pool: &Pool<MySql>,
        employee_id: i64,
    ) -> Result<Vec<EmployeeDepartment>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee_department")
            .and_eq("employee_id", employee_id);

        let rels = EmployeeDepartment::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(rels)
    }

    /// 根据部门ID查找所有员工关系
    pub async fn find_by_department_id(
        pool: &Pool<MySql>,
        department_id: i64,
    ) -> Result<Vec<EmployeeDepartment>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee_department")
            .and_eq("department_id", department_id);

        let rels = EmployeeDepartment::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(rels)
    }

    /// 查找员工的主部门
    pub async fn find_primary_by_employee_id(
        pool: &Pool<MySql>,
        employee_id: i64,
    ) -> Result<Option<EmployeeDepartment>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee_department")
            .and_eq("employee_id", employee_id)
            .and_eq("is_primary", 1);

        let rel = EmployeeDepartment::find_one(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(rel)
    }

    /// 查找特定的员工部门关系
    pub async fn find_by_employee_and_department(
        pool: &Pool<MySql>,
        employee_id: i64,
        department_id: i64,
    ) -> Result<Option<EmployeeDepartment>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee_department")
            .and_eq("employee_id", employee_id)
            .and_eq("department_id", department_id);

        let rel = EmployeeDepartment::find_one(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(rel)
    }

    /// 清除员工的主部门标记
    pub async fn clear_primary_by_employee_id(pool: &Pool<MySql>, employee_id: i64) -> Result<()> {
        sqlx::query("UPDATE employee_department SET is_primary = 0 WHERE employee_id = ?")
            .bind(employee_id)
            .execute(pool)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 检查部门是否有员工
    pub async fn has_employees(pool: &Pool<MySql>, department_id: i64) -> Result<bool> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee_department")
            .and_eq("department_id", department_id);

        let count = EmployeeDepartment::count(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(count > 0)
    }
}

/// 员工岗位关系 Repository
pub struct EmployeePositionRepo;

impl EmployeePositionRepo {
    /// 根据员工ID查找所有岗位关系
    pub async fn find_by_employee_id(
        pool: &Pool<MySql>,
        employee_id: i64,
    ) -> Result<Vec<EmployeePosition>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee_position")
            .and_eq("employee_id", employee_id);

        let rels = EmployeePosition::find_all(pool, Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(rels)
    }

    /// 查找员工的主岗位
    pub async fn find_primary_by_employee_id(
        pool: &Pool<MySql>,
        employee_id: i64,
    ) -> Result<Option<EmployeePosition>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee_position")
            .and_eq("employee_id", employee_id)
            .and_eq("is_primary", 1);

        let rel = EmployeePosition::find_one(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(rel)
    }

    /// 查找特定的员工岗位关系
    pub async fn find_by_employee_and_position(
        pool: &Pool<MySql>,
        employee_id: i64,
        position_id: i64,
    ) -> Result<Option<EmployeePosition>> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee_position")
            .and_eq("employee_id", employee_id)
            .and_eq("position_id", position_id);

        let rel = EmployeePosition::find_one(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(rel)
    }

    /// 清除员工的主岗位标记
    pub async fn clear_primary_by_employee_id(pool: &Pool<MySql>, employee_id: i64) -> Result<()> {
        sqlx::query("UPDATE employee_position SET is_primary = 0 WHERE employee_id = ?")
            .bind(employee_id)
            .execute(pool)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 检查岗位是否有员工
    pub async fn has_employees(pool: &Pool<MySql>, position_id: i64) -> Result<bool> {
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee_position")
            .and_eq("position_id", position_id);

        let count = EmployeePosition::count(pool, builder)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        Ok(count > 0)
    }
}
