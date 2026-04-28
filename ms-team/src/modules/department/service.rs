use super::model::dto::{
    CreateDepartmentRequest, DepartmentResponse, DepartmentTreeNode, ListDepartmentsQuery,
    UpdateDepartmentRequest,
};
use super::model::entity::Department;
use super::repository::DepartmentRepo;
use crate::error::{OrganizationError, Result};
use crate::modules::employee::EmployeeDepartmentRepo;
use crate::modules::organization::Organization;
use fbc_starter::cache::{CacheKeyBuilder, SimpleCacheKeyBuilder, ValueType};
use redis::AsyncCommands;
use sqlxplus::Crud;
use sqlxplus::DbPool;
use sqlxplus::QueryBuilder;
use std::sync::Arc;
use std::time::Duration;

/// 部门 Service
pub struct DepartmentService {
    db_pool: Arc<DbPool>,
    fbc_app_state: Arc<fbc_starter::AppState>,
}

impl DepartmentService {
    /// 创建新的 DepartmentService
    pub fn new(db_pool: Arc<DbPool>, fbc_app_state: Arc<fbc_starter::AppState>) -> Self {
        Self {
            db_pool,
            fbc_app_state,
        }
    }

    /// 分页查询部门
    pub async fn find_page(
        &self,
        tenant_id: i64,
        req: ListDepartmentsQuery,
    ) -> Result<(Vec<Department>, i64)> {
        let mut builder =
            QueryBuilder::new("SELECT * FROM department").and_eq("tenant_id", tenant_id);

        if let Some(org_id) = req.org_id {
            builder = builder.and_eq("org_id", org_id);
        }

        if let Some(parent_id) = req.parent_id {
            builder = builder.and_eq("parent_id", parent_id);
        }

        if let Some(ref keyword) = req.keyword {
            builder = builder.and_group(|mut g| {
                g = g.or_like("name", format!("%{}%", keyword));
                g = g.or_like("code", format!("%{}%", keyword));
                g
            });
        }

        if let Some(status) = req.status {
            builder = builder.and_eq("status", status);
        }

        // Default sort by sort_order, then created_at
        builder = builder
            .order_by("sort_order", false)
            .order_by("created_at", true);

        let page_num = req.page.cursor.unwrap_or(1);
        let result = Department::paginate(
            self.db_pool.mysql_pool(),
            builder,
            page_num,
            req.page.page_size,
        )
        .await
        .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok((result.items, result.total as i64))
    }

    /// 转换实体为响应（不包含员工数统计）
    fn to_response(dept: &Department) -> DepartmentResponse {
        DepartmentResponse {
            id: dept.id.unwrap_or(0),
            tenant_id: dept.tenant_id,
            org_id: dept.org_id,
            parent_id: dept.parent_id,
            code: dept.code.clone(),
            name: dept.name.clone(),
            full_name: dept.full_name.clone(),
            path: dept.path.clone(),
            level: dept.level,
            leader_employee_id: dept.leader_employee_id,
            sort_order: dept.sort_order,
            status: dept.status,
            total_employee_count: None,
            employee_count: None,
        }
    }

    /// 转换实体为响应（包含员工数统计，带缓存）
    async fn to_response_with_count(
        &self,
        dept: &Department,
    ) -> Result<DepartmentResponse> {
        let dept_id = dept.id.unwrap_or(0);
        let path = dept.path.clone().unwrap_or_default();

        // 统计该部门的总员工数（含下属部门）
        let total_employee_count = self
            .get_total_employee_count(dept_id, &path, dept.tenant_id)
            .await?;

        // 统计该部门的直属员工数
        let employee_count = self
            .get_direct_employee_count(dept_id)
            .await?;

        Ok(DepartmentResponse {
            id: dept.id.unwrap_or(0),
            tenant_id: dept.tenant_id,
            org_id: dept.org_id,
            parent_id: dept.parent_id,
            code: dept.code.clone(),
            name: dept.name.clone(),
            full_name: dept.full_name.clone(),
            path: dept.path.clone(),
            level: dept.level,
            leader_employee_id: dept.leader_employee_id,
            sort_order: dept.sort_order,
            status: dept.status,
            total_employee_count: Some(total_employee_count),
            employee_count: Some(employee_count),
        })
    }

