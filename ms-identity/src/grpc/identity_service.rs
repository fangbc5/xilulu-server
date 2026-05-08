// 身份验证 gRPC 服务实现

use crate::error::IdentityError;
use crate::modules::auth::{Role, RoleService};
use crate::modules::plan::repository::PlanRepo;
use crate::modules::tenant::{SystemTenant, Tenant, TenantService};
use crate::modules::user::{TenantUserRel, User, UserRole, UserService, UserTenantService};
use chrono::{Duration, Utc};
use sqlxplus::Crud;
use std::collections::HashMap;
use std::sync::Arc;
use tonic::{Request, Response, Status};

// 生成的 proto 代码
pub mod identity {
    tonic::include_proto!("identity");
}

use identity::{
    identity_service_server::{IdentityService, IdentityServiceServer},
    BatchGetUserInfoRequest, BatchGetUserInfoResponse, GetUserInfoRequest, RegisterUserRequest,
    RegisterUserResponse, TenantInfo, UserInfo, UserInfoResponse, UserTenantsResponse,
    VerifyRequest, VerifyResponse, SearchUserRpcRequest, SearchUserRpcResponse,
    CreateTenantForOrgRequest, CreateTenantForOrgResponse,
    InitOrgRolesRequest, InitOrgRolesResponse,
    AssignRoleRequest, AssignRoleResponse,
};

/// 身份验证服务实现
pub struct IdentityServiceImpl {
    user_service: Arc<UserService>,
    user_tenant_service: Arc<UserTenantService>,
    tenant_service: Arc<TenantService>,
    role_service: Arc<RoleService>,
}

impl IdentityServiceImpl {
    pub fn new(
        user_service: Arc<UserService>,
        user_tenant_service: Arc<UserTenantService>,
        tenant_service: Arc<TenantService>,
        role_service: Arc<RoleService>,
    ) -> Self {
        Self {
            user_service,
            user_tenant_service,
            tenant_service,
            role_service,
        }
    }

    /// 获取 gRPC 服务实例（用于注册到服务器）
    pub fn server(
        user_service: Arc<UserService>,
        user_tenant_service: Arc<UserTenantService>,
        tenant_service: Arc<TenantService>,
        role_service: Arc<RoleService>,
    ) -> IdentityServiceServer<IdentityServiceImpl> {
        IdentityServiceServer::new(Self::new(
            user_service,
            user_tenant_service,
            tenant_service,
            role_service,
        ))
    }
}

#[tonic::async_trait]
impl IdentityService for IdentityServiceImpl {
    /// 统一验证方法（支持用户名/邮箱/手机号 + 密码/验证码）
    async fn verify(
        &self,
        request: Request<VerifyRequest>,
    ) -> Result<Response<VerifyResponse>, Status> {
        let req = request.into_inner();

        // 直接调用 verify_password，它会自动处理所有登录方式
        let username = if req.username.is_empty() {
            None
        } else {
            Some(req.username.as_str())
        };
        let password = if req.password.is_empty() {
            None
        } else {
            Some(req.password.as_str())
        };
        let mobile = if req.mobile.is_empty() {
            None
        } else {
            Some(req.mobile.as_str())
        };
        let email = if req.email.is_empty() {
            None
        } else {
            Some(req.email.as_str())
        };
        let region = if req.region.is_empty() {
            None
        } else {
            Some(req.region.as_str())
        };

        match self
            .user_service
            .verify_password(username, password, mobile, email, region)
            .await
        {
            Ok(user) => {
                let user_info = convert_user_to_proto(&user);
                Ok(Response::new(VerifyResponse {
                    success: true,
                    message: "验证成功".to_string(),
                    user: Some(user_info),
                }))
            }
            Err(e) => {
                let error_msg = if let Some(identity_err) = e.downcast_ref::<IdentityError>() {
                    identity_err.to_string()
                } else {
                    e.to_string()
                };
                Ok(Response::new(VerifyResponse {
                    success: false,
                    message: error_msg,
                    user: None,
                }))
            }
        }
    }

