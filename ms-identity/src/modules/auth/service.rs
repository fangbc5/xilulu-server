// 权限模块 Service 层
// 负责权限相关的业务逻辑

use crate::error::IdentityError;
use crate::modules::auth::{
    Application, Resource, ResourceRepo, Role, RoleRepo, RoleResourceRel,
    RoleResourceRelRepo,
};
use crate::modules::tenant::TenantApplicationRelRepo;
use anyhow::Result;
use chrono::Utc;
use sqlxplus::Crud;
use sqlxplus::DbPool;
use std::sync::Arc;

/// 角色 Service
pub struct RoleService {
    db_pool: Arc<DbPool>,
}

impl RoleService {
    /// 创建新的 RoleService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 获取数据库连接池引用
    pub fn db_pool(&self) -> &DbPool {
        &self.db_pool
    }

    /// 为组织创建角色（组织级别，带 biz_id 标识）
    pub async fn create_org_role(
        &self,
        tenant_id: i64,
        code: &str,
        name: &str,
        biz_id: Option<i64>,
        create_by: Option<i64>,
    ) -> Result<i64> {
        let role = Role {
            name: name.to_string(),
            code: code.to_string(),
            tenant_id,
            biz_id,
            create_by,
            create_time: Some(Utc::now()),
            ..Default::default()
        };

        let role_id = role
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(role_id)
    }

    /// 获取角色信息（只读操作，不需要事务）
    pub async fn get_role_info(&self, role_id: i64) -> Result<Role> {
        let role = Role::find_by_id(self.db_pool.mysql_pool(), role_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        role.ok_or_else(|| anyhow::Error::from(IdentityError::RoleNotFound))
    }



    /// 获取租户的所有角色（只读操作，不需要事务）
    pub async fn get_tenant_roles(&self, tenant_id: i64) -> Result<Vec<Role>> {
        use crate::modules::tenant::Tenant;
        let mut tenant_ids = vec![tenant_id];
        if let Ok(Some(tenant)) = Tenant::find_by_id(self.db_pool.mysql_pool(), tenant_id).await {
            tenant_ids.push(tenant.pid);
        }
        
        // sqlxplus 会自动处理软删除字段
        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `role`").and_in("tenant_id", tenant_ids);
        Role::find_all(self.db_pool.mysql_pool(), Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()).into())
    }

    /// 获取角色列表（分页）
    pub async fn list_roles(
        &self,
        page: u32,
        page_size: u32,
        tenant_id: Option<i64>,
    ) -> Result<(Vec<Role>, i64)> {
        use sqlxplus::QueryBuilder;

        let mut builder = QueryBuilder::new("SELECT * FROM `role`");
        if let Some(tid) = tenant_id {
            use crate::modules::tenant::Tenant;
            let mut tenant_ids = vec![tid];
            if let Ok(Some(tenant)) = Tenant::find_by_id(self.db_pool.mysql_pool(), tid).await {
                tenant_ids.push(tenant.pid);
            }
            builder = builder.and_in("tenant_id", tenant_ids);
        }

        // 使用 CRUD trait 的 paginate 方法
        // paginate(pool, builder, page, page_size)
        let result = Role::paginate(self.db_pool.mysql_pool(), builder, page, page_size)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok((result.items, result.total as i64))
    }

    /// 创建角色
    pub async fn create_role(
        &self,
        name: &str,
        code: &str,
        tenant_id: i64,
        category: Option<&str>,
        type_: Option<&str>,
        remarks: Option<&str>,
        state: Option<bool>,
        readonly_: Option<bool>,
        created_org_id: Option<i64>,
        create_by: Option<i64>,
    ) -> Result<i64> {
        // 检查角色代码是否已存在（同一租户下）
        if RoleRepo::find_by_code(self.db_pool.mysql_pool(), code, tenant_id)
            .await
            .is_ok()
        {
            return Err(IdentityError::BusinessError("角色代码已存在".to_string()).into());
        }

        // 创建角色
        let role = Role {
            name: name.to_string(),
            code: code.to_string(),
            tenant_id,
            category: category.map(|s| s.to_string()),
            type_: type_.map(|s| s.to_string()),
            remarks: remarks.map(|s| s.to_string()),
            state,
            readonly_,
            created_org_id,
            create_by,
            create_time: Some(Utc::now()),
            ..Default::default()
        };

        let role_id = role
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(role_id)
    }

