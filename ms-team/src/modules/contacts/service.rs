use std::collections::HashMap;
use std::sync::Arc;
use tracing::warn;

use sqlxplus::DbPool;
use sqlxplus::Crud;

use crate::config::ContactsConfig;
use crate::error::{OrganizationError, Result};
use crate::middleware::CurrentUser;
use crate::modules::department::{Department, DepartmentRepo, DepartmentService};
use crate::modules::employee::{
    Employee, EmployeeDepartmentRepo, EmployeeDepartmentService, EmployeePositionRepo,
    EmployeePositionService, EmployeeRepo, EmployeeService,
};
use crate::modules::organization::OrganizationService;
use crate::modules::position::Position;

use super::model::dto::*;
use super::permission::port::{ContactsPermission, ContactsViewer, FieldAction, FieldRestrictions};
use super::search::port::{EmployeeSearchPort, SearchCriteria, SearchDocument};

/// 通讯录聚合查询服务
pub struct ContactsService {
    db_pool: Arc<DbPool>,
    organization_service: Arc<OrganizationService>,
    department_service: Arc<DepartmentService>,
    employee_service: Arc<EmployeeService>,
    employee_department_service: Arc<EmployeeDepartmentService>,
    employee_position_service: Arc<EmployeePositionService>,
    search_port: Arc<dyn EmployeeSearchPort>,
    permission: Arc<dyn ContactsPermission>,
    config: ContactsConfig,
}

impl ContactsService {
    pub fn new(
        db_pool: Arc<DbPool>,
        organization_service: Arc<OrganizationService>,
        department_service: Arc<DepartmentService>,
        employee_service: Arc<EmployeeService>,
        employee_department_service: Arc<EmployeeDepartmentService>,
        employee_position_service: Arc<EmployeePositionService>,
        search_port: Arc<dyn EmployeeSearchPort>,
        permission: Arc<dyn ContactsPermission>,
        config: ContactsConfig,
    ) -> Self {
        Self {
            db_pool,
            organization_service,
            department_service,
            employee_service,
            employee_department_service,
            employee_position_service,
            search_port,
            permission,
            config,
        }
    }

    /// 从当前用户请求构建查看者身份快照
    fn build_viewer(&self, current_user: &CurrentUser) -> ContactsViewer {
        ContactsViewer {
            user_id: current_user.user_id,
            tenant_id: current_user.tenant_id,
            org_id: current_user.org_id.unwrap_or(0),
            employee_id: current_user.employee_id,
            department_ids: vec![], // Phase 1 暂不查询，Phase 2+ 从缓存加载
            is_admin: false,        // TODO: 根据实际鉴权中间件补充
        }
    }

    /// 通讯录入口：组织信息 + 根部门列表
    pub async fn get_entry(
        &self,
        current_user: CurrentUser,
        req: ContactsEntryQuery,
    ) -> Result<ContactsEntryResponse> {
        let viewer = self.build_viewer(&current_user);

        // 1. 查询组织信息
        let org = self
            .organization_service
            .get_by_id(req.org_id)
            .await?;

        // 2. 查询根部门
        let root_depts = DepartmentRepo::find_root_by_org_id(self.db_pool.mysql_pool(), req.org_id)
            .await?;

        // 3. 过滤可见根部门 (Layer 1)
        let root_dept_ids: Vec<i64> = root_depts.iter().filter_map(|d| d.id).collect();
        let visible_dept_ids = self
            .permission
            .filter_visible_departments(&viewer, &root_dept_ids)
            .await;

        // 4. 构建根部门摘要
        let mut department_summaries = Vec::new();
        for dept in root_depts {
            if let Some(id) = dept.id {
                if visible_dept_ids.contains(&id) {
                    department_summaries.push(self.build_department_summary(&viewer, &dept).await?);
                }
            }
        }

        // 5. 计算组织总可见人数
        let total_count = EmployeeRepo::count_by_org_id(self.db_pool.mysql_pool(), req.org_id).await?;
        // Phase 1 直接返回实际总数，Phase 2+ 需要减去不可见人数
        let total_visible_count = self.permission.count_visible_members(&viewer, 0, total_count).await;

        Ok(ContactsEntryResponse {
            organization: OrganizationBrief {
                id: org.id.unwrap_or(0),
                name: org.name,
                logo: org.logo,
            },
            departments: department_summaries,
            total_member_count: total_visible_count,
        })
    }

