# ms-identity 与 ms-team 集成开发方案

**创建时间**: 2026-05-08
**最后更新**: 2026-05-08
**状态**: Phase 2 开发完成，解散组织流程设计中

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

- [x] **Step 1**: `role` 表加 `biz_id` 字段 + 更新唯一约束
- [x] **Step 2**: `Role` 实体增加 `biz_id` 字段（`auth/model/entity/role.rs` 已添加）
- [x] **Step 3**: `RoleService` 增加 `create_org_role` 方法（`auth/service.rs` 已实现）
- [x] **Step 4**: `identity.proto` 新增 3 个 RPC + message（已完成）
- [x] **Step 5**: 实现 `create_tenant_for_org`（`grpc/identity_service.rs`，固定 plan_id=6）
- [x] **Step 6**: 实现 `init_org_roles`（`grpc/identity_service.rs`，创建 owner/admin/member 角色）
- [x] **Step 7**: 实现 `assign_role`（`grpc/identity_service.rs`，创建 UserRole 记录）
- [x] **Step 8**: `grpc/identity_service.rs` 实现 3 个新 handler（全部完成）
- [x] **Step 9**: 编译验证 ✅（含 tenant_type 字段补充）

### Phase 2: ms-team 集成

- [x] **Step 10**: 同步 `identity.proto` 到 ms-team（已包含 3 个新 RPC + message）
- [x] **Step 11**: `IdentityClient` 新增 3 个 gRPC 调用方法（`client/identity.rs` 已完成）
- [x] **Step 12**: 改造 `organization/service.rs` 创建组织逻辑（集成 gRPC 调用链）
- [x] **Step 13**: 改造 `organization/handler.rs`（通过 service 层完成）
- [x] **Step 14**: 创建组织时自动创建员工（`service.rs` 第 267-282 行）
- [x] **Step 15**: 编译验证 ✅

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

---

## 八、逆向流程：解散组织

### 8.1 创建 ↔ 解散 对称关系

| 步骤 | 创建组织（正向） | 解散组织（逆向） |
|------|-----------------|-----------------|
| 1 | gRPC: CreateTenantForOrganization → 创建 Tenant + TenantUserRel | gRPC: **DestroyTenantForOrg** → 删除 TenantUserRel + 停用 Tenant |
| 2 | 本地: 创建 Organization + Department | 本地: 删除 Employee + Position + Department + Organization |
| 3 | gRPC: InitOrgRoles → 创建 3 个 Role | gRPC: **CleanupOrgRoles** → 删除 Role(biz_id=org_id) + 关联的 UserRole |
| 4 | 本地: 创建 Employee | （已在步骤2处理） |
| 5 | gRPC: AssignRole → 创建 UserRole | （已在步骤3处理） |

### 8.2 解散顶级组织（parent_id = null）

```
owner 解散组织 (DELETE /api/v1/team/organizations/:id)
  │
  ├─ 0. 前置校验
  │     → 无子组织
  │     → 操作人必须是 owner 角色
  │     → 二次确认（前端弹窗）
  │
  ├─ 1. 本地事务：删除 ms-team 数据
  │     → 删除所有员工（Employee）
  │     → 删除所有岗位（Position）
  │     → 删除所有部门（Department）
  │     → 删除 Organization 记录
  │
  ├─ 2. gRPC: CleanupOrgRoles(tenant_id, org_id)
  │     → 查找 Role { biz_id: org_id, tenant_id } 的所有角色
  │     → 删除 UserRole 中 role_id 在这些角色中的记录
  │     → 软删除 Role { biz_id: org_id } 的角色记录
  │     ← 返回删除数量
  │
  └─ 3. gRPC: DestroyTenantForOrg(tenant_id, owner_user_id)
        → 删除 TenantUserRel { tenant_id }（所有关联）
        → 停用 Tenant { status = 0 }（软停用，非硬删除）
```

### 8.3 解散子组织（parent_id ≠ null）

```
owner 解散子组织 (DELETE /api/v1/team/organizations/:id)
  │
  ├─ 0. 前置校验
  │     → 无子组织
  │     → 操作人必须是该组织的 owner 角色
  │
  ├─ 1. 本地事务：删除 ms-team 数据（同上）
  │
  └─ 2. gRPC: CleanupOrgRoles(tenant_id, org_id)
        → 删除该组织对应的 UserRole
        → 软删除该组织的 3 个角色
        ※ 不删除 Tenant 和 TenantUserRel（父组织仍在使用该租户）
```

### 8.4 新增 gRPC 接口

```protobuf
// 清理组织角色（解散组织时调用）
rpc CleanupOrgRoles(CleanupOrgRolesRequest) returns (CleanupOrgRolesResponse);

// 销毁组织租户（仅顶级组织解散时调用）
rpc DestroyTenantForOrg(DestroyTenantForOrgRequest) returns (DestroyTenantForOrgResponse);
```

### 8.5 新增 Message 定义

