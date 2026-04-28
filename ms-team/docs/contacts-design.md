# ms-team 通讯录功能设计文档

> 版本：v3.0 | 更新时间：2026-04-28

## 一、功能定位

通讯录是企业应用的核心高频功能，参考**钉钉/飞书**通讯录交互模式设计。

### 交互模型

```
┌──────────────────────────────────────────┐
│  组织选择器（多组织场景下切换）              │
├──────────────────────────────────────────┤
│  🔍 搜索框（全局搜索，走 Meilisearch）       │
├──────────────────────────────────────────┤
│  📁 技术部 (32人)                     ▶    │
│    👤 张三（部门负责人）                     │
│    👤 李四                                 │
│    👤 王五                                 │
│    📁 前端组 (12人)                    ▶    │
│    📁 后端组 (15人)                    ▶    │
│    📁 测试组 (5人)                     ▶    │
├──────────────────────────────────────────┤
│  📁 产品部 (8人)                      ▶    │
│  📁 运营部 (6人)                      ▶    │
└──────────────────────────────────────────┘
```

**核心交互流程**：

1. 进入通讯录 → 看到组织下的根部门列表（含可见人数）
2. 点击部门 → 懒加载展开子部门 + 前几名直属成员预览（负责人置顶）
3. 点击"查看全部成员" → 走独立分页接口
4. 点击人名 → 打开联系人详情卡片
5. 顶部搜索 → Meilisearch 匹配 ID → 回源 MySQL 获取最新数据

---

## 二、业务规则约束

### 上限配置

通过 `config.rs` 管理，支持环境变量覆盖。

| 约束项 | 配置键 | 默认值 | 说明 |
|--------|--------|--------|------|
| 子部门数上限 | `max_sub_departments` | **8** | 单个部门下最多可创建的子部门数 |
| 部门成员上限 | `max_dept_members` | **200** | 单个部门最多可挂靠的直属成员数 |
| 部门树深度上限 | `max_dept_depth` | **8** | 部门树最大层级深度（集团→事业群→事业部→中心→部门→小组→小队→个人项目组） |
| 根部门数上限 | `max_org_root_depts` | **20** | 单个组织下最多根部门数 |
| 部门展开预览成员数 | `dept_preview_members` | **20** | 部门展开时默认加载的直属成员数 |
| 含子部门查询成员上限 | `include_children_max` | **2000** | `include_children=true` 时最多返回的成员数，防止全组织查询 |

### 校验时机与并发安全

```
创建部门时 → 检查父部门子部门数 ≤ max_sub_departments
            → 检查层级深度 ≤ max_dept_depth
            → 如果是根部门，检查组织根部门数 ≤ max_org_root_depts

添加员工到部门时 → 检查部门直属成员数 ≤ max_dept_members
```

**并发安全策略**：

上限校验存在 TOCTOU 竞态风险（两个请求同时检查通过后都写入）。采用 **数据库行锁** 方案：

```sql
-- 创建部门时，锁定父部门行，防止并发超限
SELECT id FROM department WHERE id = ? FOR UPDATE;
-- 获得锁后再 COUNT 子部门数并校验
SELECT COUNT(*) FROM department WHERE parent_id = ? AND is_deleted = 0;
-- 校验通过后 INSERT，事务提交释放锁
```

```sql
-- 添加员工到部门时，锁定部门行
SELECT id FROM department WHERE id = ? FOR UPDATE;
SELECT COUNT(*) FROM employee_department WHERE department_id = ?;
-- 校验通过后 INSERT
```

> 选择数据库行锁而非 Redis 分布式锁的理由：创建部门和添加成员本身就在事务中操作数据库，行锁的一致性保证最强，且无需额外引入分布式锁的超时与续租复杂度。

---

## 三、搜索引擎集成（Meilisearch）

### 3.1 索引设计

**索引名称**：`employees`

**文档结构**（"胖文档"设计，包含展示所需的全部字段，直接作为搜索响应的数据源）：

```json
{
  "id": 1,
  "tenant_id": 1,
  "org_id": 1,
  "employee_no": "EMP001",
  "name": "张三",
  "avatar": "https://...",
  "mobile": "13812345678",
  "email": "zhangsan@xilulu.com",
  "department_title": "技术总监",
  "primary_department_name": "技术部",
  "status": 1
}
```

### 3.2 索引属性配置