    /// 获取用户信息
    async fn get_user_info(
        &self,
        request: Request<GetUserInfoRequest>,
    ) -> Result<Response<UserInfoResponse>, Status> {
        let req = request.into_inner();

        match self.user_service.get_user_info(req.user_id).await {
            Ok(user) => {
                let user_info = convert_user_to_proto(&user);
                Ok(Response::new(UserInfoResponse {
                    success: true,
                    message: "获取成功".to_string(),
                    user: Some(user_info),
                }))
            }
            Err(e) => {
                let error_msg = if let Some(identity_err) = e.downcast_ref::<IdentityError>() {
                    identity_err.to_string()
                } else {
                    e.to_string()
                };
                Ok(Response::new(UserInfoResponse {
                    success: false,
                    message: error_msg,
                    user: None,
                }))
            }
        }
    }

    /// 批量获取用户信息
    async fn batch_get_user_info(
        &self,
        request: Request<BatchGetUserInfoRequest>,
    ) -> Result<Response<BatchGetUserInfoResponse>, Status> {
        let req = request.into_inner();

        match self.user_service.get_users_by_ids(&req.user_ids).await {
            Ok(users) => {
                let user_infos: Vec<UserInfo> = users.iter().map(convert_user_to_proto).collect();
                Ok(Response::new(BatchGetUserInfoResponse {
                    success: true,
                    message: "获取成功".to_string(),
                    users: user_infos,
                }))
            }
            Err(e) => {
                let error_msg = if let Some(identity_err) = e.downcast_ref::<IdentityError>() {
                    identity_err.to_string()
                } else {
                    e.to_string()
                };
                Ok(Response::new(BatchGetUserInfoResponse {
                    success: false,
                    message: error_msg,
                    users: vec![],
                }))
            }
        }
    }

    /// 获取用户租户列表
    async fn get_user_tenants(
        &self,
        request: Request<GetUserInfoRequest>,
    ) -> Result<Response<UserTenantsResponse>, Status> {
        let req = request.into_inner();

        match self.user_tenant_service.get_user_tenants(req.user_id).await {
            Ok(tenant_rels) => {
                if tenant_rels.is_empty() {
                    return Ok(Response::new(UserTenantsResponse {
                        success: true,
                        message: "获取成功".to_string(),
                        tenants: vec![],
                    }));
                }

                // 批量查询租户信息
                let tenant_ids: Vec<i64> = tenant_rels.iter().map(|rel| rel.tenant_id).collect();
                let tenants = match self.tenant_service.get_tenants_by_ids(&tenant_ids).await {
                    Ok(tenants) => tenants,
                    Err(e) => {
                        return Ok(Response::new(UserTenantsResponse {
                            success: false,
                            message: format!("批量查询租户失败: {}", e),
                            tenants: vec![],
                        }));
                    }
                };

                // 构建租户ID到租户信息的映射
                let tenant_map: HashMap<i64, &Tenant> =
                    tenants.iter().map(|t| (t.id.unwrap_or(0), t)).collect();

                // 组合关联关系和租户信息
                let tenant_infos: Vec<TenantInfo> = tenant_rels
                    .into_iter()
                    .filter_map(|rel| {
                        tenant_map.get(&rel.tenant_id).map(|tenant| TenantInfo {
                            user_id: rel.user_id,
                            tenant_id: rel.tenant_id,
                            is_owner: rel.is_owner.unwrap_or(0) as i32,
                            name: tenant.name.clone(),
                            status: rel.status.unwrap_or(0) as i32,
                        })
                    })
                    .collect();

                Ok(Response::new(UserTenantsResponse {
                    success: true,
                    message: "获取成功".to_string(),
                    tenants: tenant_infos,
                }))
            }
            Err(e) => {
                let error_msg = if let Some(identity_err) = e.downcast_ref::<IdentityError>() {
                    identity_err.to_string()
                } else {
                    e.to_string()
                };
                Ok(Response::new(UserTenantsResponse {
                    success: false,
                    message: error_msg,
                    tenants: vec![],
                }))
            }
        }
    }