    /// 构建部门摘要（含成员数量和 leader）
    async fn build_department_summary(
        &self,
        viewer: &ContactsViewer,
        dept: &Department,
    ) -> Result<DepartmentSummary> {
        let dept_id = dept.id.unwrap_or(0);
        let path = dept.path.as_deref().unwrap_or("");

        // 检查是否有子部门
        let has_children = DepartmentRepo::has_children(self.db_pool.mysql_pool(), dept_id).await?;

        // 统计可见成员数（包含子部门）
        let actual_count = DepartmentRepo::count_employees_by_dept_id(
            self.db_pool.mysql_pool(),
            dept_id,
            path,
            viewer.tenant_id,
        )
        .await?;
        let visible_count = self
            .permission
            .count_visible_members(viewer, dept_id, actual_count)
            .await;

        // 查找负责人
        let leader = self.get_leader_brief(dept_id, viewer).await?;

        Ok(DepartmentSummary {
            id: dept_id,
            name: dept.name.clone(),
            has_children,
            member_count: visible_count,
            leader,
        })
    }

    /// 获取部门负责人简要信息
    async fn get_leader_brief(
        &self,
        dept_id: i64,
        viewer: &ContactsViewer,
    ) -> Result<Option<LeaderBrief>> {
        let pool = self.db_pool.mysql_pool();
        // 获取 is_leader = 1 的记录
        let sql = sqlx::query_as::<_, Employee>(
            r#"
            SELECT e.* 
            FROM employee e
            INNER JOIN employee_department ed ON e.id = ed.employee_id
            WHERE ed.department_id = ? AND ed.is_leader = 1 AND e.is_deleted = 0
            LIMIT 1
            "#,
        )
        .bind(dept_id);

        if let Ok(Some(emp)) = sql.fetch_optional(pool).await {
            // Layer 2: 检查负责人是否对当前用户可见
            if let Some(emp_id) = emp.id {
                if self.permission.is_employee_visible(viewer, emp_id).await {
                    return Ok(Some(LeaderBrief {
                        id: emp_id,
                        name: emp.name,
                        avatar: emp.avatar,
                    }));
                }
            }
        }
        Ok(None)
    }

