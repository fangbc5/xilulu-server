# Task 1.3 - Service 层业务验证规则矩阵

**创建日期**: 2026-02-06  
**任务**: Service 层业务异常处理  
**状态**: 规则设计完成，待实施  

---

## 📊 验证规则矩阵

### 1️⃣ OrganizationService - 组织管理

| 方法 | 参数 | 当前验证 | ✅ 已实施 | 需要添加 | 错误返回 |
|------|------|---------|---------|---------|--------|
| `create(tenant_id, req)` | code, parent_id | 代码唯一性 | ✅ | 无 | `OrganizationExists` |
| | | 父级存在性 | ✅ | 无 | `OrganizationNotFound` |
| `get_by_id(id)` | id | 存在性检查 | ✅ | 无 | `OrganizationNotFound` |
| `update(id, req)` | id | 存在性检查 | ✅ | 无 | `OrganizationNotFound` |
| `delete(id)` | id | 存在性检查 | ✅ | 无 | `OrganizationNotFound` |
| | | 子组织检查 | ✅ | 无 | `BusinessConflict` |
| | | 员工数检查 | ✅ | 无 | `BusinessConflict` |
| | | 岗位数检查 | ✅ | 无 | `BusinessConflict` |
| | | 部门数检查 | ✅ | 无 | `BusinessConflict` |

**现状**: ✅ 已完整实施所有验证

---

### 2️⃣ DepartmentService - 部门管理

| 方法 | 参数 | 当前验证 | ✅ 已实施 | 需要添加 | 错误返回 |
|------|------|---------|---------|---------|--------|
| `create(tenant_id, req)` | code, org_id | 代码唯一性 | ✅ | 无 | `DepartmentExists` |
| | | 父级存在性 | ✅ | 无 | `DepartmentNotFound` |
| | | 组织存在性 | ❌ | **✏️ 需要** | `OrganizationNotFound` |
| `get_by_id(id)` | id | 存在性检查 | ✅ | 无 | `DepartmentNotFound` |
| `update(id, req)` | id | 存在性检查 | ✅ | 无 | `DepartmentNotFound` |
| `delete(id)` | id | 存在性检查 | ✅ | 无 | `DepartmentNotFound` |
| | | 子部门检查 | ✅ | 无 | `DepartmentHasChildren` |
| | | 员工检查 | ✅ | 无 | `DepartmentHasEmployees` |

**需要修复**: 1 个 - 创建部门时需要验证组织是否存在

---

### 3️⃣ PositionService - 岗位管理

| 方法 | 参数 | 当前验证 | ✅ 已实施 | 需要添加 | 错误返回 |
|------|------|---------|---------|---------|--------|
| `create(tenant_id, req)` | code, org_id | 代码唯一性 | ✅ | 无 | `PositionExists` |
| | | 组织存在性 | ❌ | **✏️ 需要** | `OrganizationNotFound` |
| `get_by_id(id)` | id | 存在性检查 | ✅ | 无 | `PositionNotFound` |
| `update(id, req)` | id | 存在性检查 | ✅ | 无 | `PositionNotFound` |
| `delete(id)` | id | 存在性检查 | ✅ | 无 | `PositionNotFound` |
| | | 员工检查 | ✅ | 无 | `PositionHasEmployees` |

**需要修复**: 1 个 - 创建岗位时需要验证组织是否存在

---

### 4️⃣ EmployeeService - 员工管理

| 方法 | 参数 | 当前验证 | ✅ 已实施 | 需要添加 | 错误返回 |
|------|------|---------|---------|---------|--------|
| `create(tenant_id, req)` | org_id, user_id | 用户是否已是员工 | ✅ | 无 | `UserAlreadyEmployee` |
| | | 工号唯一性 | ✅ | 无 | `EmployeeNoExists` |
| | | 组织存在性 | ❌ | **✏️ 需要** | `OrganizationNotFound` |
| | | 用户存在性 | ❌ | **✏️ 需要** | `UserNotFound` |
| `get_by_id(id)` | id | 存在性检查 | ✅ | 无 | `EmployeeNotFound` |
| `update(id, req)` | id | 存在性检查 | ✅ | 无 | `EmployeeNotFound` |
| `delete(id)` | id | 存在性检查 | ✅ | 无 | `EmployeeNotFound` |
| | | 关系清理 | ⚠️ TODO | **✏️ 需要** | - |

**需要修复**: 4 个 - 组织/用户验证、删除关系清理

---

