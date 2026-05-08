use anyhow::Result;
use tonic::transport::Channel;
use tonic::Request;

// 导入生成的 protobuf 代码
pub mod identity_proto {
    tonic::include_proto!("identity");
}

use identity_proto::{
    identity_service_client::IdentityServiceClient, GetUserInfoRequest, UserInfoResponse,
    CreateTenantForOrgRequest, CreateTenantForOrgResponse,
    InitOrgRolesRequest, InitOrgRolesResponse,
    AssignRoleRequest, AssignRoleResponse,
};

/// 创建到 ms-identity 的 gRPC 连接
async fn connect() -> Result<IdentityServiceClient<Channel>> {
    let lb = fbc_starter::get_load_balancer("ms-identity");
    use fbc_starter::LoadBalancer;
    let endpoint = lb
        .next_endpoint()
        .ok_or_else(|| anyhow::anyhow!("未找到 ms-identity 服务"))?;

    let channel = Channel::from_shared(format!("http://{}", endpoint.endpoint.uri()))
        .map_err(|e| anyhow::anyhow!("创建 Channel 失败: {}", e))?
        .connect()
        .await?;

    Ok(IdentityServiceClient::new(channel))
}

/// ms-identity gRPC 客户端
pub struct IdentityClient;

impl IdentityClient {
    /// 获取用户信息
    pub async fn get_user_info(user_id: i64) -> Result<UserInfoResponse> {
        let mut client = connect().await?;
        let request = Request::new(GetUserInfoRequest { user_id });
        let response = client.get_user_info(request).await?;
        Ok(response.into_inner())
    }

    /// 为组织创建租户（顶级组织时调用）
    /// 创建 Tenant + TenantUserRel(is_owner=1)
    pub async fn create_tenant_for_org(
        org_name: String,
        owner_user_id: i64,
        contact_name: String,
        contact_mobile: Option<String>,
    ) -> Result<CreateTenantForOrgResponse> {
        let mut client = connect().await?;
        let request = Request::new(CreateTenantForOrgRequest {
            org_name,
            owner_user_id,
            contact_name,
            contact_mobile: contact_mobile.unwrap_or_default(),
        });
        let response = client.create_tenant_for_organization(request).await?;
        Ok(response.into_inner())
    }

    /// 初始化组织角色（创建 owner/admin/member 三个角色）
    pub async fn init_org_roles(
        tenant_id: i64,
        org_id: i64,
        created_by: i64,
    ) -> Result<InitOrgRolesResponse> {
        let mut client = connect().await?;
        let request = Request::new(InitOrgRolesRequest {
            tenant_id,
            org_id,
            created_by,
        });
        let response = client.init_org_roles(request).await?;
        Ok(response.into_inner())
    }

    /// 分配角色给用户
    pub async fn assign_role(
        user_id: i64,
        role_id: i64,
        role_code: String,
        tenant_id: i64,
    ) -> Result<AssignRoleResponse> {
        let mut client = connect().await?;
        let request = Request::new(AssignRoleRequest {
            user_id,
            role_id,
            role_code,
            tenant_id,
        });
        let response = client.assign_role(request).await?;
        Ok(response.into_inner())
    }
}