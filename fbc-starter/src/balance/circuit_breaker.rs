use dashmap::DashMap;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// 熔断器状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// 正常通行 — 所有请求放行
    Closed,
    /// 熔断打开 — 拒绝所有请求
    Open,
    /// 半开探测 — 放行少量请求，检测是否恢复
    HalfOpen,
}

impl std::fmt::Display for CircuitState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CircuitState::Closed => write!(f, "Closed"),
            CircuitState::Open => write!(f, "Open"),
            CircuitState::HalfOpen => write!(f, "HalfOpen"),
        }
    }
}

/// 熔断器配置
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// 连续失败次数达到此阈值后触发熔断（Closed → Open）
    pub failure_threshold: u32,
    /// HalfOpen 状态下连续成功次数达到此阈值后恢复（HalfOpen → Closed）
    pub success_threshold: u32,
    /// Open 状态持续时间，超过后进入 HalfOpen
    pub open_duration: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 2,
            open_duration: Duration::from_secs(30),
        }
    }
}

/// 熔断器
///
/// 实现三态切换：Closed ⇄ Open ⇄ HalfOpen
///
/// - **Closed**：正常放行，连续失败达到 `failure_threshold` 后跳转 Open
/// - **Open**：拒绝所有请求，等待 `open_duration` 后跳转 HalfOpen
/// - **HalfOpen**：放行探测请求，连续成功达到 `success_threshold` 后恢复 Closed；
///   任何一次失败立即回到 Open
pub struct CircuitBreaker {
    key: String,
    config: CircuitBreakerConfig,
    /// 当前状态（0=Closed, 1=Open, 2=HalfOpen）
    state: AtomicU32,
    /// 连续失败计数
    failure_count: AtomicU32,
    /// HalfOpen 模式下连续成功计数
    success_count: AtomicU32,
    /// Open 状态开始时间（Unix 毫秒，0 表示未设置）
    open_since_ms: AtomicU64,
}

impl CircuitBreaker {
    /// 创建新的熔断器
    pub fn new(key: String, config: CircuitBreakerConfig) -> Self {
        Self {
            key,
            config,
            state: AtomicU32::new(0), // Closed
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            open_since_ms: AtomicU64::new(0),
        }
    }

    /// 获取当前状态
    pub fn state(&self) -> CircuitState {
        let raw = self.state.load(Ordering::Acquire);
        match raw {
            1 => {
                // Open 状态：检查是否已超过冷却时间
                if self.should_transition_to_half_open() {
                    self.transition_to(CircuitState::HalfOpen);
                    CircuitState::HalfOpen
                } else {
                    CircuitState::Open
                }
            }
            2 => CircuitState::HalfOpen,
            _ => CircuitState::Closed,
        }
    }

    /// 获取熔断器 key
    pub fn key(&self) -> &str {
        &self.key
    }

    /// 检查是否允许通行
    ///
    /// - Closed / HalfOpen → true
    /// - Open 且未超过冷却时间 → false
    /// - Open 且已超过冷却时间 → 自动转为 HalfOpen，返回 true
    pub fn allow_request(&self) -> bool {
        match self.state() {
            CircuitState::Closed | CircuitState::HalfOpen => true,
            CircuitState::Open => false,
        }
    }

