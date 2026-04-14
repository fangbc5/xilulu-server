/// Identity 服务 gRPC 客户端
///
/// 用于 ms-im 调用 ms-identity 获取用户信息

use anyhow::Result;
use fbc_starter::{get_load_balancer, LoadBalancer, ServiceEndpoint};
use std::collections::HashMap;
use tonic::Request;

pub mod identity_proto {
    tonic::include_proto!("identity");
}

use identity_proto::{
    identity_service_client::IdentityServiceClient, BatchGetUserInfoRequest, SearchUserRpcRequest,
};

/// Identity 服务客户端
pub struct IdentityClient;

/// 用户简要信息（从 gRPC 响应转换）
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserBrief {
    pub id: i64,
    pub nick_name: String,
    pub avatar: String,
}

impl IdentityClient {
    const SERVICE_NAME: &'static str = "ms-identity";

    /// 批量获取用户信息，返回 uid → UserBrief 映射
    pub async fn batch_get_user_info(user_ids: Vec<i64>) -> Result<HashMap<i64, UserBrief>> {
        if user_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let endpoint = Self::get_endpoint().await?;
        let mut client = IdentityServiceClient::new(endpoint.endpoint.connect().await?);

        let request = Request::new(BatchGetUserInfoRequest { user_ids });
        let response = client.batch_get_user_info(request).await?;
        let inner = response.into_inner();

        if !inner.success {
            return Err(anyhow::anyhow!("批量获取用户信息失败: {}", inner.message));
        }

        let map = inner
            .users
            .into_iter()
            .map(|u| {
                (
                    u.id,
                    UserBrief {
                        id: u.id,
                        nick_name: u.nick_name,
                        avatar: u.avatar,
                    },
                )
            })
            .collect();

        Ok(map)
    }

    /// 搜索用户并转为简要信息
    pub async fn search_user(keyword: &str) -> Result<Option<UserBrief>> {
        let endpoint = Self::get_endpoint().await?;
        let mut client = IdentityServiceClient::new(endpoint.endpoint.connect().await?);

        let request = Request::new(SearchUserRpcRequest {
            keyword: keyword.to_string(),
        });

        let response = client.search_user(request).await?;
        let inner = response.into_inner();

        if !inner.success {
            return Ok(None);
        }

        if let Some(u) = inner.user {
            Ok(Some(UserBrief {
                id: u.id,
                nick_name: u.nick_name,
                avatar: u.avatar,
            }))
        } else {
            Ok(None)
        }
    }

    /// 获取服务端点（使用负载均衡）
    async fn get_endpoint() -> Result<ServiceEndpoint> {
        let balancer = get_load_balancer(Self::SERVICE_NAME);
        balancer
            .next_endpoint()
            .ok_or_else(|| anyhow::anyhow!("未找到可用的服务实例: {}", Self::SERVICE_NAME))
    }
}