### 5️⃣ EmployeeDepartmentService - 员工部门关系

| 方法 | 参数 | 当前验证 | ✅ 已实施 | 需要添加 | 错误返回 |
|------|------|---------|---------|---------|--------|
| `add_to_department(...)` | emp_id, dept_id | 关系不重复 | ✅ | 无 | `EmployeeDepartmentRelExists` |
| | | 员工存在性 | ❌ | **✏️ 需要** | `EmployeeNotFound` |
| | | 部门存在性 | ❌ | **✏️ 需要** | `DepartmentNotFound` |
| | | 主部门冲突 | ✅ | 无 | (自动清除) |
| `remove_from_department(...)` | emp_id, dept_id | 关系存在 | ✅ | 无 | `EmployeeDepartmentRelNotFound` |

**需要修复**: 2 个 - 员工、部门存在性验证

---

### 6️⃣ EmployeePositionService - 员工岗位关系

| 方法 | 参数 | 当前验证 | ✅ 已实施 | 需要添加 | 错误返回 |
|------|------|---------|---------|---------|--------|
| `add_position(...)` | emp_id, pos_id | 关系不重复 | ✅ | 无 | `EmployeePositionRelExists` |
| | | 员工存在性 | ❌ | **✏️ 需要** | `EmployeeNotFound` |
| | | 岗位存在性 | ❌ | **✏️ 需要** | `PositionNotFound` |
| | | 主岗位冲突 | ✅ | 无 | (自动清除) |
| `remove_position(...)` | emp_id, pos_id | 关系存在 | ✅ | 无 | `EmployeePositionRelNotFound` |

**需要修复**: 2 个 - 员工、岗位存在性验证

---

## 🔍 详细验证规则

### OrganizationService

#### ✅ `create()` - 完整实施

```rust
pub async fn create(&self, tenant_id: i64, req: CreateOrganizationRequest, created_by: Option<i64>) 
    -> Result<i64, OrganizationError> {
    // 1. 检查编码是否已存在
    if OrganizationRepo::find_by_tenant_and_code(
        pool, tenant_id, &req.code
    ).await? {
        return Err(OrganizationError::OrganizationExists);  // ✅
    }
    
    // 2. 如果有上级，检查上级是否存在
    if let Some(parent_id) = req.parent_id {
        if !Organization::find_by_id(pool, parent_id).await? {
            return Err(OrganizationError::OrganizationNotFound);  // ✅
        }
    }
    
    // ... 创建逻辑
}
```

#### ✅ `delete()` - 完整实施

```rust
pub async fn delete(&self, id: i64) -> Result<(), OrganizationError> {
    // 1. 检查组织是否存在
    self.get_by_id(id).await?;  // ✅
    
    // 2. 检查子组织
    if OrganizationRepo::has_children(pool, id).await? {
        return Err(OrganizationError::BusinessConflict("存在下级组织".into()));  // ✅
    }
    
    // 3. 检查员工
    if EmployeeRepo::count_by_org_id(pool, id).await? > 0 {
        return Err(OrganizationError::BusinessConflict("组织下存在员工".into()));  // ✅
    }
    
    // 4. 检查岗位
    if PositionRepo::count_by_org_id(pool, id).await? > 0 {
        return Err(OrganizationError::BusinessConflict("组织下存在岗位".into()));  // ✅
    }
    
    // 5. 检查部门
    if DepartmentRepo::find_by_org_id(pool, id).await?.len() > 1 {
        return Err(OrganizationError::BusinessConflict("组织下存在下级部门".into()));  // ✅
    }
}
```

---

### DepartmentService

#### ⚠️ `create()` - 需要修复：添加组织存在性检查

```rust
pub async fn create(&self, tenant_id: i64, req: CreateDepartmentRequest, created_by: Option<i64>) 
    -> Result<i64, OrganizationError> {
    // 1. 检查编码是否已存在 ✅ 已有
    if DepartmentRepo::find_by_org_and_code(pool, req.org_id, &req.code).await? {
        return Err(OrganizationError::DepartmentExists(req.code));
    }
    
    // 2. ❌ 缺少：检查组织是否存在
    // 需要添加：
    if !Organization::find_by_id(pool, req.org_id).await? {
        return Err(OrganizationError::OrganizationNotFound);  // ✏️ 需要添加
    }
    
    // 3. 检查父级部门是否存在 ✅ 已有
    if let Some(parent_id) = req.parent_id {
        if !Department::find_by_id(pool, parent_id).await? {
            return Err(OrganizationError::DepartmentNotFound);
        }
    }
    
    // ... 创建逻辑
}
```