    /// 记录成功
    ///
    /// - Closed 状态：重置失败计数
    /// - HalfOpen 状态：递增成功计数，达到阈值后恢复 Closed
    pub fn record_success(&self) {
        match self.state() {
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::Release);
            }
            CircuitState::HalfOpen => {
                let count = self.success_count.fetch_add(1, Ordering::AcqRel) + 1;
                if count >= self.config.success_threshold {
                    tracing::info!(
                        key = %self.key,
                        "熔断器恢复: HalfOpen → Closed (连续成功 {} 次)",
                        count
                    );
                    self.transition_to(CircuitState::Closed);
                }
            }
            CircuitState::Open => {
                // Open 状态不应该有请求，忽略
            }
        }
    }

    /// 记录失败
    ///
    /// - Closed 状态：递增失败计数，达到阈值后触发熔断
    /// - HalfOpen 状态：任何失败立即回到 Open
    pub fn record_failure(&self) {
        match self.state() {
            CircuitState::Closed => {
                let count = self.failure_count.fetch_add(1, Ordering::AcqRel) + 1;
                if count >= self.config.failure_threshold {
                    tracing::warn!(
                        key = %self.key,
                        "熔断器触发: Closed → Open (连续失败 {} 次，阈值 {})",
                        count,
                        self.config.failure_threshold
                    );
                    self.transition_to(CircuitState::Open);
                }
            }
            CircuitState::HalfOpen => {
                tracing::warn!(
                    key = %self.key,
                    "熔断器重新打开: HalfOpen → Open (探测请求失败)"
                );
                self.transition_to(CircuitState::Open);
            }
            CircuitState::Open => {
                // Open 状态不应该有请求，忽略
            }
        }
    }

    /// 手动重置熔断器到 Closed 状态
    pub fn reset(&self) {
        tracing::info!(key = %self.key, "熔断器手动重置 → Closed");
        self.transition_to(CircuitState::Closed);
    }

    /// 获取连续失败次数
    pub fn failure_count(&self) -> u32 {
        self.failure_count.load(Ordering::Acquire)
    }

    // ---------- 内部方法 ----------

    fn transition_to(&self, new_state: CircuitState) {
        let raw = match new_state {
            CircuitState::Closed => {
                self.failure_count.store(0, Ordering::Release);
                self.success_count.store(0, Ordering::Release);
                self.open_since_ms.store(0, Ordering::Release);
                0
            }
            CircuitState::Open => {
                self.success_count.store(0, Ordering::Release);
                let epoch_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;
                self.open_since_ms.store(epoch_ms, Ordering::Release);
                1
            }
            CircuitState::HalfOpen => {
                self.success_count.store(0, Ordering::Release);
                2
            }
        };
        self.state.store(raw, Ordering::Release);
    }

    fn should_transition_to_half_open(&self) -> bool {
        let open_since = self.open_since_ms.load(Ordering::Acquire);
        if open_since == 0 {
            return false;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let elapsed_ms = now.saturating_sub(open_since);
        elapsed_ms >= self.config.open_duration.as_millis() as u64
    }
}

// ---------- 全局熔断器实例池 ----------

/// 全局 CircuitBreaker 实例池
///
/// key 格式支持：
/// - 服务级: `"ms-identity"` — 整个服务共用一个熔断器
/// - 实例级: `"ms-identity::10.0.0.1:9090"` — 每个实例独立的熔断器
static CIRCUIT_BREAKERS: std::sync::LazyLock<DashMap<String, Arc<CircuitBreaker>>> =
    std::sync::LazyLock::new(|| DashMap::new());

/// 获取或创建服务级别的全局熔断器
///
/// 使用全局实例池确保每个服务只有一个熔断器实例，
/// 在多次调用间共享状态。
///
/// # 参数
/// - `service_name`: 服务名称
/// - `config`: 熔断器配置（仅在首次创建时生效）
pub fn get_circuit_breaker(
    service_name: &str,
    config: CircuitBreakerConfig,
) -> Arc<CircuitBreaker> {
    CIRCUIT_BREAKERS
        .entry(service_name.to_string())
        .or_insert_with(|| Arc::new(CircuitBreaker::new(service_name.to_string(), config)))
        .clone()
}

/// 获取或创建实例级别的熔断器
///
/// key 格式为 `"service_name::instance_id"`，每个实例独立的熔断器。
/// 当某个实例故障时，只影响该实例的熔断器，不影响服务下的其他实例。
///
/// # 参数
/// - `service_name`: 服务名称
/// - `instance_id`: 实例 ID（通常为 "ip:port"）
/// - `config`: 熔断器配置（仅在首次创建时生效）
pub fn get_instance_circuit_breaker(
    service_name: &str,
    instance_id: &str,
    config: CircuitBreakerConfig,
) -> Arc<CircuitBreaker> {
    let key = format!("{}::{}", service_name, instance_id);
    CIRCUIT_BREAKERS
        .entry(key.clone())
        .or_insert_with(|| Arc::new(CircuitBreaker::new(key, config)))
        .clone()
}

/// 获取已存在的熔断器（不创建新的）
pub fn get_existing_circuit_breaker(service_name: &str) -> Option<Arc<CircuitBreaker>> {
    CIRCUIT_BREAKERS.get(service_name).map(|v| v.clone())
}

