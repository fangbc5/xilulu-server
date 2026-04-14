// Identity 服务 gRPC 客户端

use anyhow::Result;
use fbc_starter::{get_load_balancer, LoadBalancer, ServiceEndpoint};
use tonic::Request;

// 生成的 proto 代码
pub mod identity_proto {
    tonic::include_proto!("identity");
}

use identity_proto::{
    identity_service_client::IdentityServiceClient, GetUserInfoRequest, RegisterUserRequest,
    RegisterUserResponse, SearchUserRpcRequest, SearchUserRpcResponse, UserInfoResponse,
    UserTenantsResponse, VerifyRequest, VerifyResponse,
};

/// Identity 服务客户端
pub struct IdentityClient;

impl IdentityClient {
    const SERVICE_NAME: &'static str = "ms-identity";

    /// 统一验证方法（支持用户名/邮箱/手机号 + 密码/验证码）
    pub async fn verify(
        username: Option<&str>,
        password: Option<&str>,
        mobile: Option<&str>,
        email: Option<&str>,
        region: Option<&str>,
    ) -> Result<VerifyResponse> {
        let endpoint = Self::get_endpoint().await?;
        let mut client = IdentityServiceClient::new(endpoint.endpoint.connect().await?);

        let request = Request::new(VerifyRequest {
            username: username.unwrap_or("").to_string(),
            password: password.unwrap_or("").to_string(),
            mobile: mobile.unwrap_or("").to_string(),
            email: email.unwrap_or("").to_string(),
            region: region.unwrap_or("").to_string(),
        });

        let response = client.verify(request).await?;
        Ok(response.into_inner())
    }

    /// 获取用户租户列表
    pub async fn get_user_tenants(user_id: i64) -> Result<UserTenantsResponse> {
        let endpoint = Self::get_endpoint().await?;
        let mut client = IdentityServiceClient::new(endpoint.endpoint.connect().await?);

        let request = Request::new(GetUserInfoRequest { user_id });

        let response = client.get_user_tenants(request).await?;
        Ok(response.into_inner())
    }

    /// 获取用户信息
    pub async fn get_user_info(user_id: i64) -> Result<UserInfoResponse> {
        let endpoint = Self::get_endpoint().await?;
        let mut client = IdentityServiceClient::new(endpoint.endpoint.connect().await?);

        let request = Request::new(GetUserInfoRequest { user_id });

        let response = client.get_user_info(request).await?;
        Ok(response.into_inner())
    }

    /// 注册用户
    ///
    /// # 参数
    /// - `username`: 用户名（可选，与手机号/邮箱二选一）
    /// - `password`: 密码（必填）
    /// - `mobile`: 手机号（可选，与用户名/邮箱二选一）
    /// - `email`: 邮箱（可选，与用户名/手机号二选一）
    /// - `nick_name`: 昵称（可选）
    ///
    /// # 返回
    /// - `Ok(RegisterUserResponse)`: 注册响应
    pub async fn register_user(
        username: Option<&str>,
        password: Option<&str>,
        mobile: Option<&str>,
        email: Option<&str>,
        nick_name: Option<&str>,
        avatar: Option<&str>,
        region: Option<&str>,
    ) -> Result<RegisterUserResponse> {
        let endpoint = Self::get_endpoint().await?;
        let mut client = IdentityServiceClient::new(endpoint.endpoint.connect().await?);

        let request = Request::new(RegisterUserRequest {
            username: username.unwrap_or("").to_string(),
            password: password.unwrap_or("").to_string(),
            mobile: mobile.unwrap_or("").to_string(),
            email: email.unwrap_or("").to_string(),
            nick_name: nick_name.unwrap_or("").to_string(),
            avatar: avatar.unwrap_or("").to_string(),
            region: region.unwrap_or("").to_string(),
        });

        let response = client.register_user(request).await?;
        Ok(response.into_inner())
    }

    pub async fn search_user(keyword: &str) -> Result<SearchUserRpcResponse> {
        let endpoint = Self::get_endpoint().await?;
        let mut client = IdentityServiceClient::new(endpoint.endpoint.connect().await?);

        let request = Request::new(SearchUserRpcRequest {
            keyword: keyword.to_string(),
        });

        let response = client.search_user(request).await?;
        Ok(response.into_inner())
    }

    /// 获取服务端点（使用负载均衡）
    async fn get_endpoint() -> Result<ServiceEndpoint> {
        let balancer = get_load_balancer(Self::SERVICE_NAME);
        balancer
            .next_endpoint()
            .ok_or_else(|| anyhow::anyhow!("未找到可用的服务实例: {}", Self::SERVICE_NAME))
    }
}