#### ✅ `delete()` - 完整实施

```rust
pub async fn delete(&self, id: i64) -> Result<(), OrganizationError> {
    // 1. 检查部门是否存在 ✅
    self.get_by_id(id).await?;
    
    // 2. 检查子部门 ✅
    if DepartmentRepo::has_children(pool, id).await? {
        return Err(OrganizationError::DepartmentHasChildren);
    }
    
    // 3. 检查员工 ✅
    if EmployeeDepartmentRepo::has_employees(pool, id).await? {
        return Err(OrganizationError::DepartmentHasEmployees);
    }
}
```

---

### PositionService

#### ⚠️ `create()` - 需要修复：添加组织存在性检查

```rust
pub async fn create(&self, tenant_id: i64, req: CreatePositionRequest, created_by: Option<i64>) 
    -> Result<i64, OrganizationError> {
    // 1. 检查编码是否已存在 ✅
    if PositionRepo::find_by_org_and_code(pool, req.org_id, &req.code).await? {
        return Err(OrganizationError::PositionExists(req.code));
    }
    
    // 2. ❌ 缺少：检查组织是否存在
    // 需要添加：
    if !Organization::find_by_id(pool, req.org_id).await? {
        return Err(OrganizationError::OrganizationNotFound);  // ✏️ 需要添加
    }
    
    // ... 创建逻辑
}
```

#### ✅ `delete()` - 完整实施

```rust
pub async fn delete(&self, id: i64) -> Result<(), OrganizationError> {
    // 1. 检查岗位是否存在 ✅
    self.get_by_id(id).await?;
    
    // 2. 检查员工 ✅
    if EmployeePositionRepo::has_employees(pool, id).await? {
        return Err(OrganizationError::PositionHasEmployees);
    }
}
```

---

### EmployeeService

#### ⚠️ `create()` - 需要修复：添加组织和用户存在性检查

```rust
pub async fn create(&self, tenant_id: i64, req: CreateEmployeeRequest, created_by: Option<i64>) 
    -> Result<i64, OrganizationError> {
    // 1. 检查用户是否已是员工 ✅
    if EmployeeRepo::find_by_org_and_user(pool, req.org_id, req.user_id).await? {
        return Err(OrganizationError::UserAlreadyEmployee);
    }
    
    // 2. 检查工号是否存在 ✅
    if EmployeeRepo::find_by_org_and_employee_no(pool, req.org_id, &req.employee_no).await? {
        return Err(OrganizationError::EmployeeNoExists(req.employee_no));
    }
    
    // 3. ❌ 缺少：检查组织是否存在
    // 需要添加：
    if !Organization::find_by_id(pool, req.org_id).await? {
        return Err(OrganizationError::OrganizationNotFound);  // ✏️ 需要添加
    }
    
    // 4. ❌ 缺少：检查用户是否存在（需要调用 identity 服务）
    // 需要添加：
    if !IdentityClient::check_user_exists(req.user_id).await? {
        return Err(OrganizationError::UserNotFound);  // ✏️ 需要添加
    }
    
    // ... 创建逻辑 + 部门、岗位关系创建
}
```

#### ⚠️ `delete()` - 需要清理关系

```rust
pub async fn delete(&self, id: i64) -> Result<(), OrganizationError> {
    // 1. 检查员工是否存在 ✅
    let emp = self.get_by_id(id).await?;
    
    // 2. ❌ TODO：清理员工部门关系
    // 需要添加：
    let dept_rels = EmployeeDepartmentService::get_by_employee(id).await?;
    for rel in dept_rels {
        if let Some(rel_id) = rel.id {
            EmployeeDepartment::delete_by_id(pool, rel_id).await?;  // ✏️ 需要添加
        }
    }
    
    // 3. ❌ TODO：清理员工岗位关系
    // 需要添加：
    let pos_rels = EmployeePositionService::get_by_employee(id).await?;
    for rel in pos_rels {
        if let Some(rel_id) = rel.id {
            EmployeePosition::delete_by_id(pool, rel_id).await?;  // ✏️ 需要添加
        }
    }
    
    // 4. 删除员工 ✅
    Employee::delete_by_id(pool, id).await?;
}
```

---

### EmployeeDepartmentService

#### ⚠️ `add_to_department()` - 需要添加存在性检查