    /// 用户注册
    async fn register_user(
        &self,
        request: Request<RegisterUserRequest>,
    ) -> Result<Response<RegisterUserResponse>, Status> {
        let req = request.into_inner();

        let username = if req.username.is_empty() {
            None
        } else {
            Some(req.username.as_str())
        };
        let password = if req.password.is_empty() {
            None
        } else {
            Some(req.password.as_str())
        };
        let email = if req.email.is_empty() {
            None
        } else {
            Some(req.email.as_str())
        };
        let mobile = if req.mobile.is_empty() {
            None
        } else {
            Some(req.mobile.as_str())
        };
        let nick_name = if req.nick_name.is_empty() {
            None
        } else {
            Some(req.nick_name.as_str())
        };
        let avatar = if req.avatar.is_empty() {
            None
        } else {
            Some(req.avatar.as_str())
        };
        let region = if req.region.is_empty() {
            None
        } else {
            Some(req.region.as_str())
        };

        match self
            .user_service
            .register_user(username, password, email, mobile, nick_name, avatar, region)
            .await
        {
            Ok(user_id) => Ok(Response::new(RegisterUserResponse {
                success: true,
                message: "注册成功".to_string(),
                user_id,
            })),
            Err(e) => {
                let error_msg = if let Some(identity_err) = e.downcast_ref::<IdentityError>() {
                    identity_err.to_string()
                } else {
                    e.to_string()
                };
                Ok(Response::new(RegisterUserResponse {
                    success: false,
                    message: error_msg,
                    user_id: 0,
                }))
            }
        }
    }

    /// 搜索用户
    async fn search_user(
        &self,
        request: Request<SearchUserRpcRequest>,
    ) -> Result<Response<SearchUserRpcResponse>, Status> {
        let req = request.into_inner();
        let keyword = req.keyword.trim();

        if keyword.is_empty() {
            return Ok(Response::new(SearchUserRpcResponse {
                success: false,
                message: "搜索关键词不能为空".to_string(),
                user: None,
            }));
        }

        let result = if keyword.contains('@') {
            self.user_service.find_by_email(keyword).await
        } else if keyword.chars().all(|c| c.is_ascii_digit()) && keyword.len() >= 11 {
            self.user_service.find_by_mobile(keyword).await
        } else {
            self.user_service.find_by_username(keyword).await
        };

        match result {
            Ok(user) => Ok(Response::new(SearchUserRpcResponse {
                success: true,
                message: "获取成功".to_string(),
                user: Some(convert_user_to_proto(&user)),
            })),
            Err(_) => Ok(Response::new(SearchUserRpcResponse {
                success: false,
                message: "用户不存在".to_string(),
                user: None,
            })),
        }
    }

    /// 为组织创建租户（顶级组织时调用）
    /// 创建 Tenant + TenantUserRel(is_owner=1)
    async fn create_tenant_for_organization(
        &self,
        request: Request<CreateTenantForOrgRequest>,
    ) -> Result<Response<CreateTenantForOrgResponse>, Status> {
        let req = request.into_inner();

        // 查找 Free 套餐
        let free_plan = match PlanRepo::find_by_name(
            self.tenant_service.db_pool().mysql_pool(),
            "Free",
        ).await {
            Ok(plan) => plan,
            Err(e) => {
                return Ok(Response::new(CreateTenantForOrgResponse {
                    success: false,
                    message: format!("查找Free套餐失败: {}", e),
                    tenant_id: 0,
                }));
            }
        };
        let plan_id = free_plan.id.unwrap_or(0);

        let now = Utc::now();
        let expire_time = now + Duration::days(365 * 10);

        // 创建租户
        let tenant_result = self.tenant_service.create_tenant(
            &req.org_name,
            &req.contact_name,
            if req.contact_mobile.is_empty() { None } else { Some(&req.contact_mobile) },
            plan_id,
            expire_time,
            100, // 默认账号数
            None,
            Some(req.owner_user_id),
        ).await;

        match tenant_result {
            Ok(tenant_id) => {
                // 将用户添加到租户（is_owner=1）
                if let Err(e) = self.user_tenant_service.add_user_to_tenant(
                    req.owner_user_id,
                    tenant_id,
                    true, // is_owner
                    Some(req.owner_user_id),
                ).await {
                    return Ok(Response::new(CreateTenantForOrgResponse {
                        success: false,
                        message: format!("创建租户用户关系失败: {}", e),
                        tenant_id: 0,
                    }));
                }

                Ok(Response::new(CreateTenantForOrgResponse {
                    success: true,
                    message: "租户创建成功".to_string(),
                    tenant_id,
                }))
            }
            Err(e) => {
                let error_msg = if let Some(identity_err) = e.downcast_ref::<IdentityError>() {
                    identity_err.to_string()
                } else {
                    e.to_string()
                };
                Ok(Response::new(CreateTenantForOrgResponse {
                    success: false,
                    message: error_msg,
                    tenant_id: 0,
                }))
            }
        }
    }