    /// 创建部门
    pub async fn create(
        &self,
        tenant_id: i64,
        req: CreateDepartmentRequest,
        created_by: Option<i64>,
    ) -> Result<i64> {
        // 检查编码是否已存在
        if let Some(_) =
            DepartmentRepo::find_by_org_and_code(self.db_pool.mysql_pool(), req.org_id, &req.code)
                .await?
        {
            return Err(OrganizationError::DepartmentExists(req.code).into());
        }

        // ✏️ 检查组织是否存在
        if let None = Organization::find_by_id(self.db_pool.mysql_pool(), req.org_id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
        {
            return Err(OrganizationError::OrganizationNotFound.into());
        }

        // 计算路径和层级
        let (path, level, full_name) = if let Some(parent_id) = req.parent_id {
            let parent = Department::find_by_id(self.db_pool.mysql_pool(), parent_id)
                .await
                .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
                .ok_or(OrganizationError::DepartmentNotFound)?;

            let parent_path = parent.path.unwrap_or_else(|| format!("/{}/", parent_id));
            let new_level = parent.level.unwrap_or(1) + 1;
            let parent_full_name = parent.full_name.unwrap_or(parent.name.clone());
            let new_full_name = format!("{}/{}", parent_full_name, req.name);

            (parent_path, new_level, new_full_name)
        } else {
            // 顶级部门
            (String::from("/"), 1, req.name.clone())
        };

        let now = chrono::Utc::now().timestamp_millis();
        let mut dept = Department::default();
        dept.tenant_id = tenant_id;
        dept.org_id = req.org_id;
        dept.parent_id = req.parent_id;
        dept.code = req.code;
        dept.name = req.name;
        dept.full_name = Some(full_name);
        dept.path = Some(path);
        dept.level = Some(level);
        dept.leader_employee_id = req.leader_employee_id;
        dept.sort_order = req.sort_order.or(Some(0));
        dept.status = Some(1);
        dept.created_by = created_by;
        dept.created_at = Some(now);
        dept.updated_at = Some(now);

        let id = dept
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        // 更新 path（包含自己的 ID）
        let mut update_dept = Department::default();
        update_dept.id = Some(id);
        update_dept.path = Some(format!(
            "{}{}/",
            dept.path.unwrap_or_default().trim_end_matches('/'),
            id
        ));
        update_dept
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(id)
    }

    /// 获取部门详情
    pub async fn get_by_id(&self, id: i64) -> Result<Department> {
        let dept = Department::find_by_id(self.db_pool.mysql_pool(), id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        dept.ok_or_else(|| OrganizationError::DepartmentNotFound.into())
    }

    /// 获取组织下的所有部门
    pub async fn list_by_org(&self, org_id: i64) -> Result<Vec<Department>> {
        DepartmentRepo::find_by_org_id(self.db_pool.mysql_pool(), org_id).await
    }

    /// 获取部门树
    pub async fn get_tree(&self, org_id: i64) -> Result<Vec<DepartmentTreeNode>> {
        let all_depts = self.list_by_org(org_id).await?;
        let root_depts: Vec<&Department> =
            all_depts.iter().filter(|d| d.parent_id.is_none()).collect();

        let mut tree = Vec::new();
        for root in root_depts {
            let node = self.build_tree_node(root, &all_depts);
            tree.push(node);
        }

        Ok(tree)
    }

    /// 递归构建部门树节点
    fn build_tree_node(&self, dept: &Department, all_depts: &[Department]) -> DepartmentTreeNode {
        let dept_id = dept.id.unwrap_or(0);
        let children: Vec<DepartmentTreeNode> = all_depts
            .iter()
            .filter(|d| d.parent_id == Some(dept_id))
            .map(|d| self.build_tree_node(d, all_depts))
            .collect();

        DepartmentTreeNode {
            department: Self::to_response(dept),
            children,
        }
    }

    /// 更新部门
    pub async fn update(
        &self,
        id: i64,
        req: UpdateDepartmentRequest,
        updated_by: Option<i64>,
    ) -> Result<()> {
        let existing = self.get_by_id(id).await?;

        let mut dept = Department::default();
        dept.id = Some(id);
        dept.name = req.name.unwrap_or(existing.name);
        dept.leader_employee_id = req.leader_employee_id.or(existing.leader_employee_id);
        dept.sort_order = req.sort_order.or(existing.sort_order);
        dept.status = req.status.or(existing.status);
        dept.updated_by = updated_by;
        dept.updated_at = Some(chrono::Utc::now().timestamp_millis());

        dept.update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除部门
    pub async fn delete(&self, id: i64) -> Result<()> {
        let _ = self.get_by_id(id).await?;

        // 检查是否有子部门
        if DepartmentRepo::has_children(self.db_pool.mysql_pool(), id).await? {
            return Err(OrganizationError::DepartmentHasChildren.into());
        }

        // 检查是否有员工
        if EmployeeDepartmentRepo::has_employees(self.db_pool.mysql_pool(), id).await? {
            return Err(OrganizationError::DepartmentHasEmployees.into());
        }

        Department::delete_by_id(self.db_pool.mysql_pool(), id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 获取子部门ID列表（包括孙子部门）
    pub async fn get_children_ids(&self, dept_id: i64) -> Result<Vec<i64>> {
        let dept = self.get_by_id(dept_id).await?;
        let path = dept.path.unwrap_or_default();

        let children =
            DepartmentRepo::find_by_path_prefix(self.db_pool.mysql_pool(), &path).await?;
        let ids: Vec<i64> = children.into_iter().filter_map(|d| d.id).collect();

        Ok(ids)
    }

    /// 获取部门的所有下属员工总数（含子部门）
    /// 使用 CacheKeyBuilder 规范构建缓存键，支持 Redis 缓存
    async fn get_total_employee_count(
        &self,
        dept_id: i64,
        path: &str,
        tenant_id: i64,
    ) -> Result<i64> {
        // 1. 构建缓存键（遵循 CacheKeyBuilder 规范）
        let cache_builder = SimpleCacheKeyBuilder::new("department")
            .with_modular("organization")
            .with_field("employee_count")
            .with_value_type(ValueType::Number)
            .with_expire(Duration::from_secs(300)); // 5分钟

        let cache_key = cache_builder.key(&[&dept_id]);

        // 2. 尝试从 Redis 缓存获取
        if let Ok(mut redis) = self.fbc_app_state.as_ref().redis().await {
            if let Ok(value) = redis.get::<_, String>(&cache_key.key).await {
                if let Ok(count) = value.parse::<i64>() {
                    return Ok(count);
                }
            }
        }

        // 3. 从数据库查询
        let count =
            DepartmentRepo::count_employees_by_dept_id(self.db_pool.mysql_pool(), dept_id, path, tenant_id)
                .await?;

        // 4. 写入缓存
        if let Ok(mut redis) = self.fbc_app_state.as_ref().redis().await {
            let _ = redis
                .set_ex::<_, _, ()>(&cache_key.key, count.to_string(), 300)
                .await;
        }

        Ok(count)
    }

    /// 获取部门的直属员工数
    /// 使用 CacheKeyBuilder 规范构建缓存键，支持 Redis 缓存
    async fn get_direct_employee_count(
        &self,
        dept_id: i64,
    ) -> Result<i64> {
        // 1. 构建缓存键
        let cache_builder = SimpleCacheKeyBuilder::new("department")
            .with_modular("organization")
            .with_field("direct_employee_count")
            .with_value_type(ValueType::Number)
            .with_expire(Duration::from_secs(300));

        let cache_key = cache_builder.key(&[&dept_id]);

        // 2. 尝试从 Redis 缓存获取
        if let Ok(mut redis) = self.fbc_app_state.as_ref().redis().await {
            if let Ok(value) = redis.get::<_, String>(&cache_key.key).await {
                if let Ok(count) = value.parse::<i64>() {
                    return Ok(count);
                }
            }
        }

        // 3. 从数据库查询
        let count = DepartmentRepo::count_direct_employees(self.db_pool.mysql_pool(), dept_id).await?;

        // 4. 写入缓存
        if let Ok(mut redis) = self.fbc_app_state.as_ref().redis().await {
            let _ = redis
                .set_ex::<_, _, ()>(&cache_key.key, count.to_string(), 300)
                .await;
        }

        Ok(count)
    }

    /// 获取根部门列表（带员工数）
    pub async fn get_roots(
        &self,
        org_id: i64,
        tenant_id: i64,
    ) -> Result<Vec<DepartmentResponse>> {
        // 从数据库查询根部门
        let depts = DepartmentRepo::find_root_by_org_id(self.db_pool.mysql_pool(), org_id).await?;
        
        // 为每个部门补充员工数统计
        let mut responses = Vec::new();
        for dept in depts {
            let resp = self.department_to_response_with_counts(dept, tenant_id).await?;
            responses.push(resp);
        }

        Ok(responses)
    }

    /// 获取子部门列表（带员工数）
    pub async fn get_children(
        &self,
        parent_id: i64,
        tenant_id: i64,
    ) -> Result<Vec<DepartmentResponse>> {
        // 从数据库查询子部门
        let depts = DepartmentRepo::find_by_parent_id(self.db_pool.mysql_pool(), parent_id).await?;
        
        // 为每个部门补充员工数统计
        let mut responses = Vec::new();
        for dept in depts {
            let resp = self.department_to_response_with_counts(dept, tenant_id).await?;
            responses.push(resp);
        }

        Ok(responses)
    }

    /// 将 Department 实体转换为带员工数的响应
    async fn department_to_response_with_counts(
        &self,
        dept: Department,
        tenant_id: i64,
    ) -> Result<DepartmentResponse> {
        let dept_id = dept.id.unwrap_or(0);
        let path = dept.path.clone().unwrap_or_default();

        let total_count = self
            .get_total_employee_count(dept_id, &path, tenant_id)
            .await
            .ok();
        let direct_count = self
            .get_direct_employee_count(dept_id)
            .await
            .ok();

        Ok(DepartmentResponse {
            id: dept_id,
            tenant_id: dept.tenant_id,
            org_id: dept.org_id,
            parent_id: dept.parent_id,
            code: dept.code,
            name: dept.name,
            full_name: dept.full_name,
            path: dept.path,
            level: dept.level,
            leader_employee_id: dept.leader_employee_id,
            sort_order: dept.sort_order,
            status: dept.status,
            total_employee_count: total_count,
            employee_count: direct_count,
        })
    }

    /// 失效单个部门的员工数缓存
    /// 用于员工添加/删除到部门时调用
    pub async fn invalidate_employee_count_cache(&self, dept_id: i64) -> Result<()> {
        // 清理总员工数缓存
        let cache_builder_total = SimpleCacheKeyBuilder::new("department")
            .with_modular("organization")
            .with_field("employee_count")
            .with_value_type(ValueType::Number);
        let cache_key_total = cache_builder_total.key(&[&dept_id]);

        // 清理直属员工数缓存
        let cache_builder_direct = SimpleCacheKeyBuilder::new("department")
            .with_modular("organization")
            .with_field("direct_employee_count")
            .with_value_type(ValueType::Number);
        let cache_key_direct = cache_builder_direct.key(&[&dept_id]);

        // 从 Redis 删除缓存
        if let Ok(mut redis) = self.fbc_app_state.as_ref().redis().await {
            let _ = redis.del::<_, ()>(&cache_key_total.key).await;
            let _ = redis.del::<_, ()>(&cache_key_direct.key).await;
        }

        Ok(())
    }

    /// 失效部门及其所有祖先部门的员工数缓存
    /// 因为父部门的总员工数会受子部门的影响
    pub async fn invalidate_ancestor_caches(&self, dept_id: i64) -> Result<()> {
        // 收集所有祖先部门 ID（包括当前部门）
        let mut ancestor_ids = vec![dept_id];
        let mut current_id = dept_id;

        loop {
            // 获取当前部门信息
            match self.get_by_id(current_id).await {
                Ok(dept) => {
                    // 如果有父部门，继续向上
                    if let Some(parent_id) = dept.parent_id {
                        ancestor_ids.push(parent_id);
                        current_id = parent_id;
                    } else {
                        // 到达根部门了
                        break;
                    }
                }
                Err(_) => {
                    // 如果获取父部门失败，停止
                    break;
                }
            }
        }

        // 批量失效所有祖先部门的缓存
        for ancestor_id in ancestor_ids {
            let _ = self.invalidate_employee_count_cache(ancestor_id).await;
        }

        Ok(())
    }
}