| 配置项 | 值 |
|--------|-----|
| **主键** | `id` |
| **可搜索字段** | `name`, `mobile`, `email`, `employee_no`, `department_title` |
| **可过滤字段** | `tenant_id`, `org_id`, `status` |
| **可排序字段** | `name`, `employee_no` |

### 3.3 搜索流程：短时最终一致的胖文档直出

搜索接口的数据流程不再回源 MySQL，充分释放数据库压力：

```
客户端请求: keyword="张三", page_size=20
    ↓
Step 1: Meilisearch 全文检索
        → 过采样请求 page_size * 1.5 = 30 条
        → 返回完整的 SearchDocument 列表
    ↓
Step 2: 内存权限过滤 → filter_visible_employees() + get_batch_field_restrictions()
        → 在内存中脱敏/隐藏敏感字段，移除不可见员工
        → 过滤后截取前 page_size 条
        → 若不足 page_size 且搜索引擎还有更多结果，透明请求下一批
    ↓
Step 3: 组装最终响应直接返回
```

**坚持不回源的考量与 Trade-off**：

1. **极致性能与高可用**：发挥搜索引擎的完全优势，搜索相关的高发流量全部被隔离在 DB 之外，不影响主库事务。
2. **短时最终一致性**：关联信息（如部门改名）有秒级延迟，但在通讯录场景中完全可接受。

**SearchPort trait 设计**：

```rust
#[async_trait]
pub trait EmployeeSearchPort: Send + Sync {
    /// 搜索 → 返回匹配的完整文档列表 + 估算总数
    async fn search(&self, criteria: SearchCriteria) -> anyhow::Result<SearchDocumentResult>;

    /// 索引/更新文档
    async fn index(&self, doc: SearchDocument) -> anyhow::Result<()>;

    /// 批量索引
    async fn batch_index(&self, docs: Vec<SearchDocument>) -> anyhow::Result<()>;

    /// 删除文档
    async fn delete(&self, id: i64) -> anyhow::Result<()>;
}

/// 搜索条件
pub struct SearchCriteria {
    pub keyword: String,
    pub org_id: i64,
    pub tenant_id: i64,
    /// 偏移量（offset 分页）
    pub offset: u32,
    /// 请求数量（含过采样）
    pub limit: u32,
}

/// 搜索结果 — 包含完整显示业务数据
pub struct SearchDocumentResult {
    /// 匹配的员工文档列表
    pub items: Vec<SearchDocument>,
    /// 估算命中总数
    pub estimated_total: u64,
}
```

### 3.4 索引同步策略

**写时同步** + **全量重建兜底**：

| 触发时机 | 操作 | 说明 |
|---------|------|------|
| 创建员工 | `index` | 员工创建成功后组装搜索文档并写入 |
| 更新员工基本信息 | `index` | 覆盖写（以 id 为主键，天然幂等）|
| 删除员工 | `delete` | 从索引中移除 |
| 员工个人部门/岗位变更 | `index` | 重新索引该员工文档 |
| **部门基础信息变更（重命名）** | `batch_index` | **牵一发而动全身**：需批量更新该部门下所有成员的搜索文档 |
| **管理员触发** | `rebuild_all` | 全量重建索引（补偿机制）|

**同步失败处理**：

1. 同步失败**不阻塞**主流程，仅记录 `warn` 日志
2. 提供管理员接口全量重建（见 4.6 节）
3. 未来可接入 Kafka/Redis Stream 解耦写操作与索引更新

### 3.5 搜索引擎降级策略

当 Meilisearch 不可用时，搜索接口**自动降级到 MySQL** `LIKE` 查询：

```rust
// MeilisearchAdapter::search 内部
match self.client.index(INDEX_NAME).search(...).await {
    Ok(results) => Ok(/* 正常结果 */),
    Err(e) => {
        tracing::warn!("Meilisearch 不可用，降级到 MySQL 搜索: {}", e);
        // 返回 Err，由 ContactsService 捕获后走 MySQL fallback
        Err(e.into())
    }
}
```

`ContactsService` 中的降级编排：

```rust
let search_result = match self.search_port.search(criteria).await {
    Ok(result) => result,
    Err(_) => {
        // 降级到 MySQL LIKE 查询
        return self.search_by_mysql_fallback(org_id, keyword, page, page_size).await;
    }
};
```

降级查询性能较差，但保证服务可用。响应中可标记 `"degraded": true` 提示客户端。

### 3.6 架构模式（Port-Adapter）

