# ms-identity 与 ms-team 集成开发方案

**创建时间**: 2026-05-08
**状态**: 待实施

---

## 一、架构概览

```
┌─────────────┐     HTTP/JWT      ┌─────────────┐     gRPC      ┌──────────────┐
│  Console     │ ───Bearer Token──→│   ms-auth    │ ─────────────→│  ms-identity  │
│  (前端)      │                   │  :8080       │               │  :9090        │
└─────────────┘                   └─────────────┘               └──────────────┘
       │                                                                ↑
       │──Bearer Token──→ ┌─────────────┐     gRPC      ┌──────────────┘
       │                  │   ms-team    │ ──────────────┘
       └─────────────────→│  :30101      │
                          └─────────────┘
```

### 服务职责

| 服务 | 职责 |
|------|------|
| **ms-auth** | JWT Token 生成/验证/刷新，登录/登出 |
| **ms-identity** | 用户管理、租户管理、角色/权限管理（gRPC 服务端） |
| **ms-team** | 组织、部门、员工、职位管理（gRPC 客户端，调用 ms-identity） |

---

## 二、核心流程：创建组织

### 2.1 顶级组织创建（parent_id = null）

```
用户创建组织 (POST /api/v1/team/organizations)
  │
  ├─ 1. gRPC 调 ms-identity: CreateTenantForOrganization
  │     → 创建 Tenant { name: 组织名 }
  │     → 创建 TenantUserRel { user_id, tenant_id, is_owner=1 }
  │     ← 返回 tenant_id
  │
  ├─ 2. 创建 Organization { tenant_id, name, code }
  │
  ├─ 3. gRPC 调 ms-identity: InitOrgRoles
  │     → 创建 Role { code: "owner",   biz_id: org_id, tenant_id }
  │     → 创建 Role { code: "admin",   biz_id: org_id, tenant_id }
  │     → 创建 Role { code: "member",  biz_id: org_id, tenant_id }
  │     ← 返回 role_ids
  │
  ├─ 4. 创建 Employee { org_id, user_id, name, tenant_id }
  │
  └─ 5. gRPC 调 ms-identity: AssignRole
        → UserRole { user_id, role_id=owner_role_id, tenant_id }
```

### 2.2 子组织创建（parent_id ≠ null）

```
用户创建子组织 (POST /api/v1/team/organizations)
  │
  ├─ 1. 查父组织 → 继承 tenant_id
  │
  ├─ 2. 创建 Organization { tenant_id, parent_id, name, code }
  │
  ├─ 3. gRPC 调 ms-identity: InitOrgRoles（同上，创建 3 个角色）
  │
  ├─ 4. 创建 Employee { org_id, user_id, tenant_id }
  │
  ├─ 5. gRPC 调 ms-identity: AssignRole (owner)
  │
  └─ 6. 如果该用户还不在该租户下，创建 TenantUserRel
```

---

## 三、数据库改动

### 3.1 ms-identity: `role` 表增加字段

```sql
-- 新增 biz_id 字段
ALTER TABLE role ADD COLUMN biz_id BIGINT DEFAULT NULL COMMENT '业务关联ID（组织角色=org_id）';
CREATE INDEX idx_role_biz_id ON role(biz_id);

-- 唯一约束从 code 改为 (tenant_id, code, biz_id) 联合唯一
ALTER TABLE role DROP INDEX idx_role_code;
CREATE UNIQUE INDEX uk_role_tenant_code_biz ON role(tenant_id, code, biz_id);
```

### 3.2 Role 实体改动（ms-identity）

```rust
// ms-identity/src/modules/auth/model/entity/role.rs
#[derive(Clone, Debug, Serialize, Deserialize, sqlxplus::FromRow)]
pub struct Role {
    // ... 现有字段 ...
    /// 业务关联ID（组织角色保存 org_id）
    pub biz_id: Option<i64>,
}
```

### 3.3 Role 编码规则

同一租户下，每个组织有 3 个标准角色，通过 `biz_id` (org_id) 区分：

| tenant_id | code    | biz_id (org_id) | name       |
|-----------|---------|-----------------|------------|
| 100       | owner   | 1               | 组织所有者  |
| 100       | admin   | 1               | 组织管理员  |
| 100       | member  | 1               | 组织成员    |
| 100       | owner   | 2               | 组织所有者  |  ← 子组织
| 100       | admin   | 2               | 组织管理员  |
| 100       | member  | 2               | 组织成员    |

### 3.4 ms-team: `employee` 表

无结构变更，已有 `user_id` 和 `tenant_id` 字段可直接使用。

---

## 四、gRPC 接口定义

### 4.1 identity.proto 新增 RPC