    /// 部门展开：子部门 + 直属成员预览
    pub async fn get_department(
        &self,
        current_user: CurrentUser,
        dept_id: i64,
    ) -> Result<ContactsDepartmentResponse> {
        let viewer = self.build_viewer(&current_user);

        // 1. Layer 1 权限校验
        if !self.permission.is_department_visible(&viewer, dept_id).await {
            return Err(OrganizationError::ContactsDepartmentNotVisible.into());
        }

        // 获取部门详情
        let dept = self.department_service.get_by_id(dept_id).await?;

        // 2. 查询子部门并过滤
        let children = DepartmentRepo::find_by_parent_id(self.db_pool.mysql_pool(), dept_id).await?;
        let child_ids: Vec<i64> = children.iter().filter_map(|d| d.id).collect();
        let visible_child_ids = self
            .permission
            .filter_visible_departments(&viewer, &child_ids)
            .await;

        let mut child_summaries = Vec::new();
        for child in children {
            if let Some(id) = child.id {
                if visible_child_ids.contains(&id) {
                    child_summaries.push(self.build_department_summary(&viewer, &child).await?);
                }
            }
        }

        // 3. 获取直属成员总数 (用于计算 has_more_members)
        let actual_direct_count =
            DepartmentRepo::count_direct_employees(self.db_pool.mysql_pool(), dept_id).await?;
        let visible_direct_count = self
            .permission
            .count_visible_members(&viewer, dept_id, actual_direct_count)
            .await;

        // 4. 获取前 N 名成员预览（负责人置顶）
        let limit = self.config.dept_preview_members;
        
        let pool = self.db_pool.mysql_pool();

        // 分步查询：先获取部门成员关系（含 is_leader），再批量查员工
        let dept_rels = EmployeeDepartmentRepo::find_by_department_id(pool, dept_id).await?;

        // 收集员工 ID → is_leader 映射
        let mut emp_leader_map: HashMap<i64, bool> = HashMap::new();
        for rel in &dept_rels {
            emp_leader_map.insert(rel.employee_id, rel.is_leader.unwrap_or(0) == 1);
        }

        // 批量查询员工实体（过采样）
        let emp_ids: Vec<i64> = dept_rels.iter().map(|r| r.employee_id).collect();
        let members = if emp_ids.is_empty() {
            vec![]
        } else {
            let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee")
                .and_in("id", emp_ids)
                .order_by("sort_order", true)
                .order_by("id", true);
            Employee::find_all(pool, Some(builder))
                .await
                .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
        };

        // 按 is_leader DESC 排序（负责人置顶）
        let mut sorted_members: Vec<(Employee, bool)> = members
            .into_iter()
            .map(|e| {
                let is_leader = e.id.map(|id| *emp_leader_map.get(&id).unwrap_or(&false)).unwrap_or(false);
                (e, is_leader)
            })
            .collect();
        sorted_members.sort_by(|a, b| b.1.cmp(&a.1));

        // 过滤和限制数量
        let mut member_previews = Vec::new();
        let member_ids: Vec<i64> = sorted_members.iter().filter_map(|(e, _)| e.id).collect();
        
        // Layer 2: 过滤人员
        let visible_member_ids = self
            .permission
            .filter_visible_employees(&viewer, &member_ids)
            .await;
            
        // Layer 3: 批量获取字段权限
        let field_restrictions = self
            .permission
            .get_batch_field_restrictions(&viewer, &member_ids)
            .await;

        for (emp, is_leader) in sorted_members {
            if let Some(id) = emp.id {
                if visible_member_ids.contains(&id) {
                    let restrictions = field_restrictions
                        .get(&id)
                        .cloned()
                        .unwrap_or_else(FieldRestrictions::all_visible);
                        
                    // 查询主岗位名称用于显示
                    let primary_pos = EmployeePositionRepo::find_primary_by_employee_id(pool, id).await?;
                    let mut dept_title = None;
                    if let Some(pos_rel) = primary_pos {
                        if let Ok(Some(pos)) = Position::find_by_id(pool, pos_rel.position_id).await {
                            dept_title = Some(pos.name);
                        }
                    }

                    member_previews.push(self.build_member_preview(emp, is_leader, dept_title, &restrictions));
                    
                    if member_previews.len() >= limit as usize {
                        break;
                    }
                }
            }
        }

        let has_more_members = visible_direct_count > member_previews.len() as i64;

        Ok(ContactsDepartmentResponse {
            department: DepartmentInfo {
                id: dept_id,
                name: dept.name,
                full_name: dept.full_name,
            },
            children: child_summaries,
            members: member_previews,
            direct_member_count: visible_direct_count,
            has_more_members,
        })
    }

    /// 组装单个成员预览（应用脱敏/隐藏规则）
    fn build_member_preview(
        &self,
        emp: Employee,
        is_leader: bool,
        department_title: Option<String>,
        restrictions: &FieldRestrictions,
    ) -> MemberPreview {
        MemberPreview {
            id: emp.id.unwrap_or(0),
            name: emp.name,
            avatar: emp.avatar,
            department_title,
            mobile: match restrictions.mobile {
                FieldAction::Visible => emp.mobile,
                FieldAction::Masked => emp.mobile.map(|m| Self::mask_mobile(&m)),
                FieldAction::Hidden => None,
            },
            email: match restrictions.email {
                FieldAction::Visible => emp.email,
                FieldAction::Masked => emp.email.map(|e| Self::mask_email(&e)),
                FieldAction::Hidden => None,
            },
            is_leader,
        }
    }

    fn mask_mobile(mobile: &str) -> String {
        if mobile.len() == 11 {
            format!("{}****{}", &mobile[0..3], &mobile[7..11])
        } else {
            "***".to_string()
        }
    }

    fn mask_email(email: &str) -> String {
        let parts: Vec<&str> = email.split('@').collect();
        if parts.len() == 2 {
            let name = parts[0];
            let domain = parts[1];
            if name.len() > 2 {
                format!("{}***@{}", &name[0..2], domain)
            } else {
                format!("***@{}", domain)
            }
        } else {
            "***".to_string()
        }
    }