```
src/modules/contacts/search/
├── port.rs          # EmployeeSearchPort trait（返回 ID 列表）
└── adapter.rs       # MeilisearchAdapter（SDK 实现）
```

业务层通过 `EmployeeSearchPort` trait 调用搜索，仅获取 ID 列表，然后回源 MySQL。
如未来切换搜索引擎（如 Elasticsearch），仅需新增 Adapter 实现。

---

## 四、API 设计

### 路由总表

#### 通讯录接口：`/api/v1/team/contacts`

| # | 方法 | 路径 | 数据源 | 说明 |
|---|------|------|--------|------|
| 1 | GET | `/contacts/entry?org_id=1` | MySQL | 通讯录首屏数据 |
| 2 | GET | `/contacts/departments/{dept_id}` | MySQL | 部门展开（子部门 + 成员预览） |
| 3 | GET | `/contacts/employees/{id}` | MySQL | 联系人详情卡片 |
| 4 | GET | `/contacts/search?org_id=1&keyword=xxx` | Meilisearch | 全局搜索（胖文档直出） |
| 5 | GET | `/contacts/departments/{dept_id}/members` | MySQL | 部门成员分页 |

#### 管理接口：`/api/v1/team/admin`

| # | 方法 | 路径 | 说明 |
|---|------|------|------|
| 6 | POST | `/admin/search/rebuild` | 全量重建搜索索引（管理员权限） |

**数据源策略**：
- **搜索** → Meilisearch 胖文档直出（降低大流量下的 DB 压力），不可用时降级到 MySQL
- **浏览/详情** → MySQL 直查（强一致，不依赖搜索引擎可用性）

### 4.1 通讯录入口

```
GET /api/v1/team/contacts/entry?org_id=1
```

**参数**：

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `org_id` | i64 | 是 | 组织 ID |

**响应**：

```json
{
  "success": true,
  "data": {
    "organization": {
      "id": 1,
      "name": "希璐璐科技",
      "logo": "https://..."
    },
    "departments": [
      {
        "id": 10,
        "name": "技术部",
        "has_children": true,
        "member_count": 32,
        "leader": {
          "id": 1,
          "name": "张三",
          "avatar": "https://..."
        }
      },
      {
        "id": 20,
        "name": "产品部",
        "has_children": false,
        "member_count": 8,
        "leader": null
      }
    ],
    "total_member_count": 128
  }
}
```

> `member_count` 始终为**当前用户可见的人数**。Phase 1 中等于实际人数；Phase 2+ 中通过权限引擎过滤后计算。`leader` 来源于该部门 `employee_department.is_leader=1` 的成员（见 4.7 约定）。

### 4.2 部门展开

```
GET /api/v1/team/contacts/departments/{dept_id}
```

**响应**：返回子部门列表 + **前 N 名**直属成员预览（负责人置顶，N = `dept_preview_members` 默认 20）

```json
{
  "success": true,
  "data": {
    "department": {
      "id": 10,
      "name": "技术部",
      "full_name": "希璐璐科技/技术部"
    },
    "children": [
      {
        "id": 11,
        "name": "前端组",
        "has_children": false,
        "member_count": 12,
        "leader": { "id": 3, "name": "王五", "avatar": "..." }
      }
    ],
    "members": [
      {
        "id": 1,
        "name": "张三",
        "avatar": "https://...",
        "department_title": "技术总监",
        "mobile": "13812345678",
        "email": "zhangsan@xilulu.com",
        "is_leader": true
      },
      {
        "id": 2,
        "name": "李四",
        "avatar": "https://...",
        "department_title": null,
        "mobile": "13900001234",
        "email": "lisi@xilulu.com",
        "is_leader": false
      }
    ],
    "direct_member_count": 18,
    "has_more_members": true
  }
}
```

> `members` 仅返回预览数量（默认 20 人），负责人始终置顶。`direct_member_count` 为该部门的**直属可见成员总数**（不含子部门），`has_more_members` 为 true 时前端显示"查看全部"按钮，引导跳转 4.5 分页接口。

### 4.3 联系人详情

```
GET /api/v1/team/contacts/employees/{id}
```

**响应**：完整个人信息 + 所有部门关系 + 所有岗位关系

