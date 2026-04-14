# 企业级部门管理功能清单

**创建日期**: 2026-02-09  
**版本**: 2.0（MVP 核心方案）  
**状态**: 待实施  

---

## 📋 需求分析

### 系统约束
- **部门数量**: 数千 ~ 1万级别
- **性能目标**: 200ms 内完成查询
- **关键需求**: 每个部门显示其下 **所有员工数**（含子部门）
- **用户体验**: 前端懒加载，按需下钻

### 设计核心原则
- ✅ **分层下钻** - 不一次加载整棵树（会超过 200ms）
- ✅ **真实员工统计** - 每个部门返回 total_employee_count（含下属）
- ✅ **高效查询** - 利用路径索引快速统计下属员工
- ✅ **Redis 缓存** - 员工数统计结果缓存 5-10 分钟

---

## 🎯 MVP 核心功能清单

### ⭐ 必需功能（第2周实现）

| 功能 | 端点 | 方法 | 说明 | 关键指标 |
|------|------|------|------|---------|
| 创建部门 | `POST /api/departments` | POST | ✅ 已实现 | - |
| 获取部门详情 | `GET /api/departments/{id}` | GET | ✅ 已实现 | 包含 total_employee_count |
| 列表查询 | `GET /api/departments?org_id=X` | GET | ✅ 已实现 | 包含 total_employee_count |
| 更新部门 | `PUT /api/departments/{id}` | PUT | ✅ 已实现 | - |
| 删除部门 | `DELETE /api/departments/{id}` | DELETE | ✅ 已实现 | - |
| **获取一级部门（含员工数）** | `GET /api/departments/roots?org_id=X` | GET | ⭐ **新增** | **返回所有一级部门 + 每个的 total_employee_count** |
| **分层下钻（直属子部门）** | `GET /api/departments/{id}/children` | GET | ⭐ **新增** | **返回直属子部门 + 每个的 total_employee_count** |
| 获取完整树 | `GET /api/departments/tree?org_id=X` | GET | ✅ 已实现（限 500） | 部门数 < 500 才取整棵树 |

### 📊 性能优化（关键）

| 项目 | 实现方式 | 性能目标 |
|------|---------|---------|
| **一级部门查询** | 直接查询 (无递归) + Redis 缓存employee_count | < 50ms (缓存命中) |
| **子部门查询** | 按 parent_id 查询 + Redis 缓存 | < 100ms (缓存命中) |
| **员工数统计** | 利用 path 索引快速统计 + 5min 缓存 | < 20ms (缓存) / < 200ms (DB) |
| **整体响应** | 并行查询 + 缓存 | **< 200ms** ✅

---

## 📊 核心数据结构更新

### 1. DepartmentResponse 扩展（最小化）

```rust
#[derive(Debug, Serialize)]
pub struct DepartmentResponse {
    // 基础字段（现有）
    pub id: i64,
    pub tenant_id: i64,
    pub org_id: i64,
    pub parent_id: Option<i64>,
    pub code: String,
    pub name: String,
    pub full_name: Option<String>,
    pub path: Option<String>,
    pub level: Option<i32>,
    pub leader_employee_id: Option<i64>,
    pub sort_order: Option<i32>,
    pub status: Option<i16>,
    
    // ⭐ 新增字段（MVP 核心）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_count: Option<i64>,          // 直属员工数
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_employee_count: Option<i64>,    // 所有下属员工总数（包括子部门）⭐⭐⭐
}
```

**说明**:
- `employee_count`: 仅直属于本部门的员工数
- `total_employee_count`: 本部门 + 所有子部门的员工总数 （**前端必需显示**）
- 两个字段都可选，避免一次查询时不必要的统计开销

### 2. 新增查询 DTO

```rust
#[derive(Debug, Deserialize)]
pub struct GetDepartmentChildrenQuery {
    pub org_id: i64,                   // 必需：组织ID
    pub parent_id: Option<i64>,        // 可选：父部门ID (None = 一级部门)
    
    #[serde(default = "default_page")]
    pub page: i32,                     // 页码，默认1
    
    #[serde(default = "default_page_size")]
    pub page_size: i32,                // 每页数量，默认20
    
    pub keyword: Option<String>,       // 搜索关键词
}

fn default_page() -> i32 { 1 }
fn default_page_size() -> i32 { 20 }
```

---

## � 关键实现细节

### 场景1: 获取一级部门（前端初始化）

```
请求: GET /api/departments/roots?org_id=123

步骤1: 查询一级部门 (parent_id IS NULL)
  SQL: SELECT * FROM department WHERE org_id=123 AND parent_id IS NULL ORDER BY sort_order

步骤2: 对每个部门统计员工总数（包括下属）
  SQL: SELECT COUNT(*) FROM employee_department 
       WHERE dept_id IN (
         SELECT id FROM department 
         WHERE path LIKE '{parent_path}%'  -- 利用path快速查询所有下属部门
       ) AND status = 1

步骤3: 使用 Redis 缓存结果 5-10 分钟
  Key: dept:tree:org_id:123
  Value: 一级部门列表 + employee_count

响应 (< 50ms 缓存命中):
{
  "code": 0,
  "data": [
    {
      "id": 1,
      "name": "技术部",
      "employee_count": 5,          // 直属
      "total_employee_count": 25,   // 含子部门
      "has_children": true          // 提示前端可以展开
    },
    ...
  ]
}
```

