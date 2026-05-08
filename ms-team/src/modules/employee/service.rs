use super::model::dto::{
    CreateEmployeeRequest, EmployeeDepartmentResponse, EmployeePositionResponse,
    HireStatsQuery, ListEmployeesQuery, MonthlyCount, UpdateEmployeeRequest,
};
use super::model::entity::{Employee, EmployeeDepartment, EmployeePosition};
use super::repository::{EmployeeDepartmentRepo, EmployeePositionRepo, EmployeeRepo};
use crate::error::{OrganizationError, Result};
use crate::modules::department::Department;
use crate::modules::position::Position;
use crate::modules::organization::Organization;
use sqlxplus::Crud;
use sqlxplus::DbPool;
use sqlxplus::QueryBuilder;
use std::sync::Arc;

/// 员工 Service
pub struct EmployeeService {
    db_pool: Arc<DbPool>,
    department_service: Option<Arc<crate::modules::department::DepartmentService>>,
}

impl EmployeeService {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self {
            db_pool,
            department_service: None,
        }
    }

    /// 设置部门服务（用于缓存失效）
    pub fn with_department_service(
        mut self,
        department_service: Arc<crate::modules::department::DepartmentService>,
    ) -> Self {
        self.department_service = Some(department_service);
        self
    }

    /// 分页查询员工
    pub async fn find_page(
        &self,
        tenant_id: i64,
        req: ListEmployeesQuery,
    ) -> Result<(Vec<Employee>, i64)> {
        use sqlxplus::QueryBuilder;

        let mut builder = QueryBuilder::new("SELECT * FROM employee")
            .and_eq("tenant_id", tenant_id)
            .and_eq("org_id", req.org_id);

        if let Some(status) = req.status {
            builder = builder.and_eq("status", status);
        }

        if let Some(ref keyword) = req.keyword {
            if !keyword.is_empty() {
                builder = builder.and_group(|mut g| {
                    g = g.or_like("name", format!("%{}%", keyword));
                    g = g.or_like("employee_no", format!("%{}%", keyword));
                    g = g.or_like("mobile", format!("%{}%", keyword));
                    g
                });
            }
        }

        // Handle department_id filtering
        if let Some(dept_id) = req.department_id {
            let ids: Vec<i64> = if req.include_children.unwrap_or(false) {
                // Need to find department to get path
                let dept = Department::find_by_id(self.db_pool.mysql_pool(), dept_id)
                    .await
                    .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

                if let Some(d) = dept {
                    if let Some(path) = d.path {
                        sqlx::query_scalar("SELECT ed.employee_id FROM employee_department ed JOIN department d ON ed.department_id = d.id WHERE d.path LIKE ?")
                            .bind(format!("{}%", path))
                            .fetch_all(self.db_pool.mysql_pool())
                            .await
                            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
                    } else {
                        vec![]
                    }
                } else {
                    vec![]
                }
            } else {
                sqlx::query_scalar(
                    "SELECT employee_id FROM employee_department WHERE department_id = ?",
                )
                .bind(dept_id)
                .fetch_all(self.db_pool.mysql_pool())
                .await
                .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
            };

            if ids.is_empty() {
                return Ok((vec![], 0));
            }
            builder = builder.and_in("id", ids);
        }

        // Handle position_id
        if let Some(pos_id) = req.position_id {
            let ids: Vec<i64> = sqlx::query_scalar(
                "SELECT employee_id FROM employee_position WHERE position_id = ?",
            )
            .bind(pos_id)
            .fetch_all(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

            if ids.is_empty() {
                return Ok((vec![], 0));
            }
            builder = builder.and_in("id", ids);
        }

        builder = builder.order_by("sort_order", true).order_by("id", true);

        let page_num = req.page.cursor.unwrap_or(1);
        let page_size = req.page.page_size;

        let result = Employee::paginate(self.db_pool.mysql_pool(), builder, page_num, page_size)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok((result.items, result.total as i64))
    }

    /// 创建员工
    pub async fn create(
        &self,
        tenant_id: i64,
        req: CreateEmployeeRequest,
        created_by: Option<i64>,
    ) -> Result<i64> {
        // 检查用户是否已是该组织员工
        if let Some(_) =
            EmployeeRepo::find_by_org_and_user(self.db_pool.mysql_pool(), req.org_id, req.user_id)
                .await?
        {
            return Err(OrganizationError::UserAlreadyEmployee.into());
        }

        // 检查工号是否已存在
        if let Some(_) = EmployeeRepo::find_by_org_and_employee_no(
            self.db_pool.mysql_pool(),
            req.org_id,
            &req.employee_no,
        )
        .await?
        {
            return Err(OrganizationError::EmployeeNoExists(req.employee_no).into());
        }

        // ✏️ 检查组织是否存在
        if let None = Organization::find_by_id(self.db_pool.mysql_pool(), req.org_id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
        {
            return Err(OrganizationError::OrganizationNotFound.into());
        }

        let now = chrono::Utc::now().timestamp_millis();
        let mut emp = Employee::default();
        emp.tenant_id = tenant_id;
        emp.org_id = req.org_id;
        emp.user_id = req.user_id;
        emp.employee_no = req.employee_no;
        emp.name = req.name;
        emp.avatar = req.avatar;
        emp.gender = req.gender;
        emp.mobile = req.mobile;
        emp.email = req.email;
        emp.hire_date = req.hire_date;
        emp.status = Some(1); // 默认在职
        emp.sort_order = Some(0);
        emp.created_by = created_by;
        emp.created_at = Some(now);
        emp.updated_at = Some(now);

        let employee_id = emp
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        // 如果指定了主部门，添加部门关系
        if let Some(dept_id) = req.primary_department_id {
            let mut dept_rel = EmployeeDepartment::default();
            dept_rel.tenant_id = tenant_id;
            dept_rel.employee_id = employee_id;
            dept_rel.department_id = dept_id;
            dept_rel.is_primary = Some(1);
            dept_rel.is_leader = Some(0);
            dept_rel.created_by = created_by;
            dept_rel.created_at = Some(now);

            dept_rel
                .insert(self.db_pool.mysql_pool())
                .await
                .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        }

        // 如果指定了主岗位，添加岗位关系
        if let Some(pos_id) = req.primary_position_id {
            let mut pos_rel = EmployeePosition::default();
            pos_rel.tenant_id = tenant_id;
            pos_rel.employee_id = employee_id;
            pos_rel.position_id = pos_id;
            pos_rel.is_primary = Some(1);
            pos_rel.created_by = created_by;
            pos_rel.created_at = Some(now);

            pos_rel
                .insert(self.db_pool.mysql_pool())
                .await
                .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        }

        Ok(employee_id)
    }

    /// 获取员工详情
    pub async fn get_by_id(&self, id: i64) -> Result<Employee> {
        let emp = Employee::find_by_id(self.db_pool.mysql_pool(), id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        emp.ok_or_else(|| OrganizationError::EmployeeNotFound.into())
    }

    /// 根据用户ID和租户ID获取员工
    pub async fn get_by_user_and_tenant(
        &self,
        user_id: i64,
        tenant_id: i64,
    ) -> Result<Vec<Employee>> {
        EmployeeRepo::find_by_tenant_and_user(self.db_pool.mysql_pool(), tenant_id, user_id).await
    }

    /// 根据用户ID和组织ID获取员工
    pub async fn get_by_user_and_org(&self, user_id: i64, org_id: i64) -> Result<Option<Employee>> {
        EmployeeRepo::find_by_org_and_user(self.db_pool.mysql_pool(), org_id, user_id).await
    }

    /// 获取组织下的所有员工
    pub async fn list_by_org(&self, org_id: i64) -> Result<Vec<Employee>> {
        EmployeeRepo::find_by_org_id(self.db_pool.mysql_pool(), org_id).await
    }

    /// 更新员工
    pub async fn update(
        &self,
        id: i64,
        req: UpdateEmployeeRequest,
        updated_by: Option<i64>,
    ) -> Result<()> {
        let mut emp = self.get_by_id(id).await?;

        if let Some(name) = req.name {
            emp.name = name;
        }
        if let Some(avatar) = req.avatar {
            emp.avatar = Some(avatar);
        }
        if let Some(gender) = req.gender {
            emp.gender = Some(gender);
        }
        if let Some(mobile) = req.mobile {
            emp.mobile = Some(mobile);
        }
        if let Some(email) = req.email {
            emp.email = Some(email);
        }
        if let Some(hire_date) = req.hire_date {
            emp.hire_date = Some(hire_date);
        }
        if let Some(leave_date) = req.leave_date {
            emp.leave_date = Some(leave_date);
        }
        if let Some(status) = req.status {
            emp.status = Some(status);
        }
        if let Some(sort_order) = req.sort_order {
            emp.sort_order = Some(sort_order);
        }

        emp.updated_by = updated_by;
        emp.updated_at = Some(chrono::Utc::now().timestamp_millis());

        emp.update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 按月统计入职人数
    pub async fn hire_stats(
        &self,
        tenant_id: i64,
        req: HireStatsQuery,
    ) -> Result<Vec<MonthlyCount>> {
        // 将 "YYYY-MM" 字符串转为毫秒时间戳
        let parse_month = |s: &str| -> Option<i64> {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 2 {
                return None;
            }
            let year: i32 = parts[0].parse().ok()?;
            let month: u32 = parts[1].parse().ok()?;
            if !(1..=12).contains(&month) {
                return None;
            }
            // UTC 月初 00:00:00 的毫秒时间戳
            use chrono::{NaiveDate, TimeZone, Utc};
            let dt = NaiveDate::from_ymd_opt(year, month, 1)?;
            Some(Utc.from_local_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()).single()?.timestamp_millis())
        };

        let start_ts = req.start_month.as_deref().and_then(parse_month);
        // end_month 当月 +1 个月，用于 `hire_date < end_ts`
        let end_ts = req.end_month.as_deref().and_then(|s| {
            let parts: Vec<&str> = s.split('-').collect();
            if parts.len() != 2 {
                return None;
            }
            let year: i32 = parts[0].parse().ok()?;
            let month: u32 = parts[1].parse().ok()?;
            use chrono::{NaiveDate, TimeZone, Utc};
            let (ny, nm) = if month == 12 { (year + 1, 1u32) } else { (year, month + 1) };
            let dt = NaiveDate::from_ymd_opt(ny, nm, 1)?;
            Some(Utc.from_local_datetime(&dt.and_hms_opt(0, 0, 0).unwrap()).single()?.timestamp_millis())
        });

        let rows = EmployeeRepo::count_hires_by_month(
            self.db_pool.mysql_pool(),
            req.org_id,
            start_ts,
            end_ts,
        )
        .await?;

        Ok(rows
            .into_iter()
            .map(|(month, count)| MonthlyCount { month, count })
            .collect())
    }

    /// 删除员工
    pub async fn delete(&self, id: i64) -> Result<()> {
        let _ = self.get_by_id(id).await?;

        // ✏️ 删除员工部门关系
        let dept_rels = EmployeeDepartmentRepo::find_by_employee_id(self.db_pool.mysql_pool(), id)
            .await?;
        for rel in dept_rels {
            if let Some(rel_id) = rel.id {
                EmployeeDepartment::delete_by_id(self.db_pool.mysql_pool(), rel_id)
                    .await
                    .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

                // 📌 失效相关部门及其祖先部门的缓存
                if let Some(dept_service) = &self.department_service {
                    let _ = dept_service.invalidate_ancestor_caches(rel.department_id).await;
                }
            }
        }

        // ✏️ 删除员工岗位关系
        let pos_rels = EmployeePositionRepo::find_by_employee_id(self.db_pool.mysql_pool(), id)
            .await?;
        for rel in pos_rels {
            if let Some(rel_id) = rel.id {
                EmployeePosition::delete_by_id(self.db_pool.mysql_pool(), rel_id)
                    .await
                    .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
            }
        }

        // 删除员工
        Employee::delete_by_id(self.db_pool.mysql_pool(), id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

/// 员工部门关系 Service
pub struct EmployeeDepartmentService {
    db_pool: Arc<DbPool>,
    department_service: Option<Arc<crate::modules::department::DepartmentService>>,
}

impl EmployeeDepartmentService {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self {
            db_pool,
            department_service: None,
        }
    }

    /// 设置部门服务（用于缓存失效）
    pub fn with_department_service(
        mut self,
        department_service: Arc<crate::modules::department::DepartmentService>,
    ) -> Self {
        self.department_service = Some(department_service);
        self
    }

    /// 添加员工到部门
    pub async fn add_to_department(
        &self,
        tenant_id: i64,
        employee_id: i64,
        department_id: i64,
        is_primary: bool,
        is_leader: bool,
        created_by: Option<i64>,
    ) -> Result<i64> {
        // 检查是否已存在
        if let Some(_) = EmployeeDepartmentRepo::find_by_employee_and_department(
            self.db_pool.mysql_pool(),
            employee_id,
            department_id,
        )
        .await?
        {
            return Err(OrganizationError::EmployeeDepartmentRelExists.into());
        }

        // ✏️ 检查员工是否存在
        if let None = Employee::find_by_id(self.db_pool.mysql_pool(), employee_id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
        {
            return Err(OrganizationError::EmployeeNotFound.into());
        }

        // ✏️ 检查部门是否存在
        if let None = Department::find_by_id(self.db_pool.mysql_pool(), department_id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
        {
            return Err(OrganizationError::DepartmentNotFound.into());
        }

        // 如果设置为主部门，先清除其他主部门标记
        if is_primary {
            EmployeeDepartmentRepo::clear_primary_by_employee_id(
                self.db_pool.mysql_pool(),
                employee_id,
            )
            .await?;
        }

        let mut rel = EmployeeDepartment::default();
        rel.tenant_id = tenant_id;
        rel.employee_id = employee_id;
        rel.department_id = department_id;
        rel.is_primary = Some(if is_primary { 1 } else { 0 });
        rel.is_leader = Some(if is_leader { 1 } else { 0 });
        rel.created_by = created_by;
        rel.created_at = Some(chrono::Utc::now().timestamp_millis());

        let id = rel
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        // 📌 失效目标部门及其祖先部门的缓存
        if let Some(dept_service) = &self.department_service {
            let _ = dept_service.invalidate_ancestor_caches(department_id).await;
        }

        Ok(id)
    }

    /// 从部门移除员工
    pub async fn remove_from_department(&self, employee_id: i64, department_id: i64) -> Result<()> {
        let rel = EmployeeDepartmentRepo::find_by_employee_and_department(
            self.db_pool.mysql_pool(),
            employee_id,
            department_id,
        )
        .await?
        .ok_or(OrganizationError::EmployeeDepartmentRelNotFound)?;

        if let Some(id) = rel.id {
            EmployeeDepartment::delete_by_id(self.db_pool.mysql_pool(), id)
                .await
                .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        }

        // 📌 失效源部门及其祖先部门的缓存
        if let Some(dept_service) = &self.department_service {
            let _ = dept_service.invalidate_ancestor_caches(department_id).await;
        }

        Ok(())
    }

    /// 获取员工的所有部门
    pub async fn get_by_employee(&self, employee_id: i64) -> Result<Vec<EmployeeDepartment>> {
        EmployeeDepartmentRepo::find_by_employee_id(self.db_pool.mysql_pool(), employee_id).await
    }

    /// 获取员工的主部门
    pub async fn get_primary_department(
        &self,
        employee_id: i64,
    ) -> Result<Option<EmployeeDepartment>> {
        EmployeeDepartmentRepo::find_primary_by_employee_id(self.db_pool.mysql_pool(), employee_id)
            .await
    }

    /// 获取员工部门详情列表
    pub async fn get_details_by_employee(
        &self,
        employee_id: i64,
    ) -> Result<Vec<EmployeeDepartmentResponse>> {
        let rels = self.get_by_employee(employee_id).await?;
        if rels.is_empty() {
            return Ok(vec![]);
        }

        let dept_ids: Vec<i64> = rels.iter().map(|r| r.department_id).collect();

        // Fetch departments
        let builder = QueryBuilder::new("SELECT * FROM department").and_in("id", dept_ids);
        let depts = Department::find_all(self.db_pool.mysql_pool(), Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        // Map
        let mut result = Vec::new();
        for rel in rels {
            if let Some(dept) = depts.iter().find(|d| d.id == Some(rel.department_id)) {
                result.push(EmployeeDepartmentResponse {
                    id: rel.id.unwrap_or(0),
                    employee_id: rel.employee_id,
                    department_id: rel.department_id,
                    department_name: dept.name.clone(),
                    department_full_name: dept.full_name.clone(),
                    is_primary: rel.is_primary == Some(1),
                    is_leader: rel.is_leader == Some(1),
                });
            }
        }
        Ok(result)
    }
}

/// 员工岗位关系 Service
pub struct EmployeePositionService {
    db_pool: Arc<DbPool>,
}

impl EmployeePositionService {
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 添加员工岗位
    pub async fn add_position(
        &self,
        tenant_id: i64,
        employee_id: i64,
        position_id: i64,
        is_primary: bool,
        created_by: Option<i64>,
    ) -> Result<i64> {
        // 检查是否已存在
        if let Some(_) = EmployeePositionRepo::find_by_employee_and_position(
            self.db_pool.mysql_pool(),
            employee_id,
            position_id,
        )
        .await?
        {
            return Err(OrganizationError::EmployeePositionRelExists.into());
        }

        // ✏️ 检查员工是否存在
        if let None = Employee::find_by_id(self.db_pool.mysql_pool(), employee_id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
        {
            return Err(OrganizationError::EmployeeNotFound.into());
        }

        // ✏️ 检查岗位是否存在
        if let None = Position::find_by_id(self.db_pool.mysql_pool(), position_id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
        {
            return Err(OrganizationError::PositionNotFound.into());
        }

        // 如果设置为主岗位，先清除其他主岗位标记
        if is_primary {
            EmployeePositionRepo::clear_primary_by_employee_id(
                self.db_pool.mysql_pool(),
                employee_id,
            )
            .await?;
        }

        let mut rel = EmployeePosition::default();
        rel.tenant_id = tenant_id;
        rel.employee_id = employee_id;
        rel.position_id = position_id;
        rel.is_primary = Some(if is_primary { 1 } else { 0 });
        rel.created_by = created_by;
        rel.created_at = Some(chrono::Utc::now().timestamp_millis());

        let id = rel
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    /// 移除员工岗位
    pub async fn remove_position(&self, employee_id: i64, position_id: i64) -> Result<()> {
        let rel = EmployeePositionRepo::find_by_employee_and_position(
            self.db_pool.mysql_pool(),
            employee_id,
            position_id,
        )
        .await?
        .ok_or(OrganizationError::EmployeePositionRelNotFound)?;

        if let Some(id) = rel.id {
            EmployeePosition::delete_by_id(self.db_pool.mysql_pool(), id)
                .await
                .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        }

        Ok(())
    }

    /// 获取员工的所有岗位
    pub async fn get_by_employee(&self, employee_id: i64) -> Result<Vec<EmployeePosition>> {
        EmployeePositionRepo::find_by_employee_id(self.db_pool.mysql_pool(), employee_id).await
    }

    /// 获取员工的主岗位
    pub async fn get_primary_position(&self, employee_id: i64) -> Result<Option<EmployeePosition>> {
        EmployeePositionRepo::find_primary_by_employee_id(self.db_pool.mysql_pool(), employee_id)
            .await
    }

    /// 获取员工岗位详情列表
    pub async fn get_details_by_employee(
        &self,
        employee_id: i64,
    ) -> Result<Vec<EmployeePositionResponse>> {
        let rels = self.get_by_employee(employee_id).await?;
        if rels.is_empty() {
            return Ok(vec![]);
        }

        let pos_ids: Vec<i64> = rels.iter().map(|r| r.position_id).collect();

        // Fetch positions
        let builder = QueryBuilder::new("SELECT * FROM position").and_in("id", pos_ids);
        let positions = Position::find_all(self.db_pool.mysql_pool(), Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        // Map
        let mut result = Vec::new();
        for rel in rels {
            if let Some(pos) = positions.iter().find(|p| p.id == Some(rel.position_id)) {
                result.push(EmployeePositionResponse {
                    id: rel.id.unwrap_or(0),
                    employee_id: rel.employee_id,
                    position_id: rel.position_id,
                    position_name: pos.name.clone(),
                    position_level: pos.level,
                    is_primary: rel.is_primary == Some(1),
                });
            }
        }
        Ok(result)
    }
}
