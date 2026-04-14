# fbc-starter 优化任务

## 阶段一：代码质量 ✅
- [x] O-1: 重构 `init_logging` 消除分支重复 — 4路分支→Option层，~200→115行
- [x] O-2: 合并重复错误类型 `BizError`/`CommonError`/`CustomerError` → `Categorized(i32, String, ErrorCategory)`
- [x] O-3: `R<T>` 空字段条件跳过序列化 — `path`/`version`/`base_version` 添加 `skip_serializing_if`
- [x] O-4: `build_channel` 使用 `AppError::ServiceUnavailable` 替代无效地址

## 阶段二：架构增强 ✅
- [x] O-5: `AppState` 增加类型安全扩展 — `set_extension<T>()`/`get_extension<T>()`
- [x] O-6: Nacos 命名空间分离 — 新增 `naming_namespace`/`config_namespace`/`naming_group`/`config_group`，向后兼容
- [x] O-7: 健康检查复用 HTTP 客户端 — `reqwest::Client` 在循环外创建
- [x] O-8: 负载均衡 `LoadBalancer` trait 抽象

## 阶段三：功能增强 ✅
- [x] O-9: 优雅关闭时 Nacos 注销 — `start_internal` 关闭后调用 `deregister_service`

## 下游服务同步 ✅
| 服务 | 变更 | 状态 |
|------|------|------|
| ms-notify | `BizError` → `biz_error()` | ✅ |
| ms-auth | 添加 `LoadBalancer` trait import | ✅ |
| ms-websocket | 添加 `LoadBalancer` trait import | ✅ |
| ms-organization | 添加 `LoadBalancer` trait import | ✅ |
| ms-im | 无需修改 | ✅ |
| ms-identity | 无需修改 | ✅ |
| ms-ai | 无需修改 | ✅ |

## 测试覆盖 ✅
- `tests/error_tests.rs` — 14 tests
- `tests/base_tests.rs` — 8 tests
- `tests/state_tests.rs` — 8 tests
- `tests/config_tests.rs` — 1 test
- `cargo build --workspace` — 全通过