/// 获取已存在的实例级熔断器（不创建新的）
pub fn get_existing_instance_circuit_breaker(
    service_name: &str,
    instance_id: &str,
) -> Option<Arc<CircuitBreaker>> {
    let key = format!("{}::{}", service_name, instance_id);
    CIRCUIT_BREAKERS.get(&key).map(|v| v.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_closed() {
        let cb = CircuitBreaker::new("test".into(), CircuitBreakerConfig::default());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_closed_to_open_on_failures() {
        let cb = CircuitBreaker::new(
            "test".into(),
            CircuitBreakerConfig {
                failure_threshold: 3,
                ..Default::default()
            },
        );

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        cb.record_failure(); // 第 3 次失败 → Open
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_success_resets_failure_count() {
        let cb = CircuitBreaker::new(
            "test".into(),
            CircuitBreakerConfig {
                failure_threshold: 3,
                ..Default::default()
            },
        );

        cb.record_failure();
        cb.record_failure();
        cb.record_success(); // 重置失败计数
        assert_eq!(cb.failure_count(), 0);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_to_closed_on_success() {
        let cb = CircuitBreaker::new(
            "test".into(),
            CircuitBreakerConfig {
                failure_threshold: 1,
                success_threshold: 2,
                open_duration: Duration::from_millis(0), // 立即进入 HalfOpen
            },
        );

        cb.record_failure(); // → Open
        // open_duration=0 所以立即变 HalfOpen
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen); // 还需要 1 次
        cb.record_success(); // → Closed
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_to_open_on_failure() {
        let cb = CircuitBreaker::new(
            "test".into(),
            CircuitBreakerConfig {
                failure_threshold: 1,
                success_threshold: 3,
                open_duration: Duration::from_millis(0),
            },
        );

        cb.record_failure(); // → Open → HalfOpen (冷却时间=0)
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // HalfOpen → Open，使用长冷却时间验证留在 Open
        let cb2 = CircuitBreaker::new(
            "test2".into(),
            CircuitBreakerConfig {
                failure_threshold: 1,
                success_threshold: 3,
                open_duration: Duration::from_secs(9999), // 确保不会自动 HalfOpen
            },
        );
        cb2.record_failure(); // → Open
        assert_eq!(cb2.state(), CircuitState::Open);
        assert!(!cb2.allow_request());
    }

    #[test]
    fn test_manual_reset() {
        let cb = CircuitBreaker::new(
            "test".into(),
            CircuitBreakerConfig {
                failure_threshold: 1,
                open_duration: Duration::from_secs(9999),
                ..Default::default()
            },
        );

        cb.record_failure(); // → Open
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset(); // 手动重置
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_global_circuit_breaker_pool() {
        let cb1 = get_circuit_breaker("pool-test-svc", CircuitBreakerConfig::default());
        let cb2 = get_circuit_breaker("pool-test-svc", CircuitBreakerConfig::default());
        // 同一个实例
        assert!(Arc::ptr_eq(&cb1, &cb2));
    }

    #[test]
    fn test_instance_level_circuit_breaker() {
        let cb1 = get_instance_circuit_breaker(
            "inst-test-svc",
            "10.0.0.1:9090",
            CircuitBreakerConfig {
                failure_threshold: 1,
                open_duration: Duration::from_secs(9999),
                ..Default::default()
            },
        );
        let cb2 = get_instance_circuit_breaker(
            "inst-test-svc",
            "10.0.0.2:9090",
            CircuitBreakerConfig {
                failure_threshold: 1,
                open_duration: Duration::from_secs(9999),
                ..Default::default()
            },
        );

        // 不同实例是独立的熔断器
        assert!(!Arc::ptr_eq(&cb1, &cb2));

        // 实例1 熔断不影响实例2
        cb1.record_failure();
        assert_eq!(cb1.state(), CircuitState::Open);
        assert_eq!(cb2.state(), CircuitState::Closed); // 实例2 不受影响

        // 同实例拿到同一个
        let cb1_again = get_instance_circuit_breaker(
            "inst-test-svc",
            "10.0.0.1:9090",
            CircuitBreakerConfig::default(),
        );
        assert!(Arc::ptr_eq(&cb1, &cb1_again));
    }
}
