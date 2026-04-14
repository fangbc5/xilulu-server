use crate::balance::load_balancer::RoundRobinLoadBalancer;
use tonic::transport::Channel;

/// gRPC 客户端构建器（简化版）
///
/// 使用 Nacos 服务发现 + 负载均衡（自动过滤不健康实例）。
/// 如需超时/重试/熔断能力，请使用 `ResilientGrpcClient`。
pub struct GrpcClientBuilder {
    service_name: String,
}

impl GrpcClientBuilder {
    pub fn new(service_name: String) -> Self {
        Self { service_name }
    }

    /// 构建带负载均衡的 Channel
    pub async fn build(&self) -> Result<Channel, crate::error::AppError> {
        let balancer = RoundRobinLoadBalancer::new(self.service_name.clone());
        balancer.build_channel().await
    }
}

/// 创建带负载均衡的 gRPC Channel（便捷函数）
///
/// 使用 Nacos 服务发现 + 轮询负载均衡 + 健康实例过滤。
/// 每次调用会创建新连接；如需连接复用和容错能力，请使用 `ResilientGrpcClient`。
pub async fn create_grpc_channel(service_name: &str) -> Result<Channel, crate::error::AppError> {
    GrpcClientBuilder::new(service_name.to_string())
        .build()
        .await
}

// 为了保持 API 兼容性，保留旧的函数名
pub use create_grpc_channel as create_grpc_client;