```json
{
  "success": true,
  "data": {
    "id": 1,
    "name": "张三",
    "avatar": "https://...",
    "employee_no": "EMP001",
    "gender": 1,
    "mobile": "13812345678",
    "email": "zhangsan@xilulu.com",
    "status": 1,
    "hire_date": 1609459200000,
    "departments": [
      {
        "id": 10,
        "name": "技术部",
        "full_name": "希璐璐科技/技术部",
        "is_primary": true,
        "is_leader": true
      }
    ],
    "positions": [
      {
        "id": 5,
        "name": "CTO",
        "level": 10,
        "is_primary": true
      }
    ]
  }
}
```

> 字段值受 Layer 3 权限过滤影响。如 `mobile` 可能返回 `"138****5678"`（脱敏）或 `null`（隐藏）。若 `is_employee_visible` 返回 false，整个接口返回 404。

### 4.4 全局搜索

```
GET /api/v1/team/contacts/search?org_id=1&keyword=张&page=1&page_size=20
```

**参数**：

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `org_id` | i64 | 是 | 限定搜索范围 |
| `keyword` | String | 是 | 搜索关键词（姓名/手机/工号/邮箱） |
| `page` | i64 | 否 | 页码，默认 1 |
| `page_size` | i64 | 否 | 每页条数，默认 20，最大 50 |

**响应**：

```json
{
  "success": true,
  "data": {
    "items": [
      {
        "id": 1,
        "name": "张三",
        "avatar": "https://...",
        "employee_no": "EMP001",
        "mobile": "13812345678",
        "email": "zhangsan@xilulu.com",
        "department_title": "技术总监",
        "primary_department_name": "技术部"
      }
    ],
    "estimated_total": 3,
    "has_next": false,
    "degraded": false
  }
}
```

> - `estimated_total` 为 Meilisearch 的 `estimatedTotalHits`（估算值，非精确计数）
> - `items` 中的基础数据直接来自 Meilisearch 胖文档，且受请求时实时的 Layer 2/3 内存权限过滤影响
> - `degraded` 标记是否降级到了 MySQL 搜索（Meilisearch 不可用时）

### 4.5 部门成员分页

```
GET /api/v1/team/contacts/departments/{dept_id}/members?page=1&page_size=20&include_children=false
```

**参数**：

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `include_children` | bool | 否 | 是否包含子部门成员，默认 false |
| `page` | i64 | 否 | 页码，默认 1 |
| `page_size` | i64 | 否 | 每页条数，默认 20 |

**响应**：与搜索结果结构一致。负责人始终排在首页第一位。

**`include_children=true` 的保护机制**：

当包含子部门成员时，利用 Materialized Path 的 `LIKE` 查询所有子树成员。为防止根部门查询全组织导致慢查询，增加上限保护：
- 先 COUNT 总数，若超过 `include_children_max`（默认 2000），返回错误提示"成员过多，请按子部门分别查看"
- 若未超过，正常分页返回

### 4.6 全量重建索引（管理接口）

```
POST /api/v1/team/admin/search/rebuild?org_id=1
```

**参数**：

| 参数 | 类型 | 必填 | 说明 |
|------|------|------|------|
| `org_id` | i64 | 否 | 限定重建范围，不传则重建全部 |

**返回**：`202 Accepted`

**实现逻辑**：

```rust
// 分批重建，避免 OOM
let batch_size = 500;
let mut offset = 0;
loop {
    let employees = EmployeeRepo::find_page_for_index(pool, org_id, offset, batch_size).await?;
    if employees.is_empty() { break; }
    let docs = employees.iter().map(|e| build_search_doc(e)).collect();
    search_port.batch_index(docs).await?;
    offset += batch_size;
}
```

> 此接口需要管理员权限校验。大数据量场景下异步执行，返回 202 后后台处理。

### 4.7 关键约定：leader 的来源

部门的 leader 统一从 `employee_department.is_leader = 1` 获取，而非 `department.leader_employee_id`。

**理由**：

- `employee_department` 表中 `is_leader` 字段天然保证了 leader 必须是该部门的直属成员
- 避免出现 `department.leader_employee_id` 指向一个不在该部门的人的不一致情况
- `department.leader_employee_id` 仅作为冗余快查字段，由创建/变更 leader 时同步维护

**查询方式**：

```sql
-- 获取部门 leader
SELECT e.* FROM employee e
INNER JOIN employee_department ed ON e.id = ed.employee_id
WHERE ed.department_id = ? AND ed.is_leader = 1
LIMIT 1;
```

---

## 五、权限体系设计