    /// 获取联系人详情
    pub async fn get_employee_detail(
        &self,
        current_user: CurrentUser,
        employee_id: i64,
    ) -> Result<ContactsEmployeeDetailResponse> {
        let viewer = self.build_viewer(&current_user);

        // Layer 2: 人员可见性校验
        if !self.permission.is_employee_visible(&viewer, employee_id).await {
            return Err(OrganizationError::ContactsEmployeeNotVisible.into());
        }

        let emp = self.employee_service.get_by_id(employee_id).await?;

        // Layer 3: 字段可见性
        let restrictions = self
            .permission
            .get_field_restrictions(&viewer, employee_id)
            .await;

        // 查询部门关系
        let dept_rels = self
            .employee_department_service
            .get_by_employee(employee_id)
            .await?;
        
        let pool = self.db_pool.mysql_pool();
        let mut dept_infos = Vec::new();
        for rel in dept_rels {
            if let Ok(Some(dept)) = Department::find_by_id(pool, rel.department_id).await {
                // Layer 1: 部门可见性校验（在详情中隐藏不可见的部门）
                if self.permission.is_department_visible(&viewer, dept.id.unwrap_or(0)).await {
                    dept_infos.push(EmployeeDeptInfo {
                        id: dept.id.unwrap_or(0),
                        name: dept.name,
                        full_name: dept.full_name,
                        is_primary: rel.is_primary.unwrap_or(0) == 1,
                        is_leader: rel.is_leader.unwrap_or(0) == 1,
                    });
                }
            }
        }

        // 查询岗位关系
        let pos_rels = self
            .employee_position_service
            .get_by_employee(employee_id)
            .await?;
            
        let mut pos_infos = Vec::new();
        for rel in pos_rels {
            if let Ok(Some(pos)) = Position::find_by_id(pool, rel.position_id).await {
                pos_infos.push(EmployeePosInfo {
                    id: pos.id.unwrap_or(0),
                    name: pos.name,
                    level: pos.level,
                    is_primary: rel.is_primary.unwrap_or(0) == 1,
                });
            }
        }

        Ok(ContactsEmployeeDetailResponse {
            id: emp.id.unwrap_or(0),
            name: emp.name,
            avatar: emp.avatar,
            employee_no: match restrictions.employee_no {
                FieldAction::Visible => Some(emp.employee_no),
                FieldAction::Masked | FieldAction::Hidden => None,
            },
            gender: emp.gender,
            mobile: match restrictions.mobile {
                FieldAction::Visible => emp.mobile,
                FieldAction::Masked => emp.mobile.map(|m| Self::mask_mobile(&m)),
                FieldAction::Hidden => None,
            },
            email: match restrictions.email {
                FieldAction::Visible => emp.email,
                FieldAction::Masked => emp.email.map(|e| Self::mask_email(&e)),
                FieldAction::Hidden => None,
            },
            status: emp.status,
            hire_date: match restrictions.hire_date {
                FieldAction::Visible => emp.hire_date,
                FieldAction::Masked | FieldAction::Hidden => None,
            },
            departments: dept_infos,
            positions: pos_infos,
        })
    }

