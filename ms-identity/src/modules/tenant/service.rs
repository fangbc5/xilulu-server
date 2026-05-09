// 租户模块 Service 层
// 负责租户相关的业务逻辑

use crate::error::IdentityError;
use crate::modules::auth::ApplicationInfo;
use crate::modules::tenant::{Tenant, TenantApplicationRelRepo, TenantRepo};
use anyhow::Result;
use chrono::Utc;
use sqlxplus::Crud;
use sqlxplus::DbPool;
use std::sync::Arc;

/// 租户 Service
pub struct TenantService {
    db_pool: Arc<DbPool>,
}

impl TenantService {
    /// 创建新的 TenantService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 获取数据库连接池引用
    pub fn db_pool(&self) -> &DbPool {
        &self.db_pool
    }

    /// 获取租户信息（只读操作，不需要事务）
    pub async fn get_tenant_info(&self, tenant_id: i64) -> Result<Tenant> {
        let tenant = Tenant::find_by_id(self.db_pool.mysql_pool(), tenant_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        tenant.ok_or_else(|| anyhow::Error::from(IdentityError::TenantNotFound))
    }

    /// 批量获取租户信息（只读操作，不需要事务）
    pub async fn get_tenants_by_ids(&self, tenant_ids: &[i64]) -> Result<Vec<Tenant>> {
        if tenant_ids.is_empty() {
            return Ok(Vec::new());
        }

        use sqlxplus::QueryBuilder;
        let builder = QueryBuilder::new("SELECT * FROM `tenant`").and_in("id", tenant_ids.to_vec());

        let tenants = Tenant::find_all(self.db_pool.mysql_pool(), Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(tenants)
    }

    /// 获取租户列表（分页）
    pub async fn list_tenants(
        &self,
        page: u32,
        page_size: u32,
        search_key: Option<&str>,
    ) -> Result<(Vec<Tenant>, i64)> {
        use sqlxplus::QueryBuilder;

        let mut sql = "SELECT * FROM `tenant`".to_string();
        if let Some(key) = search_key {
            let like_pattern = format!("%{}%", key.replace("'", "''")); // 防止 SQL 注入
            sql = format!(
                "{} WHERE (name LIKE '{}' OR contact_name LIKE '{}' OR contact_mobile LIKE '{}')",
                sql, like_pattern, like_pattern, like_pattern
            );
        }
        let builder = QueryBuilder::new(&sql);

        let result = Tenant::paginate(self.db_pool.mysql_pool(), builder, page, page_size)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok((result.items, result.total as i64))
    }

    /// 获取租户总数（只读操作，不需要事务）
    pub async fn get_tenant_count(&self) -> Result<i64> {
        let builder = sqlxplus::QueryBuilder::new("");
        let count = Tenant::count(self.db_pool.mysql_pool(), builder)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
        Ok(count as i64)
    }

    /// 创建租户
    pub async fn create_tenant(
        &self,
        name: &str,
        contact_name: &str,
        contact_mobile: Option<&str>,
        package_id: i64,
        expire_time: chrono::DateTime<chrono::Utc>,
        account_count: i32,
        website: Option<&str>,
        create_by: Option<i64>,
        tenant_type: Option<i16>,
    ) -> Result<i64> {
        // 检查租户名称是否已存在
        if TenantRepo::exists_by_name(self.db_pool.mysql_pool(), name).await? {
            return Err(IdentityError::BusinessError("租户名称已存在".to_string()).into());
        }

        // 创建租户
        let tenant = Tenant {
            name: name.to_string(),
            contact_name: contact_name.to_string(),
            contact_mobile: contact_mobile.map(|s| s.to_string()),
            package_id,
            expire_time,
            account_count,
            website: website.map(|s| s.to_string()),
            status: Some(0),
            create_by,
            create_time: Some(Utc::now()),
            tenant_type: Some(tenant_type.unwrap_or(1)), // 默认个人租户
            ..Default::default()
        };

        let tenant_id = tenant
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(tenant_id)
    }

    /// 更新租户
    pub async fn update_tenant(
        &self,
        tenant_id: i64,
        name: Option<&str>,
        contact_name: Option<&str>,
        contact_mobile: Option<&str>,
        website: Option<&str>,
        status: Option<i16>,
        update_by: Option<i64>,
    ) -> Result<()> {
        // 查找现有租户
        let existing_tenant = self.get_tenant_info(tenant_id).await?;

        // 如果更新名称，检查是否已存在
        if let Some(new_name) = name {
            if new_name != existing_tenant.name {
                if TenantRepo::exists_by_name(self.db_pool.mysql_pool(), new_name).await? {
                    return Err(IdentityError::BusinessError("租户名称已存在".to_string()).into());
                }
            }
        }

        // 更新租户
        let updated_tenant = Tenant {
            id: Some(tenant_id),
            name: name.map(|s| s.to_string()).unwrap_or(existing_tenant.name),
            contact_name: contact_name
                .map(|s| s.to_string())
                .unwrap_or(existing_tenant.contact_name),
            contact_mobile: contact_mobile
                .map(|s| s.to_string())
                .or(existing_tenant.contact_mobile),
            website: website.map(|s| s.to_string()).or(existing_tenant.website),
            status: status.or(existing_tenant.status),
            update_by,
            update_time: Some(Utc::now()),
            ..existing_tenant
        };

        updated_tenant
            .update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除租户（软删除）
    pub async fn delete_tenant(&self, tenant_id: i64, _update_by: Option<i64>) -> Result<()> {
        Tenant::delete_by_id(self.db_pool.mysql_pool(), tenant_id)
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}

/// 租户应用关系 Service
pub struct TenantApplicationService {
    db_pool: Arc<DbPool>,
    application_service: Arc<crate::modules::auth::ApplicationService>,
}

impl TenantApplicationService {
    /// 创建新的 TenantApplicationService
    pub fn new(
        db_pool: Arc<DbPool>,
        application_service: Arc<crate::modules::auth::ApplicationService>,
    ) -> Self {
        Self {
            db_pool,
            application_service,
        }
    }

    /// 添加应用到租户
    pub async fn add_application_to_tenant(
        &self,
        tenant_id: i64,
        application_id: i64,
        expiration_time: Option<chrono::DateTime<chrono::Utc>>,
        create_by: Option<i64>,
    ) -> Result<i64> {
        // 检查关系是否已存在
        if let Some(_) = TenantApplicationRelRepo::find_by_tenant_and_application(
            self.db_pool.mysql_pool(),
            tenant_id,
            application_id,
        )
        .await?
        {
            return Err(IdentityError::BusinessError("租户应用关系已存在".to_string()).into());
        }

        // 创建关系（使用 model 的 insert 方法，自动生成雪花算法 ID）
        let rel = crate::modules::tenant::TenantApplicationRel {
            id: None, // sqlxplus 自动生成雪花算法 ID
            tenant_id,
            application_id,
            expiration_time,
            create_by,
            create_time: Some(chrono::Utc::now()),
            ..Default::default()
        };

        let rel_id = rel
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        Ok(rel_id)
    }

    /// 从租户移除应用
    pub async fn remove_application_from_tenant(
        &self,
        tenant_id: i64,
        application_id: i64,
    ) -> Result<()> {
        // 先查找关系记录获取 ID
        let rel = TenantApplicationRelRepo::find_by_tenant_and_application(
            self.db_pool.mysql_pool(),
            tenant_id,
            application_id,
        )
        .await?;

        if let Some(rel) = rel {
            if let Some(rel_id) = rel.id {
                // 使用 sqlx 直接删除
                sqlx::query("DELETE FROM `tenant_application_rel` WHERE id = ?")
                    .bind(rel_id)
                    .execute(self.db_pool.mysql_pool())
                    .await
                    .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;
            }
        }

        Ok(())
    }

    /// 获取租户的应用列表（只读操作，不需要事务）
    pub async fn get_tenant_applications(&self, tenant_id: i64) -> Result<Vec<ApplicationInfo>> {
        use std::collections::HashSet;
        
        let mut tenant_ids = vec![tenant_id];
        if let Ok(Some(tenant)) = Tenant::find_by_id(self.db_pool.mysql_pool(), tenant_id).await {
            tenant_ids.push(tenant.pid);
        }

        let builder = sqlxplus::QueryBuilder::new("SELECT * FROM `tenant_application_rel`")
            .and_in("tenant_id", tenant_ids);
            
        let rels = crate::modules::tenant::TenantApplicationRel::find_all(self.db_pool.mysql_pool(), Some(builder))
            .await
            .map_err(|e| IdentityError::DatabaseError(e.to_string()))?;

        // 如果关系表为空，返回空列表
        if rels.is_empty() {
            return Ok(Vec::new());
        }

        // 根据关系表中的应用ID，去重并批量查询应用详细信息
        let app_ids: Vec<i64> = rels.into_iter()
            .map(|rel| rel.application_id)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
            
        let apps = self
            .application_service
            .get_applications_by_ids(app_ids)
            .await?;

        // 转换为 ApplicationInfo
        let infos: Vec<ApplicationInfo> = apps.into_iter().map(ApplicationInfo::from).collect();
        Ok(infos)
    }
}
