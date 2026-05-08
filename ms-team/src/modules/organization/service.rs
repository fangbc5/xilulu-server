use super::model::dto::{
    CreateOrganizationRequest, ListOrganizationsQuery, OrganizationResponse, OrganizationTreeNode,
    UpdateOrganizationRequest,
};
use super::model::entity::Organization;
use super::repository::OrganizationRepo;
use crate::client::IdentityClient;
use crate::error::OrganizationError;
use crate::modules::department::Department;
use crate::modules::department::DepartmentRepo;
use crate::modules::employee::{Employee, EmployeeRepo};
use crate::modules::position::PositionRepo;
use crate::error::Result;
use sqlxplus::Crud;
use sqlxplus::DbPool;
use std::sync::Arc;

/// 组织 Service
pub struct OrganizationService {
    db_pool: Arc<DbPool>,
}

impl OrganizationService {
    /// 创建新的 OrganizationService
    pub fn new(db_pool: Arc<DbPool>) -> Self {
        Self { db_pool }
    }

    /// 分页查询组织
    pub async fn find_page(
        &self,
        tenant_id: i64,
        req: ListOrganizationsQuery,
    ) -> Result<(Vec<Organization>, i64)> {
        use sqlxplus::QueryBuilder;

        let mut builder =
            QueryBuilder::new("SELECT * FROM organization").and_eq("tenant_id", tenant_id);

        if let Some(ref keyword) = req.keyword {
            if !keyword.is_empty() {
                builder = builder.and_group(|mut g| {
                    g = g.or_like("name", keyword);
                    g = g.or_like("code", keyword);
                    g
                });
            }
        }

        if let Some(status) = req.status {
            builder = builder.and_eq("status", status);
        }

        builder = builder.order_by("sort_order", true).order_by("id", true);

        // 使用 cursor 作为页码，默认为 1
        let page_num = req.page.cursor.unwrap_or(1);
        let page_size = req.page.page_size;

        let result =
            Organization::paginate(self.db_pool.mysql_pool(), builder, page_num, page_size)
                .await
                .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok((result.items, result.total as i64))
    }

    /// 获取组织树
    pub async fn get_tree(&self, tenant_id: i64) -> Result<Vec<OrganizationTreeNode>> {
        use sqlxplus::QueryBuilder;
        let builder = QueryBuilder::new("SELECT * FROM organization")
            .and_eq("tenant_id", tenant_id)
            .order_by("sort_order", true)
            .order_by("id", true);

        let all_orgs = Organization::find_all(self.db_pool.mysql_pool(), Some(builder))
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        let root_orgs: Vec<&Organization> =
            all_orgs.iter().filter(|o| o.parent_id.is_none()).collect();

        let mut tree = Vec::new();
        for root in root_orgs {
            let node = self.build_tree_node(root, &all_orgs);
            tree.push(node);
        }

        Ok(tree)
    }

    /// 递归构建组织树节点
    fn build_tree_node(
        &self,
        org: &Organization,
        all_orgs: &[Organization],
    ) -> OrganizationTreeNode {
        let org_id = org.id.unwrap_or(0);
        let children: Vec<OrganizationTreeNode> = all_orgs
            .iter()
            .filter(|o| o.parent_id == Some(org_id))
            .map(|o| self.build_tree_node(o, all_orgs))
            .collect();

        OrganizationTreeNode {
            organization: OrganizationResponse {
                id: org.id.unwrap_or(0),
                tenant_id: org.tenant_id,
                parent_id: org.parent_id,
                code: org.code.clone(),
                name: org.name.clone(),
                short_name: org.short_name.clone(),
                r#type: org.r#type,
                logo: org.logo.clone(),
                description: org.description.clone(),
                sort_order: org.sort_order,
                status: org.status,
            },
            children,
        }
    }