    /// 全局搜索
    pub async fn search(
        &self,
        current_user: CurrentUser,
        req: ContactsSearchQuery,
    ) -> Result<ContactsSearchResponse> {
        let viewer = self.build_viewer(&current_user);
        let page = req.page.unwrap_or(1);
        let page_size = req.page_size.unwrap_or(20).clamp(1, 50);
        let offset = (page - 1) * page_size;
        
        // 过采样以应对权限过滤带来的数据截断
        let limit = (page_size as f32 * 1.5) as u32;

        let criteria = SearchCriteria {
            keyword: req.keyword.clone(),
            org_id: req.org_id,
            tenant_id: current_user.tenant_id,
            offset: offset as u32,
            limit,
        };

        match self.search_port.search(criteria).await {
            Ok(result) => {
                // 1. Layer 2: 过滤不可见人员
                let item_ids: Vec<i64> = result.items.iter().map(|d| d.id).collect();
                let visible_ids = self
                    .permission
                    .filter_visible_employees(&viewer, &item_ids)
                    .await;
                
                // 2. Layer 3: 获取字段权限
                let field_restrictions = self
                    .permission
                    .get_batch_field_restrictions(&viewer, &item_ids)
                    .await;

                let mut final_items = Vec::new();
                for doc in result.items {
                    if visible_ids.contains(&doc.id) {
                        let restrictions = field_restrictions
                            .get(&doc.id)
                            .cloned()
                            .unwrap_or_else(FieldRestrictions::all_visible);

                        final_items.push(MemberPreview {
                            id: doc.id,
                            name: doc.name,
                            avatar: doc.avatar,
                            department_title: doc.department_title,
                            mobile: match restrictions.mobile {
                                FieldAction::Visible => doc.mobile,
                                FieldAction::Masked => doc.mobile.map(|m| Self::mask_mobile(&m)),
                                FieldAction::Hidden => None,
                            },
                            email: match restrictions.email {
                                FieldAction::Visible => doc.email,
                                FieldAction::Masked => doc.email.map(|e| Self::mask_email(&e)),
                                FieldAction::Hidden => None,
                            },
                            is_leader: false, // 搜索结果不关心是否 leader
                        });

                        if final_items.len() >= page_size as usize {
                            break;
                        }
                    }
                }

                Ok(ContactsSearchResponse {
                    items: final_items,
                    estimated_total: result.estimated_total,
                    has_next: result.estimated_total > (offset + page_size) as u64,
                    degraded: false,
                })
            }
            Err(e) => {
                warn!("Meilisearch 搜索失败，降级到 MySQL: {}", e);
                self.search_by_mysql_fallback(viewer, req.org_id, &req.keyword, page, page_size).await
            }
        }
    }

    /// MySQL 降级搜索
    async fn search_by_mysql_fallback(
        &self,
        viewer: ContactsViewer,
        org_id: i64,
        keyword: &str,
        page: i64,
        page_size: i64,
    ) -> Result<ContactsSearchResponse> {
        let pool = self.db_pool.mysql_pool();
        let kw = format!("%{}%", keyword);
        
        // 简化版的 fallback，仅提供基本的 LIKE 查询
        let sql = sqlx::query_as::<_, Employee>(
            r#"
            SELECT * FROM employee
            WHERE org_id = ? AND is_deleted = 0 AND status = 1
              AND (name LIKE ? OR employee_no LIKE ? OR mobile LIKE ?)
            ORDER BY id ASC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(org_id)
        .bind(&kw)
        .bind(&kw)
        .bind(&kw)
        .bind(page_size * 2) // 过采样
        .bind((page - 1) * page_size);

        let emps = sql.fetch_all(pool).await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

        // 估算总数 (这里简单返回，不执行 count，因为降级时可能由于 DB 压力过大才降级)
        let count_sql = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) FROM employee
            WHERE org_id = ? AND is_deleted = 0 AND status = 1
              AND (name LIKE ? OR employee_no LIKE ? OR mobile LIKE ?)
            "#,
        )
        .bind(org_id)
        .bind(&kw)
        .bind(&kw)
        .bind(&kw);

        let total = count_sql.fetch_one(pool).await.unwrap_or(0);

        let item_ids: Vec<i64> = emps.iter().filter_map(|e| e.id).collect();
        let visible_ids = self.permission.filter_visible_employees(&viewer, &item_ids).await;
        let field_restrictions = self.permission.get_batch_field_restrictions(&viewer, &item_ids).await;

        let mut final_items = Vec::new();
        for emp in emps {
            if let Some(id) = emp.id {
                if visible_ids.contains(&id) {
                    let restrictions = field_restrictions
                        .get(&id)
                        .cloned()
                        .unwrap_or_else(FieldRestrictions::all_visible);

                    // 简化的 fallback 不联表查询 department_title
                    final_items.push(self.build_member_preview(emp, false, None, &restrictions));

                    if final_items.len() >= page_size as usize {
                        break;
                    }
                }
            }
        }