通讯录权限是企业级应用的核心安全需求。采用**三层可见性模型 + trait 引擎**的架构，Phase 1 实现默认全开放策略，通过 trait 接口预留完整的权限扩展能力。

### 5.1 三层可见性模型

```
┌─────────────────────────────────────────────────────┐
│                   请求进入                            │
│                     ↓                                │
│  ┌──────────────────────────────────────────────┐   │
│  │  Layer 1: 部门可见性 (Department Visibility)   │   │
│  │  哪些部门对当前用户可见？                         │   │
│  │  → 过滤通讯录树中的部门节点                       │   │
│  │  → 影响：入口、部门展开、搜索结果中的部门归属       │   │
│  └──────────────────────────────────────────────┘   │
│                     ↓                                │
│  ┌──────────────────────────────────────────────┐   │
│  │  Layer 2: 人员可见性 (Contact Visibility)      │   │
│  │  哪些人对当前用户可见？                           │   │
│  │  → 隐藏的人从成员列表和搜索结果中消失              │   │
│  │  → 隐藏的人不计入 member_count                   │   │
│  └──────────────────────────────────────────────┘   │
│                     ↓                                │
│  ┌──────────────────────────────────────────────┐   │
│  │  Layer 3: 字段可见性 (Field Visibility)        │   │
│  │  可见的人的哪些字段对当前用户可见？                 │   │
│  │  → 手机号、邮箱等敏感字段可按关系脱敏或隐藏         │   │
│  └──────────────────────────────────────────────┘   │
│                     ↓                                │
│                 返回响应                              │
└─────────────────────────────────────────────────────┘
```

### 5.2 权限引擎 trait 设计

```rust
/// 通讯录权限引擎
/// 所有通讯录 API 在查询原始数据后、组装响应前，经过此引擎进行可见性过滤
#[async_trait]
pub trait ContactsPermission: Send + Sync {

    // ==================== Layer 1: 部门可见性 ====================

    /// 批量过滤部门列表，移除当前用户不可见的部门
    async fn filter_visible_departments(
        &self,
        viewer: &ContactsViewer,
        department_ids: &[i64],
    ) -> HashSet<i64>;

    /// 判断单个部门是否对当前用户可见（用于详情校验）
    async fn is_department_visible(
        &self,
        viewer: &ContactsViewer,
        department_id: i64,
    ) -> bool;

    // ==================== Layer 2: 人员可见性 ====================

    /// 批量过滤员工列表，移除当前用户不可见的员工
    async fn filter_visible_employees(
        &self,
        viewer: &ContactsViewer,
        employee_ids: &[i64],
    ) -> HashSet<i64>;

    /// 判断单个员工是否对当前用户可见（用于联系人详情校验）
    async fn is_employee_visible(
        &self,
        viewer: &ContactsViewer,
        target_employee_id: i64,
    ) -> bool;

    // ==================== Layer 3: 字段可见性 ====================

    /// 返回当前用户查看目标员工时的字段限制（单条）
    async fn get_field_restrictions(
        &self,
        viewer: &ContactsViewer,
        target_employee_id: i64,
    ) -> FieldRestrictions;

    /// 批量获取字段限制（用于列表场景，避免 N 次调用）
    async fn get_batch_field_restrictions(
        &self,
        viewer: &ContactsViewer,
        target_employee_ids: &[i64],
    ) -> HashMap<i64, FieldRestrictions>;

    // ==================== 计数 ====================

    /// 计算某部门对当前用户可见的直属成员数
    /// Phase 1：直接返回 actual_count
    /// Phase 2+：从快照中的 hidden_count_by_dept 计算
    async fn count_visible_members(
        &self,
        viewer: &ContactsViewer,
        department_id: i64,
        actual_count: i64,
    ) -> i64;
}

/// 当前查看者身份信息（从请求上下文中构建）
pub struct ContactsViewer {
    pub user_id: i64,
    pub tenant_id: i64,
    pub org_id: i64,
    pub employee_id: Option<i64>,
    /// 查看者所属部门 ID 列表
    /// Phase 1 中为空（DefaultPermission 不使用）
    /// Phase 2+ 中从缓存加载（不在每次请求时查 DB）
    pub department_ids: Vec<i64>,
    pub is_admin: bool,
}

/// 字段限制策略
pub struct FieldRestrictions {
    pub mobile: FieldAction,
    pub email: FieldAction,
    pub hire_date: FieldAction,
    pub employee_no: FieldAction,
}

pub enum FieldAction {
    Visible,   // 完整展示
    Masked,    // 脱敏（如 138****8888）
    Hidden,    // 完全隐藏（返回 null）
}
```

