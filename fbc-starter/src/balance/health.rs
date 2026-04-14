use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::circuit_breaker::{get_existing_circuit_breaker, CircuitState};

/// 健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// 实例健康
    Healthy,
    /// 实例不健康（连续失败次数达到阈值）
    Unhealthy,
    /// 未知（尚未检查过）
    Unknown,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "Healthy"),
            HealthStatus::Unhealthy => write!(f, "Unhealthy"),
            HealthStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// 健康检查配置
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// 健康检查间隔
    pub interval: Duration,
    /// 连续失败次数达到此阈值后标记为 Unhealthy
    pub failure_threshold: u32,
    /// 连续成功次数达到此阈值后恢复为 Healthy
    pub recovery_threshold: u32,
    /// 连接探测超时
    pub probe_timeout: Duration,
    /// 是否与 CircuitBreaker 联动
    pub sync_circuit_breaker: bool,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(10),
            failure_threshold: 3,
            recovery_threshold: 1,
            probe_timeout: Duration::from_secs(3),
            sync_circuit_breaker: true,
        }
    }
}

/// 单个实例的健康信息
#[derive(Debug, Clone)]
struct InstanceHealth {
    status: HealthStatus,
    /// 连续失败计数
    failure_count: u32,
    /// 连续成功计数
    success_count: u32,
    /// 上次检查时间
    last_check: Instant,
}

impl Default for InstanceHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            failure_count: 0,
            success_count: 0,
            last_check: Instant::now(),
        }
    }
}

/// 全局健康状态存储
///
/// key: 服务名, value: { key: instance_id ("ip:port"), value: InstanceHealth }
static HEALTH_STATE: std::sync::LazyLock<DashMap<String, DashMap<String, InstanceHealth>>> =
    std::sync::LazyLock::new(|| DashMap::new());

// ─────────────────── 公开查询 API ───────────────────

/// 获取实例健康状态
pub fn get_health_status(service_name: &str, instance_id: &str) -> HealthStatus {
    // 优先检查 CircuitBreaker：如果熔断器打开则直接标记为 Unhealthy
    if let Some(cb) = get_existing_circuit_breaker(service_name) {
        if cb.state() == CircuitState::Open {
            return HealthStatus::Unhealthy;
        }
    }

    HEALTH_STATE
        .get(service_name)
        .and_then(|svc| svc.get(instance_id).map(|h| h.status))
        .unwrap_or(HealthStatus::Unknown)
}

/// 检查实例是否可用于请求（Healthy 或 Unknown 都放行）
pub fn is_available(service_name: &str, instance_id: &str) -> bool {
    get_health_status(service_name, instance_id) != HealthStatus::Unhealthy
}

/// 获取服务所有实例的健康信息快照（调试/监控用）
pub fn get_service_health_snapshot(
    service_name: &str,
) -> Vec<(String, HealthStatus)> {
    HEALTH_STATE
        .get(service_name)
        .map(|svc| {
            svc.iter()
                .map(|entry| (entry.key().clone(), entry.value().status))
                .collect()
        })
        .unwrap_or_default()
}

// ─────────────────── 状态更新 ───────────────────

/// 记录一次健康检查成功
pub fn record_healthy(
    service_name: &str,
    instance_id: &str,
    config: &HealthCheckConfig,
) {
    let svc = HEALTH_STATE
        .entry(service_name.to_string())
        .or_insert_with(DashMap::new);

    let mut health = svc
        .entry(instance_id.to_string())
        .or_insert_with(InstanceHealth::default);

    health.last_check = Instant::now();
    health.failure_count = 0;
    health.success_count += 1;

    if health.success_count >= config.recovery_threshold {
        if health.status != HealthStatus::Healthy {
            tracing::info!(
                service = service_name,
                instance = instance_id,
                "实例恢复健康 (连续成功 {} 次)",
                health.success_count
            );
        }
        health.status = HealthStatus::Healthy;
    }

    // 联动 CircuitBreaker
    if config.sync_circuit_breaker {
        if let Some(cb) = get_existing_circuit_breaker(service_name) {
            cb.record_success();
        }
    }
}

