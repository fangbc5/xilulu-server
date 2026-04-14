use std::future::Future;
use std::time::Duration;
use tonic::transport::Channel;
use dashmap::DashMap;

use crate::balance::circuit_breaker::{
    get_circuit_breaker, get_instance_circuit_breaker, CircuitBreakerConfig,
};
use crate::balance::load_balancer::{get_load_balancer, LoadBalancer};
use crate::error::AppError;

// ─────────────────── 全局 Channel 缓存池 ───────────────────

/// 全局 Channel 缓存池
///
/// key: instance_id ("ip:port"), value: 已建立的 Channel
/// tonic::Channel 内部就是一个连接池，clone 是轻量的 Arc clone。
static CHANNEL_POOL: std::sync::LazyLock<DashMap<String, Channel>> =
    std::sync::LazyLock::new(|| DashMap::new());

/// 获取或建立到指定端点的 Channel（连接复用）
async fn get_or_connect(
    endpoint: &crate::balance::discovery::ServiceEndpoint,
    connect_timeout: Duration,
) -> Result<Channel, AppError> {
    // 先从缓存查找
    if let Some(channel) = CHANNEL_POOL.get(&endpoint.instance_id) {
        return Ok(channel.clone());
    }

    // 缓存未命中，建新连接
    let channel = tokio::time::timeout(
        connect_timeout,
        endpoint.endpoint.connect(),
    )
    .await
    .map_err(|_| {
        AppError::ServiceUnavailable(format!(
            "连接 {} 超时 ({}ms)",
            endpoint.instance_id,
            connect_timeout.as_millis()
        ))
    })?
    .map_err(|e| {
        AppError::ServiceUnavailable(format!(
            "连接 {} 失败: {}",
            endpoint.instance_id, e
        ))
    })?;

    // 缓存连接
    CHANNEL_POOL.insert(endpoint.instance_id.clone(), channel.clone());
    Ok(channel)
}

/// 移除缓存中的连接（连接失效时调用）
fn evict_channel(instance_id: &str) {
    CHANNEL_POOL.remove(instance_id);
}

// ─────────────────── 重试配置 ───────────────────

/// 重试策略配置
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// 最大重试次数（不含首次调用）
    pub max_retries: u32,
    /// 初始重试间隔
    pub base_delay: Duration,
    /// 最大重试间隔（退避上限）
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 2,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(5),
        }
    }
}

// ─────────────────── 容错客户端 ───────────────────

/// 容错 gRPC 客户端
///
/// 封装了超时、重试（指数退避）、连接池化和实例级熔断功能。
/// 通过 Builder 模式配置，对现有 `get_load_balancer` / `create_grpc_channel` 无任何侵入。
///
/// # 特性
///
/// - **连接池化**：自动缓存已建立的 Channel，避免频繁建连接
/// - **实例级熔断**：每个服务实例独立的熔断器，单实例故障不影响其他实例
/// - **故障转移**：每次重试自动选择不同端点
/// - **指数退避**：重试间隔逐步递增，避免打暴恢复中的服务
///
/// # 使用示例
///
/// ```rust,ignore
/// use fbc_starter::balance::resilient_client::ResilientGrpcClient;
///
/// let response = ResilientGrpcClient::for_service("ms-identity")
///     .call(|channel| async move {
///         let mut client = IdentityServiceClient::new(channel);
///         client.get_user_info(Request::new(req)).await
///     })
///     .await?;
/// ```
pub struct ResilientGrpcClient {
    service_name: String,
    timeout: Duration,
    retry: RetryConfig,
    cb_config: CircuitBreakerConfig,
    /// 是否使用实例级熔断（默认 true）
    instance_level_cb: bool,
}

impl ResilientGrpcClient {
    /// 创建面向指定服务的容错客户端（使用默认配置）
    ///
    /// 默认：超时 5s、重试 2 次、熔断阈值 5、实例级熔断
    pub fn for_service(service_name: &str) -> Self {
        Self {
            service_name: service_name.to_string(),
            timeout: Duration::from_secs(5),
            retry: RetryConfig::default(),
            cb_config: CircuitBreakerConfig::default(),
            instance_level_cb: true,
        }
    }

    /// 设置超时时间
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// 设置重试策略
    pub fn with_retry(mut self, retry: RetryConfig) -> Self {
        self.retry = retry;
        self
    }