    /// 初始化组织角色（创建 owner/admin/member 三个角色）
    /// Role.code 使用标准的 owner/admin/member，通过 biz_id=org_id 区分
    async fn init_org_roles(
        &self,
        request: Request<InitOrgRolesRequest>,
    ) -> Result<Response<InitOrgRolesResponse>, Status> {
        let req = request.into_inner();

        let roles = vec![
            ("owner", "组织所有者"),
            ("admin", "组织管理员"),
            ("member", "组织成员"),
        ];

        let mut owner_role_id: i64 = 0;
        let mut admin_role_id: i64 = 0;
        let mut member_role_id: i64 = 0;

        for (code, name) in roles {
            match self.role_service.create_org_role(
                req.tenant_id,
                code,
                name,
                Some(req.org_id), // biz_id = org_id
                Some(req.created_by),
            ).await {
                Ok(role_id) => {
                    match code {
                        "owner" => owner_role_id = role_id,
                        "admin" => admin_role_id = role_id,
                        "member" => member_role_id = role_id,
                        _ => {}
                    }
                }
                Err(e) => {
                    let error_msg = if let Some(identity_err) = e.downcast_ref::<IdentityError>() {
                        identity_err.to_string()
                    } else {
                        e.to_string()
                    };
                    return Ok(Response::new(InitOrgRolesResponse {
                        success: false,
                        message: format!("创建角色 {} 失败: {}", code, error_msg),
                        owner_role_id: 0,
                        admin_role_id: 0,
                        member_role_id: 0,
                    }));
                }
            }
        }

        Ok(Response::new(InitOrgRolesResponse {
            success: true,
            message: "组织角色初始化成功".to_string(),
            owner_role_id,
            admin_role_id,
            member_role_id,
        }))
    }

    /// 分配角色给用户
    async fn assign_role(
        &self,
        request: Request<AssignRoleRequest>,
    ) -> Result<Response<AssignRoleResponse>, Status> {
        let req = request.into_inner();

        // 直接创建 UserRole 记录
        let mut user_role = UserRole::default();
        user_role.user_id = req.user_id;
        user_role.role_id = req.role_id;
        user_role.role_code = req.role_code;
        user_role.tenant_id = req.tenant_id;
        user_role.created_at = Some(Utc::now());

        match user_role.insert(self.role_service.db_pool().mysql_pool()).await {
            Ok(_) => Ok(Response::new(AssignRoleResponse {
                success: true,
                message: "角色分配成功".to_string(),
            })),
            Err(e) => Ok(Response::new(AssignRoleResponse {
                success: false,
                message: format!("角色分配失败: {}", e),
            })),
        }
    }
}

/// 将 User 实体转换为 proto UserInfo
fn convert_user_to_proto(user: &User) -> UserInfo {
    UserInfo {
        id: user.id.unwrap_or(0),
        username: user.username.clone().unwrap_or_default(),
        nick_name: user.nick_name.clone().unwrap_or_default(),
        avatar: user.avatar.clone().unwrap_or_default(),
        state: user.state.unwrap_or(0) as i32,
    }
}