/// 记录一次健康检查失败
pub fn record_unhealthy(
    service_name: &str,
    instance_id: &str,
    config: &HealthCheckConfig,
) {
    let svc = HEALTH_STATE
        .entry(service_name.to_string())
        .or_insert_with(DashMap::new);

    let mut health = svc
        .entry(instance_id.to_string())
        .or_insert_with(InstanceHealth::default);

    health.last_check = Instant::now();
    health.success_count = 0;
    health.failure_count += 1;

    if health.failure_count >= config.failure_threshold {
        if health.status != HealthStatus::Unhealthy {
            tracing::warn!(
                service = service_name,
                instance = instance_id,
                "实例标记为不健康 (连续失败 {} 次，阈值 {})",
                health.failure_count,
                config.failure_threshold
            );
        }
        health.status = HealthStatus::Unhealthy;
    }

    // 联动 CircuitBreaker
    if config.sync_circuit_breaker {
        if let Some(cb) = get_existing_circuit_breaker(service_name) {
            cb.record_failure();
        }
    }
}

/// 移除服务实例的健康状态（实例下线时调用）
pub fn remove_instance(service_name: &str, instance_id: &str) {
    if let Some(svc) = HEALTH_STATE.get(service_name) {
        svc.remove(instance_id);
        tracing::debug!(
            service = service_name,
            instance = instance_id,
            "移除实例健康状态"
        );
    }
}

/// 手动标记实例为健康
pub fn mark_healthy(service_name: &str, instance_id: &str) {
    let svc = HEALTH_STATE
        .entry(service_name.to_string())
        .or_insert_with(DashMap::new);

    let mut health = svc
        .entry(instance_id.to_string())
        .or_insert_with(InstanceHealth::default);

    health.status = HealthStatus::Healthy;
    health.failure_count = 0;
    health.last_check = Instant::now();
}

// ─────────────────── 后台定时健康检查 ───────────────────

/// 启动后台健康检查任务
///
/// 定期对服务所有实例进行 gRPC 连接探测（TCP + TLS 握手），
/// 根据结果更新健康状态，并与 CircuitBreaker 联动。
///
/// # 使用示例
///
/// ```rust,ignore
/// use fbc_starter::balance::health::{start_health_checker, HealthCheckConfig};
/// use std::time::Duration;
///
/// let handle = start_health_checker(
///     "ms-identity".to_string(),
///     HealthCheckConfig {
///         interval: Duration::from_secs(15),
///         ..Default::default()
///     },
/// );
/// ```
pub fn start_health_checker(
    service_name: String,
    config: HealthCheckConfig,
) -> tokio::task::JoinHandle<()> {
    let config = Arc::new(config);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(config.interval);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        tracing::info!(
            service = %service_name,
            interval_ms = config.interval.as_millis(),
            "启动健康检查任务"
        );

        loop {
            interval.tick().await;

            // 如果熔断器在 Open 状态且未到冷却时间，跳过检查（避免无效探测）
            if let Some(cb) = get_existing_circuit_breaker(&service_name) {
                if cb.state() == CircuitState::Open {
                    tracing::debug!(
                        service = %service_name,
                        "熔断器处于 Open 状态，跳过健康检查"
                    );
                    continue;
                }
            }

            // 从 Nacos 获取当前服务实例
            let instances = match crate::nacos::get_service_instances(&service_name) {
                Some(instances) => instances,
                None => {
                    tracing::debug!(
                        service = %service_name,
                        "没有可用实例，跳过健康检查"
                    );
                    continue;
                }
            };

            // 并发探测所有实例
            let mut handles = Vec::new();
            for instance in instances {
                let svc = service_name.clone();
                let cfg = config.clone();
                let timeout = cfg.probe_timeout;

                handles.push(tokio::spawn(async move {
                    let instance_id = format!("{}:{}", instance.ip, instance.port);
                    let addr = format!("http://{}:{}", instance.ip, instance.port);

                    let is_ok = probe_grpc_endpoint(&addr, timeout).await;

                    if is_ok {
                        record_healthy(&svc, &instance_id, &cfg);
                    } else {
                        record_unhealthy(&svc, &instance_id, &cfg);
                    }
                }));
            }

            // 等待所有探测完成（不阻塞下一轮定时器）
            for handle in handles {
                let _ = handle.await;
            }
        }
    })
}