    /// 创建组织
    ///
    /// **顶级组织**（parent_id == None）：
    ///   1. gRPC 调用 ms-identity 创建租户 → 获得 tenant_id
    ///   2. 本地创建 organization + 根部门
    ///   3. gRPC 调用 ms-identity 初始化角色 + 分配 owner 角色
    ///   4. 本地创建创建人的员工记录
    ///
    /// **子组织**（parent_id != None）：
    ///   使用传入的 tenant_id，走原有逻辑
    pub async fn create(
        &self,
        tenant_id: i64,
        req: CreateOrganizationRequest,
        created_by: Option<i64>,
    ) -> Result<i64> {
        let is_top_level = req.parent_id.is_none();
        let user_id = created_by.unwrap_or(0);

        // ── 1. 确定实际 tenant_id ──────────────────────────────
        let actual_tenant_id = if is_top_level {
            // 顶级组织：gRPC 创建租户
            let resp = IdentityClient::create_tenant_for_org(
                req.name.clone(),
                user_id,
                req.contact_name.clone().unwrap_or_default(),
                req.contact_mobile.clone(),
            )
            .await
            .map_err(|e| {
                OrganizationError::BusinessConflict(format!("创建租户失败: {}", e))
            })?;

            if !resp.success {
                return Err(OrganizationError::BusinessConflict(
                    resp.message.clone(),
                )
                .into());
            }

            resp.tenant_id
        } else {
            // 子组织：使用传入的 tenant_id，并验证父组织存在
            let parent_id = req.parent_id.unwrap();
            let parent = Organization::find_by_id(self.db_pool.mysql_pool(), parent_id)
                .await
                .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
            if parent.is_none() {
                return Err(OrganizationError::OrganizationNotFound.into());
            }
            tenant_id
        };

        // 检查编码是否已存在
        if let Some(_) = OrganizationRepo::find_by_tenant_and_code(
            self.db_pool.mysql_pool(),
            actual_tenant_id,
            &req.code,
        )
        .await?
        {
            return Err(OrganizationError::OrganizationExists.into());
        }

        // ── 2. 创建组织 + 根部门（事务） ──────────────────────
        let now = chrono::Utc::now().timestamp_millis();
        let mut org = Organization::default();
        org.tenant_id = actual_tenant_id;
        org.parent_id = req.parent_id;
        org.code = req.code.clone();
        org.name = req.name.clone();
        org.short_name = req.short_name;
        org.r#type = req.r#type;
        org.logo = req.logo;
        org.description = req.description;
        org.sort_order = req.sort_order.or(Some(0));
        org.status = Some(1);
        org.created_by = created_by;
        org.created_at = Some(now);
        org.updated_at = Some(now);

        let org_id = sqlxplus::with_transaction(&self.db_pool, |tx| {
            Box::pin(async move {
                let org_id = org.insert(tx.as_mysql_executor()).await?;

                // 自动创建根部门
                let mut dept = Department::default();
                dept.tenant_id = actual_tenant_id;
                dept.org_id = org_id;
                dept.parent_id = None;
                dept.code = org.code.clone();
                dept.name = org.name.clone();
                dept.full_name = Some(dept.name.clone());
                dept.path = Some("/".to_string());
                dept.level = Some(1);
                dept.sort_order = Some(0);
                dept.status = Some(1);
                dept.created_by = created_by;
                dept.created_at = Some(now);
                dept.updated_at = Some(now);
                dept.is_deleted = Some(0);

                let dept_id = dept.insert(tx.as_mysql_executor()).await?;

                // 更新部门路径包含 ID
                let mut update_dept = Department::default();
                update_dept.id = Some(dept_id);
                update_dept.path = Some(format!("/{}/", dept_id));
                update_dept.update(tx.as_mysql_executor()).await?;

                Ok(org_id)
            })
        })
        .await
        .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        // ── 3. 顶级组织：初始化角色 + 分配 owner + 创建员工 ──
        if is_top_level && user_id > 0 {
            // 初始化角色
            let roles_resp = IdentityClient::init_org_roles(
                actual_tenant_id,
                org_id,
                user_id,
            )
            .await
            .map_err(|e| {
                OrganizationError::BusinessConflict(format!("初始化角色失败: {}", e))
            })?;

            if roles_resp.success {
                // 分配 owner 角色给创建人
                let assign_resp = IdentityClient::assign_role(
                    user_id,
                    roles_resp.owner_role_id,
                    "owner".to_string(),
                    actual_tenant_id,
                )
                .await;

                if let Err(e) = assign_resp {
                    tracing::warn!("分配 owner 角色失败（不影响组织创建）: {}", e);
                }
            }

            // 为创建人在该组织下创建员工记录
            let mut emp = Employee::default();
            emp.tenant_id = actual_tenant_id;
            emp.org_id = org_id;
            emp.user_id = user_id;
            emp.employee_no = format!("EMP-{}", user_id);
            emp.name = req.contact_name.clone().unwrap_or_default();
            emp.status = Some(1);
            emp.sort_order = Some(0);
            emp.created_by = created_by;
            emp.created_at = Some(now);
            emp.updated_at = Some(now);

            if let Err(e) = emp.insert(self.db_pool.mysql_pool()).await {
                tracing::warn!("创建创建人员工记录失败（不影响组织创建）: {}", e);
            }
        }

        Ok(org_id)
    }