    /// 更新角色
    pub async fn update_role(
        &self,
        role_id: i64,
        name: Option<&str>,
        remarks: Option<&str>,
        state: Option<bool>,
        readonly_: Option<bool>,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 查找现有角色
        let existing_role = self.get_role_info(role_id).await?;

        // 更新角色
        let updated_role = Role {
            id: Some(role_id),
            name: name.map(|s| s.to_string()).unwrap_or(existing_role.name),
            remarks: remarks.map(|s| s.to_string()).or(existing_role.remarks),
            state: state.or(existing_role.state),
            readonly_: readonly_.or(existing_role.readonly_),
            update_by,
            update_time: Some(Utc::now()),
            ..existing_role
        };

        updated_role
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除角色（软删除）
    pub async fn delete_role(&self, role_id: i64, _update_by: Option<i64>) -> Result<()> {
        // 删除角色
        Role::delete_by_id(self.db_pool.mysql_pool(), role_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

/// 资源 Service
pub struct ResourceService {
    db_pool: Arc<DbPool>,
}

impl ResourceService {
    /// 创建新的 ResourceService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 获取资源信息（只读操作，不需要事务）
    pub async fn get_resource_info(&self, resource_id: i64) -> Result<Resource> {
        let resource = Resource::find_by_id(self.db_pool.mysql_pool(), resource_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        resource.ok_or_else(|| anyhow::Error::from(IdentityError::ResourceNotFound))
    }

    /// 获取应用的所有资源（只读操作，不需要事务）
    pub async fn get_application_resources(&self, app_id: i64) -> Result<Vec<Resource>> {
        ResourceRepo::find_by_application_id(self.db_pool.mysql_pool(), app_id).await
    }



    /// 获取资源列表（分页）
    pub async fn list_resources(
        &self,
        page: u32,
        page_size: u32,
        application_id: Option<i64>,
        tenant_id: Option<i64>,
        search_key: Option<&str>,
    ) -> Result<(Vec<Resource>, i64)> {
        use sqlxplus::QueryBuilder;

        let mut builder = QueryBuilder::new("SELECT * FROM `resource`");

        // 如果提供了 tenant_id，先查询该租户下的应用ID列表，包含其 pid 继承的应用
        if let Some(tid) = tenant_id {
            use crate::modules::tenant::Tenant;
            let mut tenant_ids = vec![tid];
            if let Ok(Some(tenant)) = Tenant::find_by_id(self.db_pool.mysql_pool(), tid).await {
                tenant_ids.push(tenant.pid);
            }

            let builder_rel = QueryBuilder::new("SELECT * FROM `tenant_application_rel`").and_in("tenant_id", tenant_ids);
            let tenant_apps = crate::modules::tenant::TenantApplicationRel::find_all(self.db_pool.mysql_pool(), Some(builder_rel))
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

            let app_ids: Vec<i64> = tenant_apps
                .into_iter()
                .map(|rel| rel.application_id)
                .collect();

            if app_ids.is_empty() {
                // 如果租户下没有应用，返回空结果
                return Ok((Vec::new(), 0));
            }

            // 如果同时提供了 application_id，需要确保该应用属于该租户
            if let Some(app_id) = application_id {
                if app_ids.contains(&app_id) {
                    builder = builder.and_eq("application_id", app_id);
                } else {
                    // 应用不属于该租户，返回空结果
                    return Ok((Vec::new(), 0));
                }
            } else {
                // 使用 IN 查询过滤应用ID
                builder = builder.and_in("application_id", app_ids);
            }
        } else if let Some(app_id) = application_id {
            // 只提供了 application_id，没有 tenant_id
            builder = builder.and_eq("application_id", app_id);
        }

        // 如果提供了搜索关键词，添加搜索条件（资源代码或资源名称）
        if let Some(key) = search_key {
            if !key.is_empty() {
                builder = builder.and_group(|mut builder_group| {
                    builder_group = builder_group.or_like("code", format!("%{}%", key));
                    builder_group = builder_group.or_like("name", format!("%{}%", key));
                    builder_group
                });
            }
        }

        // 使用 CRUD trait 的 paginate 方法
        // paginate(pool, builder, page, page_size)
        let result = Resource::paginate(self.db_pool.mysql_pool(), builder, page, page_size)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok((result.items, result.total as i64))
    }

    /// 创建资源
    pub async fn create_resource(
        &self,
        application_id: i64,
        code: &str,
        name: &str,
        parent_id: i64,
        resource_type: Option<&str>,
        open_with: Option<&str>,
        describe_: Option<&str>,
        path: Option<&str>,
        component: Option<&str>,
        redirect: Option<&str>,
        icon: Option<&str>,
        create_by: Option<i64>,
    ) -> Result<i64> {
        // 创建资源
        let resource = Resource {
            application_id,
            code: code.to_string(),
            name: name.to_string(),
            parent_id,
            resource_type: resource_type.map(|s| s.to_string()),
            open_with: open_with.map(|s| s.to_string()),
            describe_: describe_.map(|s| s.to_string()),
            path: path.map(|s| s.to_string()),
            component: component.map(|s| s.to_string()),
            redirect: redirect.map(|s| s.to_string()),
            icon: icon.map(|s| s.to_string()),
            create_by,
            create_time: Some(Utc::now()),
            ..Default::default()
        };

        let resource_id = resource
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(resource_id)
    }

    /// 更新资源
    pub async fn update_resource(
        &self,
        resource_id: i64,
        name: Option<&str>,
        describe_: Option<&str>,
        path: Option<&str>,
        component: Option<&str>,
        redirect: Option<&str>,
        icon: Option<&str>,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 查找现有资源
        let existing_resource = self.get_resource_info(resource_id).await?;

        // 更新资源
        let updated_resource = Resource {
            id: Some(resource_id),
            name: name
                .map(|s| s.to_string())
                .unwrap_or(existing_resource.name),
            describe_: describe_
                .map(|s| s.to_string())
                .or(existing_resource.describe_),
            path: path.map(|s| s.to_string()).or(existing_resource.path),
            component: component
                .map(|s| s.to_string())
                .or(existing_resource.component),
            redirect: redirect
                .map(|s| s.to_string())
                .or(existing_resource.redirect),
            icon: icon.map(|s| s.to_string()).or(existing_resource.icon),
            update_by,
            update_time: Some(Utc::now()),
            ..existing_resource
        };

        updated_resource
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除资源（软删除）
    pub async fn delete_resource(&self, resource_id: i64, _update_by: Option<i64>) -> Result<()> {
        // 删除资源
        Resource::delete_by_id(self.db_pool.mysql_pool(), resource_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

/// 角色资源关系 Service
pub struct PermissionService {
    db_pool: Arc<DbPool>,
}

impl PermissionService {
    /// 创建新的 PermissionService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 为角色分配资源
    pub async fn assign_resource_to_role(
        &self,
        role_id: i64,
        resource_id: i64,
        create_by: Option<i64>,
    ) -> Result<i64> {
        // 获取角色和资源信息
        let role = Role::find_by_id(self.db_pool.mysql_pool(), role_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| IdentityError::RoleNotFound)?;

        let resource = Resource::find_by_id(self.db_pool.mysql_pool(), resource_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| IdentityError::ResourceNotFound)?;

        // 使用原始 sqlx 查询检查是否存在记录（包括软删除的记录）
        let existing_rel: Option<(i64, Option<bool>)> = sqlx::query_as(
            "SELECT id, is_del FROM `role_resource_rel` WHERE role_id = ? AND resource_id = ? and application_id = ? and tenant_id = ?",
        )
        .bind(role_id)
        .bind(resource_id)
        .bind(resource.application_id)
        .bind(role.tenant_id)
        .fetch_optional(self.db_pool.mysql_pool())
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        let rel_id = match existing_rel {
            Some((id, is_del)) => {
                // 记录已存在
                if is_del.unwrap_or(false) {
                    // 如果是软删除状态，则恢复（更新 is_del 和 update_time）
                    let now = Utc::now();
                    sqlx::query(
                        "UPDATE `role_resource_rel` SET is_del = 0, update_time = ?, update_by = ? WHERE id = ?"
                    )
                    .bind(now)
                    .bind(create_by)
                    .bind(id)
                    .execute(self.db_pool.mysql_pool())
                    .await
                    .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
                } else {
                    // 如果已存在且未删除，返回错误
                    return Err(
                        IdentityError::BusinessError("角色资源关系已存在".to_string()).into(),
                    );
                }
                id
            }
            None => {
                // 记录不存在，插入新记录
        let rel = RoleResourceRel {
            role_id,
            resource_id,
                    tenant_id: role.tenant_id, // 从角色获取租户ID
                    application_id: resource.application_id, // 从资源获取应用ID
            create_by,
            create_time: Some(Utc::now()),
                    is_del: Some(false), // 显式设置为未删除状态
            ..Default::default()
        };

                rel.insert(self.db_pool.mysql_pool())
            .await
                    .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            }
        };

        Ok(rel_id)
    }

    /// 移除角色的资源
    pub async fn remove_resource_from_role(&self, role_id: i64, resource_id: i64) -> Result<()> {
        // 先查找关系记录获取 ID
        let rel = RoleResourceRelRepo::find_by_role_and_resource(
            self.db_pool.mysql_pool(),
            role_id,
            resource_id,
        )
        .await?;

        if let Some(rel) = rel {
            if let Some(rel_id) = rel.id {
                RoleResourceRel::delete_by_id(self.db_pool.mysql_pool(), rel_id)
                    .await
                    .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// 获取角色的资源列表（只读操作，不需要事务）
    /// 返回完整的资源信息，而不仅仅是关系数据
    pub async fn get_role_resources(&self, role_id: i64) -> Result<Vec<Resource>> {
        use sqlxplus::QueryBuilder;
        use std::collections::HashSet;

        // 1. 查询角色资源关系
        let rels = RoleResourceRelRepo::find_by_role_id(self.db_pool.mysql_pool(), role_id).await?;

        if rels.is_empty() {
            return Ok(Vec::new());
        }

        // 2. 提取资源ID列表（去重）
        let resource_ids: Vec<i64> = rels
            .into_iter()
            .map(|rel| rel.resource_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // 3. 查询完整的资源信息
        let builder = QueryBuilder::new("SELECT * FROM `resource`")
            .and_in("id", resource_ids)
            .order_by("sort_value", true);

        let resources = Resource::find_all(self.db_pool.mysql_pool(), Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(resources)
    }



    /// 获取用户的菜单资源（根据用户所属角色的资源）
    /// 只返回菜单类型的资源（resource_type = '20'）
    /// 从 user_role 表查询用户在该租户下的所有角色
    pub async fn get_user_menus(
        &self,
        user_id: i64,
        tenant_id: i64,
        application_id: i64,
    ) -> Result<Vec<Resource>> {
        use sqlxplus::QueryBuilder;
        use std::collections::HashSet;

        // 0. 获取租户及其父级继承 pid
        use crate::modules::tenant::Tenant;
        let tenant = Tenant::find_by_id(self.db_pool.mysql_pool(), tenant_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::TenantNotFound))?;

        // 1. 查询用户在该租户下的所有角色（从 user_role 表）
        use crate::modules::user::UserRoleRepo;
        let user_roles =
            UserRoleRepo::find_by_user_and_tenant(self.db_pool.mysql_pool(), user_id, tenant_id)
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        if user_roles.is_empty() {
            return Ok(Vec::new()); // 用户在该租户下没有角色，返回空菜单
        }

        // 2. 提取所有角色ID
        let role_ids: Vec<i64> = user_roles.into_iter().map(|ur| ur.role_id).collect();

        // 3. 批量查询这些角色的所有资源关系（同时匹配当前租户自身资源及 pid 继承环境下的资源与应用ID）
        let builder = QueryBuilder::new("SELECT * FROM `role_resource_rel`")
            .and_in("role_id", role_ids)
            .and_in("tenant_id", vec![tenant_id, tenant.pid])
            .and_eq("application_id", application_id);

        let role_resources = RoleResourceRel::find_all(self.db_pool.mysql_pool(), Some(builder))
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        if role_resources.is_empty() {
            return Ok(Vec::new());
        }

        // 4. 提取资源ID列表（去重）
        let resource_ids: Vec<i64> = role_resources
            .into_iter()
            .map(|rel| rel.resource_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // 5. 查询资源，只返回菜单类型（resource_type = '20'）且属于指定应用的资源
        let builder = QueryBuilder::new("SELECT * FROM `resource`")
            .and_in("id", resource_ids)
            .and_eq("application_id", application_id)
            .and_eq("resource_type", "20") // 只返回菜单
            .order_by("sort_value", true);

        let resources = Resource::find_all(self.db_pool.mysql_pool(), Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(resources)
    }

    /// 获取用户在指定菜单下的子资源（按钮 / 字段 / 数据等）
    /// 只返回 parent_id 等于菜单 ID 且用户有权限的资源
    /// 从 user_role 表查询用户在该租户下的所有角色
    pub async fn get_user_menu_resources(
        &self,
        user_id: i64,
        tenant_id: i64,
        application_id: i64,
        menu_id: i64,
    ) -> Result<Vec<Resource>> {
        use sqlxplus::QueryBuilder;
        use std::collections::HashSet;

        // 0. 获取租户及其父级继承 pid
        use crate::modules::tenant::Tenant;
        let tenant = Tenant::find_by_id(self.db_pool.mysql_pool(), tenant_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::TenantNotFound))?;

        // 1. 查询用户在该租户下的所有角色（从 user_role 表）
        use crate::modules::user::UserRoleRepo;
        let user_roles =
            UserRoleRepo::find_by_user_and_tenant(self.db_pool.mysql_pool(), user_id, tenant_id)
        .await
        .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        if user_roles.is_empty() {
            return Ok(Vec::new()); // 用户在该租户下没有角色，返回空
        }

        // 2. 提取所有角色ID
        let role_ids: Vec<i64> = user_roles.into_iter().map(|ur| ur.role_id).collect();

        // 3. 批量查询这些角色的所有资源关系（同时匹配当前租户自身资源及 pid 继承环境下的资源与应用ID）
        let builder = QueryBuilder::new("SELECT * FROM `role_resource_rel`")
            .and_in("role_id", role_ids)
            .and_in("tenant_id", vec![tenant_id, tenant.pid])
            .and_eq("application_id", application_id);

        let role_resources = RoleResourceRel::find_all(self.db_pool.mysql_pool(), Some(builder))
                .await
                .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        if role_resources.is_empty() {
            return Ok(Vec::new());
        }

        // 4. 提取资源ID列表（去重）
        let resource_ids: Vec<i64> = role_resources
            .into_iter()
            .map(|rel| rel.resource_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        // 5. 查询资源：属于指定应用且 parent_id 等于菜单 ID
        let builder = QueryBuilder::new("SELECT * FROM `resource`")
            .and_in("id", resource_ids)
            .and_eq("application_id", application_id)
            .and_eq("parent_id", menu_id)
            .order_by("sort_value", true);

        let resources = Resource::find_all(self.db_pool.mysql_pool(), Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(resources)
    }
}

/// 应用 Service
pub struct ApplicationService {
    db_pool: Arc<DbPool>,
}

impl ApplicationService {
    /// 创建新的 ApplicationService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 获取应用信息（只读操作，不需要事务）
    pub async fn get_application_info(&self, app_id: i64) -> Result<Application> {
        let app = Application::find_by_id(self.db_pool.mysql_pool(), app_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        app.ok_or_else(|| anyhow::Error::from(IdentityError::ApplicationNotFound))
    }



    /// 根据 IDs 批量获取应用（只读操作，不需要事务）
    pub async fn get_applications_by_ids(&self, app_ids: Vec<i64>) -> Result<Vec<Application>> {
        if app_ids.is_empty() {
            return Ok(Vec::new());
        }
        let apps = Application::find_by_ids(self.db_pool.mysql_pool(), app_ids)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        Ok(apps)
    }



    /// 获取应用列表（分页）
    pub async fn list_applications(
        &self,
        page: u32,
        page_size: u32,
        search_key: Option<&str>,
    ) -> Result<(Vec<Application>, i64)> {
        use sqlxplus::QueryBuilder;

        let mut builder = QueryBuilder::new("SELECT * FROM `application`");

        // 如果有搜索关键词，添加搜索条件
        // 使用 and_group 将 OR 条件包裹，避免 SQL 优先级问题导致 is_del 过滤被绕过
        if let Some(key) = search_key {
            builder = builder.and_group(|g| g.or_like("app_key", key).or_like("name", key));
        }

        builder = builder.order_by("sort_value", true);

        // 使用 CRUD trait 的 paginate 方法
        let result = Application::paginate(self.db_pool.mysql_pool(), builder, page, page_size)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok((result.items, result.total))
    }

    /// 创建应用
    pub async fn create_application(
        &self,
        name: &str,
        app_key: &str,
        r#type: Option<&str>,
        app_secret: Option<&str>,
        version: Option<&str>,
        redirect: Option<&str>,
        introduce: Option<&str>,
        remark: Option<&str>,
        url: Option<&str>,
        is_general: Option<bool>,
        is_visible: Option<bool>,
        sort_value: Option<i32>,
        create_by: Option<i64>,
    ) -> Result<i64> {
        // 检查应用标识是否已存在（通过 app_key 检查）
        let builder =
            sqlxplus::QueryBuilder::new("SELECT * FROM `application`").and_eq("app_key", app_key);
        let existing = Application::find_all(self.db_pool.mysql_pool(), Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        if !existing.is_empty() {
            return Err(IdentityError::BusinessError("应用标识已存在".to_string()).into());
        }

        // 创建应用
        let app = Application {
            name: Some(name.to_string()),
            app_key: Some(app_key.to_string()),
            app_secret: app_secret.map(|s| s.to_string()),
            version: version.map(|s| s.to_string()),
            r#type: r#type.map(|s| s.to_string()).or(Some("10".to_string())), // 默认类型为自建应用
            redirect: redirect.map(|s| s.to_string()),
            introduce: introduce.map(|s| s.to_string()),
            remark: remark.map(|s| s.to_string()),
            url: url.map(|s| s.to_string()),
            is_general: is_general.or(Some(false)),
            is_visible: is_visible.or(Some(true)),
            sort_value: sort_value.or(Some(1)),
            create_by,
            create_time: Some(Utc::now()),
            ..Default::default()
        };

        let app_id = app
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(app_id)
    }

    /// 更新应用
    pub async fn update_application(
        &self,
        app_id: i64,
        name: Option<&str>,
        r#type: Option<&str>,
        version: Option<&str>,
        redirect: Option<&str>,
        introduce: Option<&str>,
        remark: Option<&str>,
        url: Option<&str>,
        is_general: Option<bool>,
        is_visible: Option<bool>,
        sort_value: Option<i32>,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 查找现有应用
        let existing_app = Application::find_by_id(self.db_pool.mysql_pool(), app_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?
            .ok_or_else(|| anyhow::Error::from(IdentityError::ApplicationNotFound))?;

        // 更新应用
        let updated_app = Application {
            id: Some(app_id),
            name: name.map(|s| s.to_string()).or(existing_app.name),
            r#type: r#type.map(|s| s.to_string()).or(existing_app.r#type),
            version: version.map(|s| s.to_string()).or(existing_app.version),
            redirect: redirect.map(|s| s.to_string()).or(existing_app.redirect),
            introduce: introduce.map(|s| s.to_string()).or(existing_app.introduce),
            remark: remark.map(|s| s.to_string()).or(existing_app.remark),
            url: url.map(|s| s.to_string()).or(existing_app.url),
            is_general: is_general.or(existing_app.is_general),
            is_visible: is_visible.or(existing_app.is_visible),
            sort_value: sort_value.or(existing_app.sort_value),
            update_by,
            update_time: Some(Utc::now()),
            ..existing_app
        };

        updated_app
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除应用（软删除）
    pub async fn delete_application(&self, app_id: i64, _update_by: Option<i64>) -> Result<()> {
        Application::delete_by_id(self.db_pool.mysql_pool(), app_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
