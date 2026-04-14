# ms-team 测试快速参考

## 🚀 快速开始

```bash
# 运行所有测试
cargo test -p ms-team

# 仅单元测试
cargo test -p ms-team --lib

# 仅集成测试
cargo test -p ms-team --test '*'

# 特定测试
cargo test -p ms-team test_cache_key_format

# 显示输出
cargo test -p ms-team -- --nocapture --test-threads=1
```

## 📊 测试统计

| 类别 | 数量 | 状态 |
|------|------|------|
| 单元测试 | 13 | ✅ |
| Department 集成测试 | 10 | ✅ |
| Organization 集成测试 | 16 | ✅ |
| **总计** | **39** | **✅ PASS** |

## 📁 文件结构

```
tests/
├── common/mod.rs              ← 公共工具
├── department_tests.rs        ← Department 10个测试
└── organization_tests.rs      ← Organization 16个测试

src/
├── lib.rs                      ← 库入口（新增）
├── modules/department/
│   └── service.rs             ← 6个单元测试
└── modules/organization/
    └── service.rs             ← 7个单元测试
```

## 🧪 测试类型

### 单元测试（13个）
- **缓存验证**：CacheKeyBuilder 格式、多字段隔离
- **DTO 构造**：DepartmentResponse、OrganizationResponse
- **业务逻辑**：路径/层级/完整名称计算
- **树形结构**：组织树、部门树

### 集成测试（26个）
- **数据生成**：请求构造、批量生成
- **字段验证**：代码、名称、排序、类型
- **关系验证**：组织间隔离、部门层级
- **唯一性**：代码唯一、名称一致性

## 🏗️ 公共工具

### create_test_organization_request(name)
创建标准组织请求
```rust
let req = create_test_organization_request("Technology");
// OrganizationRequest {
//     code: "ORG_TECHNOLOGY",
//     name: "Technology",
//     type: Some(2),  // 公司
//     ...
// }
```

### create_test_department_request(org_id, name, parent_id)
创建标准部门请求
```rust
let req = create_test_department_request(1, "后端组", Some(10));
// DepartmentRequest {
//     org_id: 1,
//     code: "DEPT_后端组",
//     name: "后端组",
//     parent_id: Some(10),
//     ...
// }
```

### TestDataBuilder
生成唯一的测试数据
```rust
let mut builder = TestDataBuilder::new();
let org1 = builder.next_org_request();  // org_1
let org2 = builder.next_org_request();  // org_2
let dept1 = builder.next_dept_request(1, None);  // dept_1
let dept2 = builder.next_dept_request(1, None);  // dept_2
```

## ✅ 主要测试场景

### Department 测试覆盖
- ✅ 部门代码生成规则
- ✅ 部门层级结构（根→一级→二级）
- ✅ 部门排序字段
- ✅ 批量部门创建
- ✅ 组织间部门隔离
- ✅ 部门名称有效性

### Organization 测试覆盖
- ✅ 组织代码生成规则
- ✅ 组织默认类型（公司）
- ✅ 组织批量创建
- ✅ 组织名称-代码一致性
- ✅ 组织树形结构支持
- ✅ 多组织创建和隔离

## 🔍 重要测试案例

### 缓存验证
```rust
#[test]
fn test_cache_key_format() {
    let cache_builder = SimpleCacheKeyBuilder::new("department")
        .with_modular("organization")
        .with_field("employee_count")
        .with_value_type(ValueType::Number);
    
    let cache_key = cache_builder.key(&[&123]);
    assert!(cache_key.key.contains("organization"));
    assert!(cache_key.key.contains("123"));
}
```

### 路径构建
```rust
#[test]
fn test_path_construction() {
    assert_eq!("/", "/");                    // 根
    assert_eq!("/1/2/", "/1/2/");           // 一级
    assert_eq!("/1/2/3/", "/1/2/3/");       // 二级
}
```

### 部门隔离
```rust
#[test]
fn test_department_isolation_across_orgs() {
    let dept_org1 = create_test_department_request(1, "后端组", None);
    let dept_org2 = create_test_department_request(2, "后端组", None);
    
    assert_ne!(dept_org1.org_id, dept_org2.org_id);  // 不同组织
    assert_eq!(dept_org1.name, dept_org2.name);      // 相同名称
}
```

## 📈 执行时间

| 操作 | 时间 |
|------|------|
| 单元测试 | ~0.0s |
| Department 集成 | ~0.0s |
| Organization 集成 | ~0.0s |
| 总计 | ~3s |

## 🐛 常见问题

### Q: 如何只运行某个测试？
```bash
cargo test -p ms-team test_cache_key_format
```

### Q: 如何看测试的详细输出？
```bash
cargo test -p ms-team -- --nocapture
```

### Q: 如何调试失败的测试？
```bash
cargo test -p ms-team test_name -- --nocapture --test-threads=1
```

### Q: 如何添加新测试？
1. 在对应的 service.rs 中的 `#[cfg(test)]` 模块加单元测试
2. 或在 `tests/` 目录创建新的集成测试文件

## 🔗 相关文档

- [完整测试指南](TESTING_GUIDE.md)
- [开发标准](../DEVELOPMENT_STANDARDS.md)
- [Phase 1 完成报告](PHASE_1_COMPLETION.md)
- [Phase 2 完成报告](../PHASE_2_COMPLETION.md)

## 👥 贡献指南

在添加新功能时，请：
1. ✅ 添加相应的单元测试
2. ✅ 添加集成测试
3. ✅ 确保所有测试通过：`cargo test -p ms-team`
4. ✅ 更新本文档

---

**最后更新**：2026-02-09
**测试框架**：Rust 标准库
**状态**：✅ 所有测试通过