```protobuf
service IdentityService {
  // ... 现有接口 ...

  // 为组织创建租户（顶级组织时调用）
  rpc CreateTenantForOrganization(CreateTenantForOrgRequest) returns (CreateTenantForOrgResponse);

  // 初始化组织角色（创建 owner/admin/member 三个角色）
  rpc InitOrgRoles(InitOrgRolesRequest) returns (InitOrgRolesResponse);

  // 分配角色给用户
  rpc AssignRole(AssignRoleRequest) returns (AssignRoleResponse);
}
```

### 4.2 新增 Message 定义

```protobuf
// ===== 组织驱动租户相关 =====

// 为组织创建租户请求
message CreateTenantForOrgRequest {
  string org_name = 1;       // 组织名（同时作为租户名）
  int64 owner_user_id = 2;   // 创建人用户ID
  string contact_name = 3;   // 联系人姓名
  string contact_mobile = 4; // 联系人手机号（可选）
}

// 为组织创建租户响应
message CreateTenantForOrgResponse {
  bool success = 1;
  string message = 2;
  int64 tenant_id = 3;
}

// 初始化组织角色请求
message InitOrgRolesRequest {
  int64 tenant_id = 1;
  int64 org_id = 2;          // biz_id
  int64 created_by = 3;      // 创建人
}

// 初始化组织角色响应
message InitOrgRolesResponse {
  bool success = 1;
  string message = 2;
  int64 owner_role_id = 3;   // owner 角色 ID
  int64 admin_role_id = 4;   // admin 角色 ID
  int64 member_role_id = 5;  // member 角色 ID
}

// 分配角色请求
message AssignRoleRequest {
  int64 user_id = 1;
  int64 role_id = 2;
  string role_code = 3;
  int64 tenant_id = 4;
}

// 分配角色响应
message AssignRoleResponse {
  bool success = 1;
  string message = 2;
}
```

---

## 五、各服务代码改动清单

### 5.1 ms-identity 改动

| 文件 | 改动 |
|------|------|
| `proto/identity.proto` | 新增 3 个 RPC + message 定义 |
| `auth/model/entity/role.rs` | Role 增加 `biz_id: Option<i64>` |
| `auth/repository.rs` | RoleRepo 支持 `biz_id` 查询 |
| `tenant/service.rs` 或新 service | 实现 `create_tenant_for_org` |
| `auth/service.rs` 或新 service | 实现 `init_org_roles`, `assign_role` |
| `grpc/identity_service.rs` | 实现 3 个新 RPC handler |

### 5.2 ms-team 改动

| 文件 | 改动 |
|------|------|
| `proto/identity.proto` | 同步更新（ms-team 依赖此 proto 生成客户端） |
| `client/identity.rs` | 新增 `create_tenant_for_org`, `init_org_roles`, `assign_role` 方法 |
| `modules/organization/service.rs` | 封装组织创建事务逻辑（调用 gRPC） |
| `modules/organization/handler.rs` | `create_organization` 改造 |
| `modules/employee/service.rs` 或 handler | 创建组织时自动创建员工 |

### 5.3 前端（Console）改动

| 文件 | 改动 |
|------|------|
| `services/organization.rs` | 创建组织 API 对接 |
| 新增组织引导页面 | 首次无组织时的引导流程 |

---

## 六、实施步骤

### Phase 1: ms-identity 基础改造

- [ ] **Step 1**: `role` 表加 `biz_id` 字段 + 更新唯一约束
- [ ] **Step 2**: `Role` 实体增加 `biz_id` 字段
- [ ] **Step 3**: `RoleRepo` 增加 `biz_id` 相关查询方法
- [ ] **Step 4**: `identity.proto` 新增 3 个 RPC + message
- [ ] **Step 5**: 实现 `create_tenant_for_org` 服务方法
- [ ] **Step 6**: 实现 `init_org_roles` 服务方法
- [ ] **Step 7**: 实现 `assign_role` 服务方法
- [ ] **Step 8**: `grpc/identity_service.rs` 实现 3 个新 handler
- [ ] **Step 9**: 编译验证

### Phase 2: ms-team 集成

- [ ] **Step 10**: 同步 `identity.proto` 到 ms-team
- [ ] **Step 11**: `IdentityClient` 新增 3 个 gRPC 调用方法
- [ ] **Step 12**: 改造 `organization/service.rs` 创建组织逻辑
- [ ] **Step 13**: 改造 `organization/handler.rs`
- [ ] **Step 14**: 创建组织时自动创建员工
- [ ] **Step 15**: 编译验证

### Phase 3: 前端对接

- [ ] **Step 16**: `services/organization.rs` 创建组织 API
- [ ] **Step 17**: 组织引导页面（首次无组织时）
- [ ] **Step 18**: 端到端测试

---

## 七、注意事项

1. **事务一致性**: ms-team 创建组织是跨服务操作（本地 DB + gRPC），需考虑失败回滚策略
2. **幂等性**: `CreateTenantForOrganization` 需处理重复调用场景
3. **权限校验**: 创建子组织时需验证用户是否属于父组织所在租户
4. **角色清理**: 删除组织时需同步清理 ms-identity 中对应的角色和用户角色关系