### 5.3 Phase 1：默认全开放策略

```rust
/// 默认权限实现 — 所有内容可见、所有字段可见
pub struct DefaultPermission;

#[async_trait]
impl ContactsPermission for DefaultPermission {
    async fn filter_visible_departments(&self, _, ids: &[i64]) -> HashSet<i64> {
        ids.iter().cloned().collect()
    }

    async fn is_department_visible(&self, _, _) -> bool { true }

    async fn filter_visible_employees(&self, _, ids: &[i64]) -> HashSet<i64> {
        ids.iter().cloned().collect()
    }

    async fn is_employee_visible(&self, _, _) -> bool { true }

    async fn get_field_restrictions(&self, _, _) -> FieldRestrictions {
        FieldRestrictions::all_visible()
    }

    async fn get_batch_field_restrictions(&self, _, ids: &[i64]) -> HashMap<i64, FieldRestrictions> {
        ids.iter().map(|id| (*id, FieldRestrictions::all_visible())).collect()
    }

    async fn count_visible_members(&self, _, _, actual_count: i64) -> i64 {
        actual_count
    }
}
```

> 使用 `Arc<dyn ContactsPermission>` 动态分发有纳秒级虚表查找开销，可忽略不计。Phase 1 中 `ContactsViewer.department_ids` 为空向量，不查 DB。

### 5.4 Phase 2+：规则引擎扩展方向

未来通过 `RuleBasedPermission` 实现精细化管控，仅需切换 trait 实现：

#### 方向 A：部门级隔离

```
规则示例：
- "法务部" 仅对 "管理层" 部门成员可见
- "审计部" 仅对 CEO 可见

实现方式：
department_visibility_rule 表
  dept_id     — 被限制的部门
  scope_type  — WHITELIST / BLACKLIST
  scope_ids   — 允许/禁止看到此部门的部门/岗位/人员 ID
```

#### 方向 B：高管保护

```
规则示例：
- VP 以上级别人员，普通员工看不到手机号
- CEO 不出现在普通员工的通讯录中

实现方式：
contact_visibility_rule 表
  target_employee_id  — 被保护的人
  rule_type           — HIDE_FROM_ALL / HIDE_FIELD / VISIBLE_TO_WHITELIST
  viewer_scope        — 哪些人受此规则影响
  field_restrictions  — JSON，指定字段级别的展示策略
```

#### 方向 C：字段级脱敏

```
规则示例：
- 非同部门员工看手机号显示为 138****8888
- 仅部门负责人可以看下属完整邮箱

实现方式：
field_visibility_rule 表
  field_name       — mobile / email / hire_date / employee_no
  action           — VISIBLE / MASKED / HIDDEN
  relationship     — SAME_DEPT / SAME_ORG / ALL / ADMIN_ONLY
  priority         — 优先级（多规则冲突时取高优先级）
```

### 5.5 权限的 API 集成点

权限过滤发生在 **Service 层查询原始数据之后、Handler 组装响应之前**：

```
Handler 接收请求
    ↓
构建 ContactsViewer（从请求头提取用户身份）
    ↓
ContactsService 查询原始数据（MySQL 或 Meilisearch胖文档直接反序列化）
    ↓
ContactsPermission.filter_visible_departments()  ← Layer 1
    ↓
ContactsPermission.filter_visible_employees()    ← Layer 2
    ↓
ContactsPermission.get_batch_field_restrictions() ← Layer 3
    ↓
组装最终响应（根据 FieldAction 脱敏/隐藏字段，count_visible_members 计数）
    ↓
Handler 返回 R<T>
```

**每个 API 的权限过滤行为**：

| API | Layer 1 (部门) | Layer 2 (人员) | Layer 3 (字段) | 计数修正 |
|-----|---------------|---------------|---------------|---------|
| 通讯录入口 | ✅ 过滤根部门 | ✅ leader 可见性 | ❌ | ✅ member_count |
| 部门展开 | ✅ 过滤子部门 | ✅ 过滤成员预览 | ✅ 批量字段 | ✅ direct_member_count |
| 联系人详情 | ❌ | ✅ 目标人可见性 | ✅ 单条字段 | ❌ |
| 全局搜索 | ❌ | ✅ 过滤搜索结果 | ✅ 批量字段 | ❌ |
| 部门成员分页 | ✅ 部门可见性 | ✅ 过滤成员列表 | ✅ 批量字段 | ✅ total |

