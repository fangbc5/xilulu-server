use crate::balance::discovery::ServiceEndpoint;
use dashmap::DashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tonic::transport::Channel;

/// 负载均衡器 trait
///
/// 提供服务端点选择的抽象，支持不同的负载均衡策略
pub trait LoadBalancer: Send + Sync {
    /// 获取下一个可用的服务端点
    fn next_endpoint(&self) -> Option<ServiceEndpoint>;

    /// 获取负载均衡器所属的服务名称
    fn service_name(&self) -> &str;
}

/// 轮询负载均衡器
///
/// 结合 Nacos 服务发现和本地健康检查过滤不健康实例，
/// 如果全部不健康则 fallback 到全部实例（避免全部不可用）。
pub struct RoundRobinLoadBalancer {
    service_name: String,
    current_index: Arc<AtomicUsize>,
}

/// 全局负载均衡器实例池（每个服务名一个负载均衡器）
/// 这样可以确保轮询计数器在多次调用之间保持状态
static LOAD_BALANCERS: std::sync::LazyLock<DashMap<String, Arc<RoundRobinLoadBalancer>>> =
    std::sync::LazyLock::new(|| DashMap::new());

/// 获取或创建服务的全局负载均衡器
///
/// 使用全局实例池确保每个服务只有一个负载均衡器实例，
/// 这样轮询计数器可以在多次调用间保持状态
///
/// # 参数
///
/// - `service_name`: 服务名称
///
/// # 返回
///
/// 返回服务对应的负载均衡器实例
pub fn get_load_balancer(service_name: &str) -> Arc<RoundRobinLoadBalancer> {
    LOAD_BALANCERS
        .entry(service_name.to_string())
        .or_insert_with(|| Arc::new(RoundRobinLoadBalancer::new(service_name.to_string())))
        .clone()
}

impl RoundRobinLoadBalancer {
    /// 创建新的负载均衡器
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            current_index: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// 构建带负载均衡的 Channel
    ///
    /// 使用 next_endpoint() 选择端点，如果没有可用端点则返回 ServiceUnavailable 错误
    pub async fn build_channel(&self) -> Result<Channel, crate::error::AppError> {
        let endpoint = self
            .next_endpoint()
            .ok_or_else(|| {
                crate::error::AppError::ServiceUnavailable(format!(
                    "服务 {} 没有可用的实例",
                    self.service_name
                ))
            })?;

        endpoint.endpoint.connect().await.map_err(|e| {
            crate::error::AppError::ServiceUnavailable(format!(
                "连接服务 {} 失败: {}",
                self.service_name, e
            ))
        })
    }
}

impl LoadBalancer for RoundRobinLoadBalancer {
    /// 获取下一个服务端点（带轮询负载均衡 + 健康过滤）
    ///
    /// 优先选择健康或状态未知的实例；如果全部不健康则 fallback 到全部实例
    /// （避免因健康检查延迟导致全部实例不可用）
    fn next_endpoint(&self) -> Option<ServiceEndpoint> {
        let endpoints = crate::balance::discovery::get_service_endpoints(&self.service_name);

        if endpoints.is_empty() {
            tracing::warn!("服务 {} 没有可用的实例", self.service_name);
            return None;
        }

        // 过滤出健康/未知的实例
        let available: Vec<&ServiceEndpoint> = endpoints
            .iter()
            .filter(|ep| {
                crate::balance::health::is_available(&self.service_name, &ep.instance_id)
            })
            .collect();

        // 如果有健康实例则从中选，否则 fallback 到全部（容错）
        let pool = if available.is_empty() {
            tracing::warn!(
                "服务 {} 所有 {} 个实例均不健康，fallback 到全部实例",
                self.service_name,
                endpoints.len()
            );
            endpoints.iter().collect::<Vec<_>>()
        } else {
            available
        };

        let index = self.current_index.fetch_add(1, Ordering::Relaxed);
        let selected = pool.get(index % pool.len()).cloned().cloned();

        if let Some(ref endpoint) = selected {
            tracing::debug!(
                "负载均衡选择: 服务={}, 实例={}, 索引={}/{} (健康池/总: {}/{})",
                self.service_name,
                endpoint.instance_id,
                index % pool.len(),
                pool.len(),
                pool.len(),
                endpoints.len()
            );
        }

        selected
    }

    fn service_name(&self) -> &str {
        &self.service_name
    }
}