### 场景2: 用户展开某部门（懒加载）

```
请求: GET /api/departments/1/children?org_id=123&page=1&page_size=20

步骤1: 快速查询直属子部门
  SQL: SELECT * FROM department 
       WHERE org_id=123 AND parent_id=1 
       LIMIT 20

步骤2: 对每个子部门统计员工总数（带缓存）
  For each dept_id:
    - 先从 Redis 查 dept:employee_count:{dept_id}
    - 未命中则: SELECT COUNT(*) FROM employee_department 
               WHERE dept_id IN (SELECT id FROM department WHERE path LIKE ...)
    - 结果缓存到 Redis

步骤3: 返回结果

响应 (< 100ms):
{
  "code": 0,
  "data": {
    "list": [
      {"id": 5, "name": "后端部门", "total_employee_count": 12, ...},
      {"id": 6, "name": "前端部门", "total_employee_count": 8, ...},
      ...
    ],
    "total": 45,
    "page": 1,
    "page_size": 20
  }
}
```

### 关键SQL优化

```sql
-- 1. 部门表索引优化（必需）
ALTER TABLE department ADD INDEX idx_org_id_parent (org_id, parent_id);
ALTER TABLE department ADD INDEX idx_path (path);  -- 用于快速查询下属部门

-- 2. 员工部门表优化（必需）
ALTER TABLE employee_department ADD INDEX idx_dept_id_status (dept_id, status);

-- 3. 统计查询（优化版）
-- 快速统计某部门的所有下属员工数
SELECT COUNT(DISTINCT ed.employee_id) 
FROM employee_department ed
INNER JOIN department d ON ed.dept_id = d.id
WHERE d.path LIKE '/123/%'   -- path 索引，非常快
  AND ed.status = 1
  AND d.tenant_id = {tenant_id};
```

---

## 🏗️ 架构设计

### 代码层次结构

```
HTTP Handler (handler.rs)
  ├─ get_roots()              ← 新增：获取一级部门
  ├─ get_children()           ← 新增：获取子部门
  └─ （其他现有接口）

    ↓

Service Layer (service.rs)
  ├─ get_roots(org_id)        ← 新增
  ├─ get_children(parent_id)  ← 新增
  ├─ count_employees(dept_id) ← 新增：统计某部门的所有下属员工数
  ├─ count_direct_employees(dept_id) ← 新增：统计直属员工数
  └─ （其他现有方法）

    ↓

Repository Layer (repository.rs)
  ├─ find_roots(org_id)       ← 新增：查询一级部门
  ├─ find_children(parent_id) ← 新增：查询直属子部门
  └─ count_employees_in_path(path) ← 新增：利用 path 快速统计下属员工

    ↓

Database + Redis Cache
  ├─ MySQL: department, employee_department 表
  └─ Redis: dept:{dept_id}:employee_count (TTL 300s)
```

### Redis 缓存策略

```rust
// 缓存键设计
Key Pattern: dept:{org_id}:{dept_id}:employee_count
TTL: 300 seconds (5 minutes)

// 触发场景
1. 查询一级部门时，逐个查询或批量查询缓存
2. 查询子部门时，同样查询缓存
3. 创建/删除/转移员工时，自动清除相关缓存

// 缓存失效规则
Clean cache when:
  - create_employee(org_id, dept_id) → 清除该部门及所有上级部门的缓存
  - delete_employee(org_id, dept_id) → 清除该部门及所有上级部门的缓存
  - transfer_employee(from_dept, to_dept) → 清除两个部门及其上级的缓存
  - update_department(dept_id) → 清除该部门缓存
```

---

## 🔗 与其他模块的交互

### 1. Employee 模块
- 员工转部门时，更新员工部门统计缓存
- 员工晋升时，可能涉及跨部门转移

### 2. Position 模块
- 部门中不同岗位的员工分布

### 3. Identity 模块
- 权限检查：用户是否是该部门的管理者
- Casbin 权限规则：base_on_dept_id

### 4. Kafka 事件
```
department.created   → 发送到 ms-notify（通知部门负责人）
department.updated   → 发送到 ms-notify（仪表板更新）
department.deleted   → 发送到其他模块（清理关系）
employee.added       → 部门缓存失效
employee.removed     → 部门缓存失效
```

---

## ⚡ MVP 性能目标