### 5.6 性能保障策略

三层过滤的性能是核心约束。设计原则：**Phase 1 可忽略开销，Phase 2+ 单请求内最多 1 次 Redis 查询**。

#### Phase 1：近乎零开销

`DefaultPermission` 所有方法为纯内存操作，无 I/O。`ContactsViewer.department_ids` 为空，不查 DB。

#### Phase 2+：每请求一次权限快照

核心思路：**请求入口一次性构建权限快照，后续所有过滤在内存中 O(1) 完成**。

```
请求进入
    ↓
┌──────────────────────────────────────────────────┐
│  PermissionSnapshot::build(viewer)                │
│  → 1 次 Redis GET（或 miss 时 1 次 DB 查询）      │
│  → 构建：                                         │
│     hidden_dept_ids:       HashSet<i64>           │
│     hidden_emp_ids:        HashSet<i64>           │
│     hidden_count_by_dept:  HashMap<i64, i64>      │
│     field_rules:           Vec<FieldRule>          │
│     viewer_dept_ids:       Vec<i64>               │
│  → 整个请求生命周期内复用                            │
└──────────────────────────────────────────────────┘
    ↓
Layer 1: hidden_dept_ids.contains(&id)             → O(1)
Layer 2: hidden_emp_ids.contains(&id)              → O(1)
Layer 3: field_rules.match(viewer, target)         → O(n)，n < 5
count:   actual - hidden_count_by_dept.get(&dept)  → O(1)
```

> 快照中 `hidden_count_by_dept` 预计算了每个部门中被隐藏的人数，`count_visible_members` 直接 `actual_count - hidden_count` 即可，无需传入完整员工 ID 列表。
> 快照中 `viewer_dept_ids` 包含查看者的部门归属，避免每次请求查 DB。

#### 缓存分层

```
┌───────────────────┐
│   请求级缓存       │  PermissionSnapshot 绑定到请求生命周期
│   生命周期：单请求   │
└────────┬──────────┘
         ↓
┌───────────────────┐
│   Redis 缓存       │  permission:snapshot:{tenant}:{viewer}
│   TTL：60s         │  主动失效 + TTL 兜底
└────────┬──────────┘
         ↓
┌───────────────────┐
│   MySQL           │  visibility_rule 表
│   仅 cache miss   │
└───────────────────┘
```

#### 缓存主动失效策略

被动 TTL 在高频变动场景下可能导致数据滞后。因此在组织架构变更时**主动失效**：

| 变更事件 | 失效范围 | 失效方式 |
|---------|---------|---------|
| 员工部门调动 | 该员工的 viewer 快照 | `DEL permission:snapshot:{tenant}:{employee_id}` |
| 部门权限规则变更 | 该部门所有成员的快照 | `SCAN + DEL` 或 Redis Pub/Sub 通知 |
| 全局规则变更 | 整个租户的快照 | `DEL permission:snapshot:{tenant}:*` |

> 这些都是**低频管理操作**，批量失效的成本可接受。

#### 批量优先的 trait 设计

所有 filter 方法接收 `&[i64]` 切片，从接口层面杜绝 N+1 查询。`get_batch_field_restrictions` 用于列表场景，`get_field_restrictions` 用于单条详情。`is_*_visible` 方法用于单条记录校验，内部同样走快照 `HashSet::contains`，O(1) 开销。

#### 性能指标预期

| 场景 | Phase 1 | Phase 2+（有规则） |
|------|---------|-------------------|
| 通讯录入口（20 根部门） | < 1ms | + 1ms（Redis GET + HashSet） |
| 部门展开（8 子部门 + 20 预览成员） | < 1ms | + 1ms |
| 搜索（Meili 匹配 + MySQL 回源 20 条） | < 5ms | + 1ms |
| 联系人详情 | < 1ms | + 1ms |

---

## 六、部门树的 Materialized Path 查询策略

数据库中 `department.path` 字段已存储物化路径（如 `/1/5/12/`），充分利用此字段优化所有树操作：

### 核心查询模式

