# fbc-starter 优化计划

## 概览

基于 [架构分析](fbc_starter_analysis.md) 中识别的 9 个待优化项，分三个阶段实施。每项优化包含代码修改和对应测试覆盖。

---

## 阶段一：代码质量（优先实施）

### O-1 重构 `init_logging` 消除分支重复 `[HIGH]`
- [ ] 状态：待实施

**当前问题**: `lib.rs` 中 `init_logging` 有 4 个 `(json × file_format)` 组合分支，每段代码几乎一致。

**改动文件**: `src/lib.rs`

**方案**: 将 stdout/file layer 的创建抽取为辅助函数，用 `Box<dyn Layer>` 统一层类型，消除分支重复。

**测试**: 新增 `tests/logging_tests.rs`，验证不同 json/file 组合下日志系统正确初始化。

---

### O-2 合并重复错误类型 `[MEDIUM]`
- [ ] 状态：待实施

**当前问题**: `BizError`、`CommonError`、`CustomerError` 三个 variant 结构和处理方式完全一致。

**改动文件**: `src/error.rs`

**方案**:
- 新增 `ErrorCategory` 枚举 (`Biz`, `Common`, `Custom`)
- 合并为 `AppError::Categorized(i32, String, ErrorCategory)`
- 保留原有的 `biz_error()`、`common_error()`、`customer_error()` 工厂方法作为兼容 API
- `IntoResponse` 中统一处理

**测试**: 新增 `tests/error_tests.rs`，验证各类错误正确映射 HTTP 状态码和响应体。

---

### O-3 `R<T>` 空字段条件跳过 `[LOW]`
- [ ] 状态：待实施

**当前问题**: `path`、`version`、`base_version` 总是空字符串，序列化占空间。

**改动文件**: `src/base.rs`

**方案**: 添加 `#[serde(skip_serializing_if = "String::is_empty")]` 注解。

**测试**: 新增 `tests/base_tests.rs`，验证空字段不出现在 JSON 输出中，非空时正常序列化。

---

### O-4 `build_channel` 自定义错误 `[LOW]`
- [ ] 状态：待实施

**当前问题**: 无可用端点时通过构造无效地址 `http://[::1]:0` 来触发错误，语义不清。

**改动文件**: `src/balance/load_balancer.rs`, `src/error.rs`

**方案**:
- 在 `AppError` 中新增 `ServiceUnavailable(String)` variant
- `build_channel` 返回 `Result<Channel, AppError>`

**测试**: 在 `tests/balance_tests.rs` 中验证无可用端点时返回正确错误。

---

## 阶段二：架构增强

### O-5 `AppState` 扩展能力 `[HIGH]`
- [ ] 状态：待实施

**当前问题**: `AppState` 字段固定，用户无法添加自定义业务状态。

**改动文件**: `src/state.rs`

**方案**: 使用 `extensions: dashmap::DashMap<TypeId, Box<dyn Any + Send + Sync>>` 存储用户扩展数据，提供类型安全的 `set_extension<T>()` / `get_extension<T>()` API。

**测试**: 验证扩展的存取、类型安全、线程安全。

---

### O-6 Nacos 命名空间分离 `[MEDIUM]`
- [ ] 状态：待实施

**当前问题**: `NacosConfig` 只有一个 `namespace`，配置中心和服务注册共享命名空间。

**改动文件**: `src/config.rs`, `src/nacos/client.rs`, `src/nacos/service.rs`

**方案**:
- `NacosConfig` 增加 `config_namespace`、`naming_namespace` 字段（`namespace` 保留作为默认值兼容）
- `config_group`、`naming_group` 分别控制配置和服务的组
- `init_nacos` 创建客户端时分别使用对应命名空间

> [!WARNING]
> 此改动涉及配置 breaking change，需要提供向后兼容：当 `config_namespace` / `naming_namespace` 未设置时，回退到 `namespace` 字段。

**测试**: 验证命名空间分离、向后兼容逻辑。

---

### O-7 健康检查复用 HTTP 客户端 `[MEDIUM]`
- [ ] 状态：待实施

**当前问题**: `check_instance_health()` 每次调用都创建新的 `reqwest::Client`。

**改动文件**: `src/balance/health.rs`

**方案**: 在 `start_health_check()` 中创建一个 `reqwest::Client` 并在闭包中复用。

**测试**: 验证健康检查功能正常工作。

---

### O-8 负载均衡 trait 抽象 `[LOW]`
- [ ] 状态：待实施

**当前问题**: `RoundRobinLoadBalancer` 是唯一策略，无 trait 抽象。

**改动文件**: `src/balance/load_balancer.rs`, `src/balance/mod.rs`

**方案**:
- 提取 `LoadBalancer` trait (`fn next_endpoint(&self) -> Option<ServiceEndpoint>`)
- `RoundRobinLoadBalancer` 实现此 trait
- 全局池改为 `DashMap<String, Arc<dyn LoadBalancer>>`

**测试**: 验证 trait 抽象和轮询逻辑。

---

## 阶段三：功能增强

### O-9 优雅关闭时 Nacos 注销 `[MEDIUM]`
- [ ] 状态：待实施

**当前问题**: 服务关闭时未主动从 Nacos 注销，依赖心跳超时被动移除。

**改动文件**: `src/server.rs`, `src/nacos/service.rs`

**方案**: 在 `shutdown_signal` 中调用 `deregister_service()` 主动注销。

**测试**: 验证 shutdown 信号触发注销逻辑。

---

## 验证计划

### 构建验证
```bash
# 无 feature
cargo build
# 全 feature
cargo build --all-features
# 单独 feature 组合
cargo build --features "nacos,balance"
cargo build --features "kafka,producer,consumer"
cargo build --features "mysql,redis"
```

### 测试执行
```bash
cargo test --all-features
```

### 兼容性验证
- 确保 hula-server 中依赖 fbc-starter 的服务（ms-websocket 等）编译正常