```rust
pub async fn add_to_department(&self, tenant_id: i64, emp_id: i64, dept_id: i64, ...) 
    -> Result<i64, OrganizationError> {
    // 1. 检查关系不重复 ✅
    if EmployeeDepartmentRepo::find_by_employee_and_department(pool, emp_id, dept_id).await? {
        return Err(OrganizationError::EmployeeDepartmentRelExists);
    }
    
    // 2. ❌ 缺少：检查员工是否存在
    // 需要添加：
    if !Employee::find_by_id(pool, emp_id).await? {
        return Err(OrganizationError::EmployeeNotFound);  // ✏️ 需要添加
    }
    
    // 3. ❌ 缺少：检查部门是否存在
    // 需要添加：
    if !Department::find_by_id(pool, dept_id).await? {
        return Err(OrganizationError::DepartmentNotFound);  // ✏️ 需要添加
    }
    
    // 4. 如果设置为主部门，清除其他主部门 ✅
    if is_primary {
        EmployeeDepartmentRepo::clear_primary_by_employee_id(pool, emp_id).await?;
    }
    
    // ... 创建关系
}
```

---

### EmployeePositionService

#### ⚠️ `add_position()` - 需要添加存在性检查

```rust
pub async fn add_position(&self, tenant_id: i64, emp_id: i64, pos_id: i64, ...) 
    -> Result<i64, OrganizationError> {
    // 1. 检查关系不重复 ✅
    if EmployeePositionRepo::find_by_employee_and_position(pool, emp_id, pos_id).await? {
        return Err(OrganizationError::EmployeePositionRelExists);
    }
    
    // 2. ❌ 缺少：检查员工是否存在
    // 需要添加：
    if !Employee::find_by_id(pool, emp_id).await? {
        return Err(OrganizationError::EmployeeNotFound);  // ✏️ 需要添加
    }
    
    // 3. ❌ 缺少：检查岗位是否存在
    // 需要添加：
    if !Position::find_by_id(pool, pos_id).await? {
        return Err(OrganizationError::PositionNotFound);  // ✏️ 需要添加
    }
    
    // 4. 如果设置为主岗位，清除其他主岗位 ✅
    if is_primary {
        EmployeePositionRepo::clear_primary_by_employee_id(pool, emp_id).await?;
    }
    
    // ... 创建关系
}
```

---

## 📈 实施清单

### 需要修改的 Service 类和方法：

| Service | 方法 | 行数 | 需要添加的验证 | 优先级 |
|---------|------|------|--------------|--------|
| **DepartmentService** | `create()` | ~100 | 组织存在性 | P0 |
| **PositionService** | `create()` | ~70 | 组织存在性 | P0 |
| **EmployeeService** | `create()` | ~120 | 组织存在性、用户存在性 | P0 |
| **EmployeeService** | `delete()` | ~10 | 关系清理 | P0 |
| **EmployeeDepartmentService** | `add_to_department()` | ~50 | 员工/部门存在性 | P1 |
| **EmployeePositionService** | `add_position()` | ~50 | 员工/岗位存在性 | P1 |

**总计**: 6 个修改点，约 400 行代码需要检查/修复

---

## ⏱️ 预计时间分配

| 任务 | 时间 |
|------|------|
| DepartmentService 修复 | 30 分钟 |
| PositionService 修复 | 20 分钟 |
| EmployeeService 修复 | 45 分钟 |
| 关系 Service(Dept/Pos) 修复 | 40 分钟 |
| 编译验证 + 测试 | 15 分钟 |
| **总计** | **2.5 小时** |

---

## 🎯 实施顺序（按优先级）

### Phase 1 (P0 - 必需)
1. ✏️ DepartmentService.create() - 添加组织存在性检查
2. ✏️ PositionService.create() - 添加组织存在性检查
3. ✏️ EmployeeService.create() - 添加组织和用户存在性检查
4. ✏️ EmployeeService.delete() - 添加关系清理

### Phase 2 (P1 - 重要)
5. ✏️ EmployeeDepartmentService.add_to_department() - 添加员工/部门存在性
6. ✏️ EmployeePositionService.add_position() - 添加员工/岗位存在性

---

## 验证验收标准

- [ ] 所有新增验证都用正确的错误类型返回
- [ ] 没有使用 unwrap()/expect()，所有错误都通过 ?
- [ ] 所有外部实体引用都已验证存在
- [ ] 删除操作都检查了依赖项
- [ ] 编译通过：`cargo check` ✅
- [ ] 完成报告已生成