```sql
-- 获取某部门的所有子孙部门（利用 path 前缀匹配 + idx_path 索引）
SELECT * FROM department
WHERE path LIKE '/1/5/%' AND is_deleted = 0
ORDER BY sort_order, id;

-- 获取某部门的直接子部门
SELECT * FROM department
WHERE parent_id = ? AND is_deleted = 0
ORDER BY sort_order, id;

-- 统计某部门及所有子孙部门的员工总数
SELECT COUNT(DISTINCT ed.employee_id)
FROM employee_department ed
INNER JOIN department d ON ed.department_id = d.id
WHERE d.path LIKE '/1/5/%' AND d.is_deleted = 0;

-- 判断某部门是否有子部门
SELECT EXISTS(SELECT 1 FROM department WHERE parent_id = ? AND is_deleted = 0);
```

### 通讯录各 API 的树查询方式

| API | 查询方式 |
|-----|---------|
| 通讯录入口 | `WHERE org_id = ? AND parent_id IS NULL` — 根部门列表 |
| 部门展开 | `WHERE parent_id = ?` — 直接子部门 |
| 部门成员（含子部门） | `WHERE d.path LIKE '/dept_path/%'` — Materialized Path 子树查询 |
| 员工数统计（含子部门） | `JOIN department d ON ed.department_id = d.id WHERE d.path LIKE ?` |

---

## 七、技术架构

### 模块结构

```
src/modules/contacts/
├── mod.rs                    # 模块声明与 re-export
├── handler.rs                # 5 个通讯录 Handler + 1 个管理 Handler
├── service.rs                # ContactsService（聚合查询 + 权限过滤编排 + 降级逻辑）
├── model/
│   ├── mod.rs
│   └── dto.rs                # 通讯录专用响应 DTO
├── permission/
│   ├── mod.rs
│   ├── port.rs               # ContactsPermission trait + 类型定义
│   └── default.rs            # DefaultPermission 实现（Phase 1）
└── search/
    ├── mod.rs
    ├── port.rs               # EmployeeSearchPort trait（返回 ID 列表）
    └── adapter.rs            # MeilisearchAdapter 实现
```

### 依赖关系

```
ContactsService
├── OrganizationService (Arc)      — 组织信息
├── DepartmentService (Arc)        — 部门查询 + Materialized Path
├── EmployeeService (Arc)          — 员工查询 + 索引同步
├── EmployeeDepartmentService      — 部门成员关系（leader 查询）
├── EmployeePositionService        — 岗位关系
├── EmployeeSearchPort (Arc<dyn>)  — 搜索引擎（仅返回 ID）
└── ContactsPermission (Arc<dyn>)  — 权限引擎
```

### 文件变更清单

**新增文件**：

| 文件 | 说明 |
|------|------|
| `src/modules/contacts/mod.rs` | 模块声明 |
| `src/modules/contacts/handler.rs` | 5 个通讯录接口 + 1 个管理接口 Handler |
| `src/modules/contacts/service.rs` | 通讯录聚合查询服务 + 降级逻辑 |
| `src/modules/contacts/model/mod.rs` | DTO 模块声明 |
| `src/modules/contacts/model/dto.rs` | 通讯录专用 DTO |
| `src/modules/contacts/permission/mod.rs` | 权限模块声明 |
| `src/modules/contacts/permission/port.rs` | ContactsPermission trait（含批量方法） |
| `src/modules/contacts/permission/default.rs` | 默认全开放权限实现 |
| `src/modules/contacts/search/mod.rs` | 搜索模块声明 |
| `src/modules/contacts/search/port.rs` | 搜索接口 trait |
| `src/modules/contacts/search/adapter.rs` | Meilisearch 实现 + 降级 |

**修改文件**：

| 文件 | 变更 |
|------|------|
| `Cargo.toml` | 新增 `meilisearch-sdk = "0.28"` |
| `src/modules/mod.rs` | 新增 `pub mod contacts;` |
| `src/router.rs` | 新增 `/contacts` + `/admin` 路由组 |
| `src/state.rs` | 新增 `contacts_service` 字段 |
| `src/config.rs` | 新增 Meilisearch 配置 + 业务限制配置项（含 `include_children_max`） |
| `src/error.rs` | 修正 `DepartmentLevelTooDeep` 消息 + 新增上限校验错误码 |
| `src/modules/department/service.rs` | 创建部门时增加子部门数 / 深度校验 + 行锁 |
| `src/modules/department/repository.rs` | 修正 `count_employees_by_dept_id` 中 `dept_id` → `department_id` |
| `src/modules/employee/service.rs` | 员工 CRUD 时触发搜索索引同步 + 添加成员行锁 |