    /// 设置最大重试次数（快捷方法）
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.retry.max_retries = max_retries;
        self
    }

    /// 禁用重试
    pub fn no_retry(mut self) -> Self {
        self.retry.max_retries = 0;
        self
    }

    /// 设置熔断器配置
    pub fn with_circuit_breaker(mut self, config: CircuitBreakerConfig) -> Self {
        self.cb_config = config;
        self
    }

    /// 禁用熔断器（设置极高阈值）
    pub fn no_circuit_breaker(mut self) -> Self {
        self.cb_config.failure_threshold = u32::MAX;
        self
    }

    /// 使用服务级熔断（所有实例共享一个熔断器）
    pub fn service_level_cb(mut self) -> Self {
        self.instance_level_cb = false;
        self
    }

    /// 执行 gRPC 调用（带连接池 + 超时 + 重试 + 实例级熔断）
    ///
    /// 闭包接收 `Channel`，返回 gRPC 调用结果。
    /// 每次重试会重新选择端点（故障转移），连接从缓存池获取。
    ///
    /// # 参数
    /// - `f`: 接收 `Channel` 并返回 gRPC 调用 Future 的闭包
    ///
    /// # 错误
    /// - `AppError::ServiceUnavailable` — 熔断器打开 或 无可用实例
    /// - `AppError::Internal` — 连接/调用失败（重试耗尽后）
    pub async fn call<F, Fut, R>(&self, f: F) -> Result<R, AppError>
    where
        F: Fn(Channel) -> Fut + Send + Sync,
        Fut: Future<Output = Result<R, tonic::Status>> + Send,
        R: Send,
    {
        // 服务级熔断检查（快速拒绝）
        if !self.instance_level_cb {
            let cb = get_circuit_breaker(&self.service_name, self.cb_config.clone());
            if !cb.allow_request() {
                tracing::warn!(
                    service = %self.service_name,
                    state = %cb.state(),
                    "熔断器拒绝请求"
                );
                return Err(AppError::ServiceUnavailable(format!(
                    "服务 {} 熔断器已打开，请稍后再试",
                    self.service_name
                )));
            }
        }

        let balancer = get_load_balancer(&self.service_name);
        let total_attempts = self.retry.max_retries + 1;
        let mut last_error: Option<AppError> = None;

        for attempt in 0..total_attempts {
            if attempt > 0 {
                // 指数退避
                let delay = self.calculate_backoff(attempt);
                tracing::debug!(
                    service = %self.service_name,
                    attempt = attempt + 1,
                    total = total_attempts,
                    delay_ms = delay.as_millis(),
                    "重试 gRPC 调用"
                );
                tokio::time::sleep(delay).await;
            }

            // 每次选择新的端点（故障转移）
            let endpoint = match balancer.next_endpoint() {
                Some(ep) => ep,
                None => {
                    let err = AppError::ServiceUnavailable(format!(
                        "服务 {} 没有可用的实例",
                        self.service_name
                    ));
                    return Err(err);
                }
            };

            // 实例级熔断检查
            if self.instance_level_cb {
                let inst_cb = get_instance_circuit_breaker(
                    &self.service_name,
                    &endpoint.instance_id,
                    self.cb_config.clone(),
                );
                if !inst_cb.allow_request() {
                    tracing::debug!(
                        service = %self.service_name,
                        instance = %endpoint.instance_id,
                        "实例熔断器打开，跳过此实例"
                    );
                    // 不消耗重试次数，继续选下一个端点
                    continue;
                }
            }

            // 从缓存池获取或建立连接
            let channel = match get_or_connect(&endpoint, self.timeout).await {
                Ok(ch) => ch,
                Err(e) => {
                    tracing::warn!(
                        service = %self.service_name,
                        instance = %endpoint.instance_id,
                        error = %e,
                        "连接失败"
                    );
                    // 连接失败：驱逐缓存、记录实例熔断
                    evict_channel(&endpoint.instance_id);
                    self.record_instance_failure(&endpoint.instance_id);
                    last_error = Some(e);
                    continue;
                }
            };

            // 带超时的 gRPC 调用
            let result = tokio::time::timeout(self.timeout, f(channel)).await;

            match result {
                Ok(Ok(response)) => {
                    // 成功
                    self.record_instance_success(&endpoint.instance_id);
                    if attempt > 0 {
                        tracing::info!(
                            service = %self.service_name,
                            instance = %endpoint.instance_id,
                            attempt = attempt + 1,
                            "gRPC 调用在第 {} 次尝试后成功",
                            attempt + 1
                        );
                    }
                    return Ok(response);
                }
                Ok(Err(status)) => {
                    // 调用失败（非超时）
                    tracing::warn!(
                        service = %self.service_name,
                        instance = %endpoint.instance_id,
                        attempt = attempt + 1,
                        error = %status,
                        "gRPC 调用失败"
                    );
                    // 传输层错误驱逐连接缓存
                    if is_transport_error(&status) {
                        evict_channel(&endpoint.instance_id);
                    }
                    self.record_instance_failure(&endpoint.instance_id);
                    last_error = Some(AppError::Internal(anyhow::anyhow!(
                        "gRPC 调用 {} ({}) 失败: {}",
                        self.service_name,
                        endpoint.instance_id,
                        status
                    )));
                }
                Err(_) => {
                    // 超时
                    tracing::warn!(
                        service = %self.service_name,
                        instance = %endpoint.instance_id,
                        attempt = attempt + 1,
                        timeout_ms = self.timeout.as_millis(),
                        "gRPC 调用超时"
                    );
                    self.record_instance_failure(&endpoint.instance_id);
                    last_error = Some(AppError::ServiceUnavailable(format!(
                        "gRPC 调用 {} ({}) 超时 ({}ms)",
                        self.service_name,
                        endpoint.instance_id,
                        self.timeout.as_millis()
                    )));
                }
            }
        }

        // 所有重试耗尽
        tracing::error!(
            service = %self.service_name,
            attempts = total_attempts,
            "gRPC 调用在 {} 次尝试后全部失败",
            total_attempts
        );

        Err(last_error.unwrap_or_else(|| {
            AppError::ServiceUnavailable(format!(
                "gRPC 调用 {} 失败（重试 {} 次后）",
                self.service_name, self.retry.max_retries
            ))
        }))
    }

    // ---------- 内部方法 ----------

    /// 记录实例成功（更新对应级别的熔断器）
    fn record_instance_success(&self, instance_id: &str) {
        if self.instance_level_cb {
            let cb = get_instance_circuit_breaker(
                &self.service_name,
                instance_id,
                self.cb_config.clone(),
            );
            cb.record_success();
        } else {
            let cb = get_circuit_breaker(&self.service_name, self.cb_config.clone());
            cb.record_success();
        }
    }

    /// 记录实例失败（更新对应级别的熔断器）
    fn record_instance_failure(&self, instance_id: &str) {
        if self.instance_level_cb {
            let cb = get_instance_circuit_breaker(
                &self.service_name,
                instance_id,
                self.cb_config.clone(),
            );
            cb.record_failure();
        } else {
            let cb = get_circuit_breaker(&self.service_name, self.cb_config.clone());
            cb.record_failure();
        }
    }

    /// 计算指数退避延迟
    fn calculate_backoff(&self, attempt: u32) -> Duration {
        let delay = self
            .retry
            .base_delay
            .saturating_mul(2u32.saturating_pow(attempt.saturating_sub(1)));
        std::cmp::min(delay, self.retry.max_delay)
    }
}

/// 判断是否是传输层错误（需要驱逐连接缓存）
fn is_transport_error(status: &tonic::Status) -> bool {
    matches!(
        status.code(),
        tonic::Code::Unavailable
            | tonic::Code::Internal
            | tonic::Code::Unknown
    )
}

/// 便捷函数：创建容错 gRPC 客户端并执行调用
///
/// 使用默认配置：超时 5s、重试 2 次、实例级熔断、连接池化。
///
/// ```rust,ignore
/// let result = grpc_call("ms-identity", |ch| async move {
///     IdentityServiceClient::new(ch).get_user_info(req).await
/// }).await?;
/// ```
pub async fn grpc_call<F, Fut, R>(service_name: &str, f: F) -> Result<R, AppError>
where
    F: Fn(Channel) -> Fut + Send + Sync,
    Fut: Future<Output = Result<R, tonic::Status>> + Send,
    R: Send,
{
    ResilientGrpcClient::for_service(service_name)
        .call(f)
        .await
}
