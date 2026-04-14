use anyhow::Result;
use tonic::transport::Channel;
use tonic::Request;

// 导入生成的 protobuf 代码
pub mod identity_proto {
    tonic::include_proto!("identity");
}

use identity_proto::{
    identity_service_client::IdentityServiceClient, GetUserInfoRequest, UserInfoResponse,
};

/// ms-identity gRPC 客户端
pub struct IdentityClient;

impl IdentityClient {
    /// 获取用户信息
    pub async fn get_user_info(user_id: i64) -> Result<UserInfoResponse> {
        let lb = fbc_starter::get_load_balancer("ms-identity");
        use fbc_starter::LoadBalancer;
        let endpoint = lb
            .next_endpoint()
            .ok_or_else(|| anyhow::anyhow!("未找到 ms-identity 服务"))?;

        let channel = Channel::from_shared(format!("http://{}", endpoint.endpoint.uri()))
            .map_err(|e| anyhow::anyhow!("创建 Channel 失败: {}", e))?
            .connect()
            .await?;

        let mut client = IdentityServiceClient::new(channel);

        let request = Request::new(GetUserInfoRequest { user_id });
        let response = client.get_user_info(request).await?;

        Ok(response.into_inner())
    }
}
