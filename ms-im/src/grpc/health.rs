use futures::stream::Empty;
use tonic::{Request, Response, Status};

// 引入生成的 gRPC 代码
pub mod health_proto {
    tonic::include_proto!("grpc.health.v1");
}

use health_proto::{
    HealthCheckRequest, HealthCheckResponse,
    health_server::{Health, HealthServer},
};

/// 健康检查服务实现
#[derive(Debug, Default)]
pub struct HealthService;

impl HealthService {
    /// 创建新的健康检查服务
    pub fn new() -> Self {
        Self
    }

    /// 转换为 gRPC 服务
    pub fn into_server(self) -> HealthServer<Self> {
        HealthServer::new(self)
    }
}

#[tonic::async_trait]
impl Health for HealthService {
    type WatchStream = Empty<Result<HealthCheckResponse, Status>>;

    /// 检查服务健康状态
    async fn check(
        &self,
        request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        let service_name = &request.get_ref().service;

        tracing::info!("收到健康检查请求, 服务名: {:?}", service_name);

        // 这里可以根据 service_name 检查不同服务的健康状态
        // 空字符串表示检查整个服务器
        let status = if service_name.is_empty() || service_name == "im-server" {
            // TODO: 可以在这里添加更复杂的健康检查逻辑
            // 例如：检查数据库连接、Redis 连接等
            health_proto::health_check_response::ServingStatus::Serving
        } else {
            // 未知的服务名
            health_proto::health_check_response::ServingStatus::ServiceUnknown
        };

        let response = HealthCheckResponse {
            status: status as i32,
        };

        Ok(Response::new(response))
    }

    /// 监听服务健康状态变化（流式响应）
    async fn watch(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        // Watch 方法暂不实现，返回未实现错误
        Err(Status::unimplemented("Watch 方法暂未实现"))
    }
}