/// gRPC 端点探测
///
/// 尝试建立 TCP 连接（tonic Endpoint::connect），
/// 用于验证实例是否可达。不发送实际 RPC 请求。
async fn probe_grpc_endpoint(addr: &str, timeout: Duration) -> bool {
    use std::str::FromStr;
    use tonic::transport::Endpoint;

    let endpoint = match Endpoint::from_str(addr) {
        Ok(ep) => ep.connect_timeout(timeout).timeout(timeout),
        Err(e) => {
            tracing::debug!(addr = addr, error = %e, "无法解析端点地址");
            return false;
        }
    };

    match tokio::time::timeout(timeout, endpoint.connect()).await {
        Ok(Ok(_channel)) => true,
        Ok(Err(e)) => {
            tracing::debug!(addr = addr, error = %e, "gRPC 探测连接失败");
            false
        }
        Err(_) => {
            tracing::debug!(addr = addr, "gRPC 探测超时");
            false
        }
    }
}

// ─────────────────── 测试 ───────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> HealthCheckConfig {
        HealthCheckConfig {
            failure_threshold: 3,
            recovery_threshold: 2,
            sync_circuit_breaker: false, // 测试中不联动 CB
            ..Default::default()
        }
    }

    #[test]
    fn test_initial_status_is_unknown() {
        let status = get_health_status("test-health-svc", "127.0.0.1:8080");
        assert_eq!(status, HealthStatus::Unknown);
    }

    #[test]
    fn test_record_healthy_updates_status() {
        let cfg = test_config();
        record_healthy("test-healthy-svc", "10.0.0.1:9090", &cfg);
        record_healthy("test-healthy-svc", "10.0.0.1:9090", &cfg);
        assert_eq!(
            get_health_status("test-healthy-svc", "10.0.0.1:9090"),
            HealthStatus::Healthy
        );
    }

    #[test]
    fn test_record_unhealthy_with_threshold() {
        let cfg = test_config(); // failure_threshold = 3

        record_unhealthy("test-unhealthy-svc", "10.0.0.2:9090", &cfg);
        assert_ne!(
            get_health_status("test-unhealthy-svc", "10.0.0.2:9090"),
            HealthStatus::Unhealthy
        ); // 1 次不够

        record_unhealthy("test-unhealthy-svc", "10.0.0.2:9090", &cfg);
        assert_ne!(
            get_health_status("test-unhealthy-svc", "10.0.0.2:9090"),
            HealthStatus::Unhealthy
        ); // 2 次不够

        record_unhealthy("test-unhealthy-svc", "10.0.0.2:9090", &cfg);
        assert_eq!(
            get_health_status("test-unhealthy-svc", "10.0.0.2:9090"),
            HealthStatus::Unhealthy
        ); // 3 次达到阈值
    }

    #[test]
    fn test_recovery_after_unhealthy() {
        let cfg = HealthCheckConfig {
            failure_threshold: 1,
            recovery_threshold: 2,
            sync_circuit_breaker: false,
            ..Default::default()
        };

        // 标记为不健康
        record_unhealthy("test-recover-svc", "10.0.0.3:9090", &cfg);
        assert_eq!(
            get_health_status("test-recover-svc", "10.0.0.3:9090"),
            HealthStatus::Unhealthy
        );

        // 一次成功不够恢复
        record_healthy("test-recover-svc", "10.0.0.3:9090", &cfg);
        // failure_count 重置但 success_count=1 < recovery_threshold=2

        // 两次成功恢复
        record_healthy("test-recover-svc", "10.0.0.3:9090", &cfg);
        assert_eq!(
            get_health_status("test-recover-svc", "10.0.0.3:9090"),
            HealthStatus::Healthy
        );
    }

    #[test]
    fn test_success_resets_failure_count() {
        let cfg = test_config(); // failure_threshold = 3

        record_unhealthy("test-reset-svc", "10.0.0.4:9090", &cfg); // failure=1
        record_unhealthy("test-reset-svc", "10.0.0.4:9090", &cfg); // failure=2
        record_healthy("test-reset-svc", "10.0.0.4:9090", &cfg); // failure=0

        // 需要重新连续失败 3 次
        record_unhealthy("test-reset-svc", "10.0.0.4:9090", &cfg); // failure=1
        record_unhealthy("test-reset-svc", "10.0.0.4:9090", &cfg); // failure=2
        assert_ne!(
            get_health_status("test-reset-svc", "10.0.0.4:9090"),
            HealthStatus::Unhealthy
        );
    }

    #[test]
    fn test_is_available() {
        let cfg = HealthCheckConfig {
            failure_threshold: 1,
            sync_circuit_breaker: false,
            ..Default::default()
        };

        // Unknown → available
        assert!(is_available("test-avail-svc", "10.0.0.5:9090"));

        // Healthy → available
        mark_healthy("test-avail-svc", "10.0.0.5:9090");
        assert!(is_available("test-avail-svc", "10.0.0.5:9090"));

        // Unhealthy → not available
        record_unhealthy("test-avail-svc", "10.0.0.5:9090", &cfg);
        assert!(!is_available("test-avail-svc", "10.0.0.5:9090"));
    }

    #[test]
    fn test_remove_instance() {
        mark_healthy("test-remove-svc", "10.0.0.6:9090");
        assert_eq!(
            get_health_status("test-remove-svc", "10.0.0.6:9090"),
            HealthStatus::Healthy
        );

        remove_instance("test-remove-svc", "10.0.0.6:9090");
        assert_eq!(
            get_health_status("test-remove-svc", "10.0.0.6:9090"),
            HealthStatus::Unknown
        );
    }

    #[test]
    fn test_health_snapshot() {
        mark_healthy("test-snapshot-svc", "10.0.0.7:9090");
        mark_healthy("test-snapshot-svc", "10.0.0.8:9090");

        let snapshot = get_service_health_snapshot("test-snapshot-svc");
        assert_eq!(snapshot.len(), 2);
        assert!(snapshot.iter().all(|(_, s)| *s == HealthStatus::Healthy));
    }

    #[test]
    fn test_circuit_breaker_overrides_health() {
        use super::super::circuit_breaker::{get_circuit_breaker, CircuitBreakerConfig};

        // 创建一个熔断器并触发 Open
        let cb = get_circuit_breaker(
            "test-cb-override-svc",
            CircuitBreakerConfig {
                failure_threshold: 1,
                open_duration: Duration::from_secs(9999),
                ..Default::default()
            },
        );

        // 手动标记实例为健康
        mark_healthy("test-cb-override-svc", "10.0.0.9:9090");
        assert_eq!(
            get_health_status("test-cb-override-svc", "10.0.0.9:9090"),
            HealthStatus::Healthy
        );

        // 触发熔断
        cb.record_failure(); // → Open
        assert_eq!(
            get_health_status("test-cb-override-svc", "10.0.0.9:9090"),
            HealthStatus::Unhealthy // 被 CircuitBreaker 覆盖
        );

        // 手动重置熔断器
        cb.reset();
        assert_eq!(
            get_health_status("test-cb-override-svc", "10.0.0.9:9090"),
            HealthStatus::Healthy // 恢复
        );
    }
}