        Ok(ContactsSearchResponse {
            items: final_items,
            estimated_total: total as u64,
            has_next: total > (page * page_size),
            degraded: true,
        })
    }

    /// 部门成员分页查询
    pub async fn get_department_members(
        &self,
        current_user: CurrentUser,
        dept_id: i64,
        req: ContactsMembersQuery,
    ) -> Result<ContactsMemberPageResponse> {
        let viewer = self.build_viewer(&current_user);

        // Layer 1
        if !self.permission.is_department_visible(&viewer, dept_id).await {
            return Err(OrganizationError::ContactsDepartmentNotVisible.into());
        }

        let include_children = req.include_children.unwrap_or(false);
        let page = req.page.unwrap_or(1);
        let page_size = req.page_size.unwrap_or(20);
        let offset = (page - 1) * page_size;

        let pool = self.db_pool.mysql_pool();

        let (total, members) = if include_children {
            let dept = self.department_service.get_by_id(dept_id).await?;
            let path = dept.path.unwrap_or_default();
            
            // 保护机制：检查总数
            let actual_count = DepartmentRepo::count_employees_by_dept_id(
                pool, dept_id, &path, viewer.tenant_id
            ).await?;

            if actual_count > self.config.include_children_max as i64 {
                return Err(OrganizationError::ContactsMemberCountExceeded(self.config.include_children_max).into());
            }

            let path_prefix = format!("{}%", path.trim_end_matches('/'));

            // 分步查询：先获取子树下的员工 ID + is_leader 关系
            let rels: Vec<(i64, i16)> = sqlx::query_as(
                r#"
                SELECT DISTINCT ed.employee_id, ed.is_leader
                FROM employee_department ed
                INNER JOIN department d ON ed.department_id = d.id
                WHERE d.path LIKE ? AND d.is_deleted = 0
                ORDER BY ed.employee_id ASC
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(&path_prefix)
            .bind(page_size * 2)
            .bind(offset)
            .fetch_all(pool)
            .await
            .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

            let emp_ids: Vec<i64> = rels.iter().map(|(id, _)| *id).collect();
            let leader_map: HashMap<i64, bool> = rels.iter().map(|(id, is_leader)| (*id, *is_leader == 1)).collect();

            let emps = if emp_ids.is_empty() {
                vec![]
            } else {
                let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee")
                    .and_in("id", emp_ids)
                    .order_by("sort_order", true)
                    .order_by("id", true);
                Employee::find_all(pool, Some(builder))
                    .await
                    .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
            };

            let members: Vec<(Employee, bool)> = emps.into_iter().map(|e| {
                let is_leader = e.id.map(|id| *leader_map.get(&id).unwrap_or(&false)).unwrap_or(false);
                (e, is_leader)
            }).collect();

            (actual_count, members)
        } else {
            let actual_count = DepartmentRepo::count_direct_employees(pool, dept_id).await?;
            
            // 分步查询：先获取直属员工关系
            let dept_rels = EmployeeDepartmentRepo::find_by_department_id(pool, dept_id).await?;
            let emp_ids: Vec<i64> = dept_rels.iter().map(|r| r.employee_id).collect();
            let leader_map: HashMap<i64, bool> = dept_rels.iter().map(|r| (r.employee_id, r.is_leader.unwrap_or(0) == 1)).collect();

            let emps = if emp_ids.is_empty() {
                vec![]
            } else {
                let builder = sqlxplus::QueryBuilder::new("SELECT * FROM employee")
                    .and_in("id", emp_ids)
                    .order_by("sort_order", true)
                    .order_by("id", true);
                Employee::find_all(pool, Some(builder))
                    .await
                    .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?
            };

            // 按 is_leader DESC 排序（负责人置顶）
            let mut members: Vec<(Employee, bool)> = emps.into_iter().map(|e| {
                let is_leader = e.id.map(|id| *leader_map.get(&id).unwrap_or(&false)).unwrap_or(false);
                (e, is_leader)
            }).collect();
            members.sort_by(|a, b| b.1.cmp(&a.1));

            // 手动分页
            let start = offset as usize;
            let end = (start + (page_size * 2) as usize).min(members.len());
            let members = if start < members.len() {
                members[start..end].to_vec()
            } else {
                vec![]
            };

            (actual_count, members)
        };

        // 过滤和权限
        let visible_total = self.permission.count_visible_members(&viewer, dept_id, total).await;
        
        let item_ids: Vec<i64> = members.iter().filter_map(|(e, _)| e.id).collect();
        let visible_ids = self.permission.filter_visible_employees(&viewer, &item_ids).await;
        let field_restrictions = self.permission.get_batch_field_restrictions(&viewer, &item_ids).await;

        let mut final_items = Vec::new();
        for (emp, is_leader) in members {
            if let Some(id) = emp.id {
                if visible_ids.contains(&id) {
                    let restrictions = field_restrictions
                        .get(&id)
                        .cloned()
                        .unwrap_or_else(FieldRestrictions::all_visible);

                    // 查询职位名称
                    let primary_pos = EmployeePositionRepo::find_primary_by_employee_id(pool, id).await?;
                    let mut dept_title = None;
                    if let Some(pos_rel) = primary_pos {
                        if let Ok(Some(pos)) = Position::find_by_id(pool, pos_rel.position_id).await {
                            dept_title = Some(pos.name);
                        }
                    }

                    final_items.push(self.build_member_preview(emp, is_leader, dept_title, &restrictions));

                    if final_items.len() >= page_size as usize {
                        break;
                    }
                }
            }
        }

        Ok(ContactsMemberPageResponse {
            items: final_items,
            total: visible_total,
            has_next: visible_total > (page * page_size),
        })
    }

    /// 全量重建搜索索引
    pub async fn rebuild_index(&self, org_id: Option<i64>) -> Result<()> {
        let pool = self.db_pool.mysql_pool();
        let batch_size = 500;
        let mut offset = 0;

        loop {
            // 构建查询
            let mut sql = String::from("SELECT * FROM employee WHERE is_deleted = 0 AND status = 1");
            if let Some(oid) = org_id {
                sql.push_str(&format!(" AND org_id = {}", oid));
            }
            sql.push_str(&format!(" ORDER BY id ASC LIMIT {} OFFSET {}", batch_size, offset));

            let emps: Vec<Employee> = sqlx::query_as(&sql)
                .fetch_all(pool)
                .await
                .map_err(|e| OrganizationError::DatabaseError(e.to_string()))?;

            if emps.is_empty() {
                break;
            }

            let mut docs = Vec::new();
            for emp in &emps {
                if let Some(doc) = self.build_search_document_from_db(emp.id.unwrap_or(0)).await? {
                    docs.push(doc);
                }
            }

            if !docs.is_empty() {
                if let Err(e) = self.search_port.batch_index(docs).await {
                    warn!("批量同步索引失败 (offset: {}): {}", offset, e);
                }
            }

            offset += batch_size;
        }

        Ok(())
    }

    /// 根据 employee_id 从数据库组装 SearchDocument
    pub async fn build_search_document_from_db(&self, employee_id: i64) -> Result<Option<SearchDocument>> {
        let pool = self.db_pool.mysql_pool();
        if let Ok(Some(emp)) = Employee::find_by_id(pool, employee_id).await {
            if emp.is_deleted.unwrap_or(0) == 1 {
                return Ok(None);
            }

            let primary_dept_rel = EmployeeDepartmentRepo::find_primary_by_employee_id(pool, employee_id).await?;
            let primary_pos_rel = EmployeePositionRepo::find_primary_by_employee_id(pool, employee_id).await?;

            let mut dept_name = None;
            let mut pos_name = None;

            if let Some(rel) = primary_dept_rel {
                if let Ok(Some(dept)) = Department::find_by_id(pool, rel.department_id).await {
                    dept_name = Some(dept.name);
                }
            }

            if let Some(rel) = primary_pos_rel {
                if let Ok(Some(pos)) = Position::find_by_id(pool, rel.position_id).await {
                    pos_name = Some(pos.name);
                }
            }

            Ok(Some(SearchDocument {
                id: employee_id,
                tenant_id: emp.tenant_id,
                org_id: emp.org_id,
                employee_no: Some(emp.employee_no),
                name: emp.name,
                avatar: emp.avatar,
                mobile: emp.mobile,
                email: emp.email,
                department_title: pos_name,
                primary_department_name: dept_name,
                status: emp.status.unwrap_or(1),
            }))
        } else {
            Ok(None)
        }
    }
}