| 场景 | 操作 | 目标 | 约束 |
|------|------|------|------|
| **一级部门查询** | GET /departments/roots | **< 100ms** | 缓存命中；部门 ≤ 200 |
| **一级部门查询** | GET /departments/roots | **< 400ms** | 缓存未命中；需计算员工数 |
| **子部门查询** | GET /departments/{id}/children | **< 200ms** | 缓存命中；直属 ≤ 50 |
| **子部门查询** | GET /departments/{id}/children | **< 500ms** | 缓存未命中；需 JOIN 计算 |
| **单个部门** | GET /departments/{id} | **< 50ms** | 不包含员工数统计 |
| **单个部门+统计** | GET /departments/{id}?include_count=true | **< 150ms** | 缓存命中时 |
| **整体 QPS** | 并发查询 | **≥ 1000 req/s** | 单机；缓存配合良好 |

---

## 📅 实施阶段

### Phase 1 (第 2 周) - 数据结构 + 统计逻辑
**目标**: 所有接口都能返回 total_employee_count

- [ ] 扩展 DepartmentResponse 添加 `total_employee_count` 字段
- [ ] Repository 层添加 `count_employees_by_dept_id()` 方法
  - 利用 path 索引快速查询下属员工数
  - 结果：department id → employee count
- [ ] Service 层集成统计逻辑
  - to_response_with_count() 方法
  - 支持批量员工数查询（避免 N+1）
- [ ] 数据库索引优化
- [ ] 所有现有接口更新为返回 total_employee_count

**验收**: cargo test 通过，性能 < 300ms

### Phase 2 (第 3 周) - 分层下钻接口
**目标**: 实现前端懒加载的两个新接口

- [ ] 实现 `get_roots(org_id)` Service 方法
- [ ] 实现 `get_children(parent_id, pagination)` Service 方法
- [ ] Handler 添加两个新接口
- [ ] 集成到 router.rs
- [ ] 前端集成测试

**验收**: 两个接口性能 < 200ms

### Phase 3 (第 4-5 周) - Redis 缓存
**目标**: 将重复查询的员工数缓存到 Redis

- [ ] Redis 连接集成
- [ ] 缓存键设计：`dept:{org_id}:{dept_id}:emp_count`
- [ ] Service 添加缓存层
  - 查询时先从 Cache 读
  - 缓存未命中才 DB 查询
  - 结果自动 SET 到缓存 (TTL 300s)
- [ ] 缓存失效机制
  - Employee 创建/删除时清除
  - 部门转移时清除
- [ ] 性能验证 (缓存命中 < 50ms)

**验收**: 缓存命中率 > 80%, 响应 < 100ms

---

## 📝 验收标准（MVP 核心验收）

### Phase 1 完成标准（第 2 周末）✅
- [ ] DepartmentResponse 添加 `total_employee_count` 字段
- [ ] Repository 层实现 `count_employees_in_path(path)` 方法
- [ ] Service 层实现员工数统计逻辑
- [ ] 可在 get_by_id() 时返回员工数
- [ ] 所有查询自动包含 total_employee_count
- [ ] 数据库索引优化完成
- [ ] 编译通过，cargo check 无警告

### Phase 2 完成标准（第 3 周末）✅
- [ ] 实现 `GET /api/departments/roots?org_id=X` 接口
  - [ ] 返回一级部门列表（10～200 个）
  - [ ] 每个部门包含 total_employee_count
  - [ ] 支持分页
  - [ ] 性能 < 100ms
- [ ] 实现 `GET /api/departments/{id}/children` 接口
  - [ ] 返回直属子部门列表（分页）
  - [ ] 每个部门包含 total_employee_count
  - [ ] 支持关键词搜索
  - [ ] 性能 < 200ms
- [ ] 编译通过，单元测试通过

### Phase 3 完成标准（第 4-5 周）✅
- [ ] Redis 缓存集成
  - [ ] 缓存 department:employee_count
  - [ ] TTL 300s
  - [ ] 命中率 > 80%
- [ ] 缓存失效触发：
  - [ ] 员工创建/删除时自动清除
  - [ ] 员工部门转移时自动清除
- [ ] 性能达到 < 50ms（缓存命中）

---

## 🎯 优先级排序

| 优先级 | 实现时间 | 功能 | 状态 |
|--------|--------|------|------|
| **P0** | 第2周 | 统计一个部门的所有下属员工数 | ⏳ |
| **P0** | 第2周 | DepartmentResponse 添加 total_employee_count | ⏳ |
| **P0** | 第3周 | /departments/roots 接口 | ⏳ |
| **P0** | 第3周 | /departments/{id}/children 接口 | ⏳ |
| **P1** | 第4周 | Redis 缓存集成 | ⏳ |
| **P2** | 第5周+ | 其他高级功能 | 暂不实施 |

---

## 📚 相关文档

- [DEVELOPMENT_TASKS.md](DEVELOPMENT_TASKS.md) - 总体开发计划
- [TASK_1_3_VALIDATION_MATRIX.md](TASK_1_3_VALIDATION_MATRIX.md) - 验证矩阵
- [ENTERPRISE_READINESS_ANALYSIS.md](../ms-team/docs/ENTERPRISE_READINESS_ANALYSIS.md) - 企业级分析