```protobuf
// 清理组织角色请求
message CleanupOrgRolesRequest {
  int64 tenant_id = 1;
  int64 org_id = 2;            // biz_id，用于定位该组织的角色
}

// 清理组织角色响应
message CleanupOrgRolesResponse {
  bool success = 1;
  string message = 2;
  int32 deleted_role_count = 3;       // 删除的角色数
  int32 deleted_user_role_count = 4;  // 删除的用户角色关系数
}

// 销毁组织租户请求
message DestroyTenantForOrgRequest {
  int64 tenant_id = 1;
  int64 owner_user_id = 2;     // 操作人（用于验证权限）
}

// 销毁组织租户响应
message DestroyTenantForOrgResponse {
  bool success = 1;
  string message = 2;
}
```

### 8.6 ms-identity 实现要点

#### CleanupOrgRoles Handler

```
1. 根据 (tenant_id, biz_id=org_id) 查询所有角色 → 获取 role_ids
2. 根据 role_ids 删除 user_role 表中关联记录
3. 软删除这些角色（is_del = 1）
4. 返回删除数量
```

#### DestroyTenantForOrg Handler

```
1. 验证 tenant_id 存在
2. 删除 tenant_user_rel 表中 tenant_id 的所有记录
3. 更新 tenant 表 status = 0（停用，保留数据便于审计）
4. 可选：清理 tenant_application_rel 关联
```

### 8.7 ms-team 实现要点

#### 改造 organization/service.rs delete 方法

当前 delete 方法仅处理本地数据，需在本地事务成功后增加：

```rust
// 伪代码
async fn delete(&self, id: i64, operator_user_id: i64) -> Result<()> {
    let org = self.get_by_id(id).await?;
    let is_top_level = org.parent_id.is_none();

    // ... 现有前置校验（子组织、员工、岗位、部门） ...

    // 1. 本地事务：删除 Employee + Position + Department + Organization
    self.delete_org_local_data(id).await?;

    // 2. gRPC 清理角色
    let cleanup = IdentityClient::cleanup_org_roles(org.tenant_id, id).await?;
    tracing::info!("清理角色: 删除 {} 个角色, {} 个用户角色关系",
        cleanup.deleted_role_count, cleanup.deleted_user_role_count);

    // 3. 顶级组织：销毁租户
    if is_top_level {
        IdentityClient::destroy_tenant_for_org(org.tenant_id, operator_user_id).await?;
        tracing::info!("租户 {} 已停用", org.tenant_id);
    }

    Ok(())
}
```

### 8.8 各服务改动清单

#### ms-identity 改动

| 文件 | 改动 |
|------|------|
| `proto/identity.proto` | 新增 2 个 RPC（CleanupOrgRoles, DestroyTenantForOrg）+ 4 个 message |
| `auth/service.rs` | RoleService 新增 `delete_org_roles(tenant_id, biz_id)` 方法 |
| `user/service.rs` | UserService 新增 `delete_user_roles_by_role_ids(role_ids)` 方法 |
| `tenant/service.rs` | TenantService 新增 `deactivate_tenant(tenant_id)` 方法 |
| `grpc/identity_service.rs` | 实现 2 个新 RPC handler |

#### ms-team 改动

| 文件 | 改动 |
|------|------|
| `proto/identity.proto` | 同步更新 |
| `client/identity.rs` | 新增 `cleanup_org_roles`, `destroy_tenant_for_org` 方法 |
| `organization/service.rs` | 改造 `delete` 方法，增加 gRPC 清理调用 |

### 8.9 实施步骤

#### Phase 1 补充: ms-identity 解散组织接口

- [ ] **Step D1**: `identity.proto` 新增 CleanupOrgRoles + DestroyTenantForOrg 及 message
- [ ] **Step D2**: `RoleService` 增加 `delete_org_roles` 方法
- [ ] **Step D3**: `UserService` 增加 `delete_user_roles_by_role_ids` 方法
- [ ] **Step D4**: `TenantService` 增加 `deactivate_tenant` 方法
- [ ] **Step D5**: `grpc/identity_service.rs` 实现 CleanupOrgRoles handler
- [ ] **Step D6**: `grpc/identity_service.rs` 实现 DestroyTenantForOrg handler

#### Phase 2 补充: ms-team 集成解散流程

- [ ] **Step D7**: 同步 `identity.proto` 到 ms-team
- [ ] **Step D8**: `IdentityClient` 新增 `cleanup_org_roles` + `destroy_tenant_for_org` 方法
- [ ] **Step D9**: 改造 `organization/service.rs` delete 方法（本地事务 + gRPC 清理）

#### Phase 3 补充: 前端解散组织

- [ ] **Step D10**: 解散组织确认弹窗（二次确认）
- [ ] **Step D11**: 解散成功后跳转处理（返回组织列表或引导页）

### 8.10 关键注意事项

1. **执行顺序**: 先删本地数据（ms-team），再删远端数据（ms-identity），避免本地残留
2. **顶级 vs 子组织**: 只有顶级组织才调用 DestroyTenantForOrg，子组织只清理角色
3. **权限控制**: 只有 owner 角色才能解散组织，需在 handler 层校验
4. **租户停用而非删除**: DestroyTenantForOrg 只停用 Tenant（status=0），不硬删除，便于审计和恢复
5. **失败补偿**: 如果 gRPC 调用失败但本地数据已删除，可通过定时任务扫描孤立角色进行补偿清理