    /// 获取组织详情
    pub async fn get_by_id(&self, id: i64) -> Result<Organization> {
        let org = Organization::find_by_id(self.db_pool.mysql_pool(), id)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;
        org.ok_or_else(|| OrganizationError::OrganizationNotFound.into())
    }

    /// 获取租户下的所有组织
    pub async fn list_by_tenant(&self, tenant_id: i64) -> Result<Vec<Organization>> {
        OrganizationRepo::find_by_tenant_id(self.db_pool.mysql_pool(), tenant_id).await
    }

    /// 更新组织
    pub async fn update(
        &self,
        id: i64,
        req: UpdateOrganizationRequest,
        updated_by: Option<i64>,
    ) -> Result<()> {
        // 获取现有组织
        let existing = self.get_by_id(id).await?;

        let mut org = Organization::default();
        org.id = Some(id);
        org.tenant_id = existing.tenant_id; // 保留原有租户ID
        org.name = req.name.unwrap_or(existing.name);
        org.short_name = req.short_name.or(existing.short_name);
        org.r#type = req.r#type.or(existing.r#type);
        org.logo = req.logo.or(existing.logo);
        org.description = req.description.or(existing.description);
        org.sort_order = req.sort_order.or(existing.sort_order);
        org.status = req.status.or(existing.status);
        org.updated_by = updated_by;
        org.updated_at = Some(chrono::Utc::now().timestamp_millis());

        org.update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 删除组织
    pub async fn delete(&self, id: i64) -> Result<()> {
        // 检查组织是否存在
        let _ = self.get_by_id(id).await?;

        // 检查是否有子组织
        if OrganizationRepo::has_children(self.db_pool.mysql_pool(), id).await? {
            return Err(
                OrganizationError::BusinessConflict("存在下级组织，无法删除".to_string()).into(),
            );
        }

        // 检查是否有员工
        if EmployeeRepo::count_by_org_id(self.db_pool.mysql_pool(), id).await? > 0 {
            return Err(
                OrganizationError::BusinessConflict("组织下存在员工，无法删除".to_string()).into(),
            );
        }

        // 检查是否有岗位
        if PositionRepo::count_by_org_id(self.db_pool.mysql_pool(), id).await? > 0 {
            return Err(
                OrganizationError::BusinessConflict("组织下存在岗位，无法删除".to_string()).into(),
            );
        }

        // 检查是否有子部门（除了根部门）
        // 这里我们可以查询所有部门，如果数量 > 1，说明有子部门
        // 如果数量 == 1，说明只有根部门（或残留的一个部门），可以一起删除
        let depts = DepartmentRepo::find_by_org_id(self.db_pool.mysql_pool(), id).await?;
        if depts.len() > 1 {
            return Err(OrganizationError::BusinessConflict(
                "组织下存在下级部门，无法删除".to_string(),
            )
            .into());
        }

        // 开启事务进行删除
        sqlxplus::with_transaction(&self.db_pool, |tx| {
            Box::pin(async move {
                // 删除关联的部门（通常是根部门）
                for dept in depts {
                    if let Some(dept_id) = dept.id {
                        Department::delete_by_id(tx.as_mysql_executor(), dept_id).await?;
                    }
                }

                Organization::delete_by_id(tx.as_mysql_executor(), id).await?;

                Ok(())
            })
        })
        .await
        .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// 获取租户下的组织数量
    pub async fn count_by_tenant(&self, tenant_id: i64) -> Result<i64> {
        OrganizationRepo::count_by_tenant_id(self.db_pool.mysql_pool(), tenant_id).await
    }
}