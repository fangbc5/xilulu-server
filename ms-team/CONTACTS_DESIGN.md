# 企业通讯录设计方案

## 1. 当前表结构分析

### 组织表（Organization）
```
id: 1, name: "集团", parent_id: null
  ├── id: 2, name: "公司A", parent_id: 1
  │   ├── id: 3, name: "分公司A1", parent_id: 2
  └── id: 4, name: "公司B", parent_id: 1
```
- 树形结构，用 parent_id 表示上下级
- 类型：集团/公司/分公司/子公司

### 部门表（Department）
```
org_id: 2 (属于公司A)
  id: 100, name: "技术部", parent_id: null
    ├── id: 101, name: "后端组", parent_id: 100
    └── id: 102, name: "前端组", parent_id: 100
  id: 103, name: "销售部", parent_id: null

org_id: 4 (属于公司B)
  id: 200, name: "运营部", parent_id: null
```
- **关键点**：每个部门有 `org_id`，确定属于哪个组织
- 树形结构，用 parent_id 表示上下级

### 员工表（Employee）
```
id: 1000, name: "张三", org_id: 2 (公司A)
id: 1001, name: "李四", org_id: 2 (公司A)
id: 1002, name: "王五", org_id: 4 (公司B)
```
- 每个员工有 `org_id`，但**只能属于一个组织**

### 员工-部门关系（EmployeeDepartment）
```
employee_id: 1000 (张三)
  ├── department_id: 101 (后端组), is_primary: 1 ✓
  └── department_id: 100 (技术部), is_primary: 0

employee_id: 1001 (李四)
  └── department_id: 103 (销售部), is_primary: 1
```
- **关键点**：员工可以关联多个部门（多对多）
- 一个标记为 is_primary 的主部门

## 2. 问题诊断

### ⚠️ 问题1：两层树的冗余性
**情况**：
- Organization 是树形（可多级：集团 > 公司 > 分公司）
- Department 是树形（可多级：部门 > 子部门）
- Department 通过 org_id 关联到 Organization

**影响**：
```
如果写这样的查询：
GET /contacts/tree  → 组织树 > 部门树 > 员工
                    → 4层深度，数据结构复杂
```

### ⚠️ 问题2：Employee.org_id 的含义不清
**情况**：
- 员工 org_id = 2（公司A）
- 但如果组织表是：集团(1) > 公司A(2) > 分公司A1(3)
- 员工是属于"公司A"还是"分公司A1"？

**结果**：
- 当前设计下，员工只能属于某一个"端点"组织
- 不能属于中间层组织
- 这限制了组织树的灵活性

### ⚠️ 问题3：跨级查询困难
```
如果要查"集团所有员工"？
  需要：找集团 > 找所有子公司 > 找所有部门 > 找所有员工
  性能差，逻辑复杂
```

## 3. 推荐的通讯录设计方案

### 方案A：传统企业通讯录（推荐用于单企业产品）

#### 表结构建议
```
修改目标：简化 Organization，让 Department 成为主体

Department (单个企业内的完整树)
├── org_id → 标识属于哪个企业
├── parent_id → NULL（根部门）或其他部门ID
└── 员工关系

Organization (扁平化，不再作为树)
├── parent_id 可忽略或只表示法人关系
├── 对应一个"根部门"（自动创建）
```

#### 查询路径
```
GET /contacts/by-org/{org_id}
  返回：{ org, departments_tree, employees }
  
GET /contacts/by-dept/{dept_id}
  返回：{ department, employees, child_departments }
  
GET /contacts/search
  返回：{ employees }
```

#### 数据结构示例
```json
{
  "organization": {
    "id": 2,
    "name": "公司A",
    "root_department_id": 100
  },
  "departments_tree": [
    {
      "id": 100,
      "name": "技术部",
      "level": 1,
      "parent_id": null,
      "employees": [
        { "id": 1000, "name": "张三", "position": "架构师" }
      ],
      "children": [
        {
          "id": 101,
          "name": "后端组",
          "level": 2,
          "parent_id": 100,
          "employees": [
            { "id": 1001, "name": "李四", "position": "高级工程师" }
          ],
          "children": []
        }
      ]
    },
    {
      "id": 103,
      "name": "销售部",
      "level": 1,
      "employees": [...]
    }
  ]
}
```

---

### 方案B：大型集团通讯录（推荐用于复杂组织结构）

这是一个真实的**集团级企业**通讯录设计，适用于多法人、多层级、跨地区的组织。

#### 核心理念
- **Organization 树**：法律实体树（集团→总公司→分公司→分支机构）
- **Department 树**：人员组织树（部门→子部门→工作组）
- **Location**：地理位置（办公地点隔离）
- **EmployeeDepartment**：员工可关联多个部门，但只有一个主部门

#### 真实场景示例

```
┌─────────────────────────────────────────────────────────┐
│              某大型科技集团的组织结构                      │
└─────────────────────────────────────────────────────────┘

集团总部
├─ 深圳总公司（总部）
│  ├─ [北京分公司] ── 地址：北京市朝阳区
│  │  └─ 技术部
│  │     ├─ 后端组
│  │     └─ 前端组
│  │
│  ├─ [上海分公司] ── 地址：上海市浦东新区
│  │  └─ 销售部
│  │     ├─ 销售一组
│  │     └─ 销售二组
│  │
│  └─ [深圳总部] ─── 地址：深圳市南山区
│     ├─ 管理部
│     ├─ 财务部
│     └─ 人力资源部
│
└─ 成都子公司
   └─ 运营部
```

#### 表结构详细设计

##### 1. Organization 表（法律实体）

```rust
pub struct Organization {
    pub id: i64,
    pub tenant_id: i64,
    pub parent_id: Option<i64>,     // 上级组织（形成树）
    pub code: String,                // 组织编码：如 "GROUP", "BJ-BRANCH"
    pub name: String,                // 组织名称：如 "某科技集团"
    pub org_type: i16,               // ★ 组织类型
                                     //   1=集团 2=总公司 3=分公司 4=分支机构
    pub location_id: Option<i64>,    // ★ 地址 (在 location 表)
    pub is_operational: bool,        // ★ 是否可运营部门（能否有员工）
    pub level: i32,                  // 层级深度（便于查询）
    pub path: String,                // 路径：/1/2/5/ （便于范围查询）
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**说明**：
- `org_type` 区分法律实体的性质
- `is_operational` = true 表示这个组织可以有员工直接隶属（如分公司）
- `path` 用于快速查询某组织的所有下级（WHERE path LIKE '/1/2/%'）

##### 2. Department 表（人员部门）

```rust
pub struct Department {
    pub id: i64,
    pub tenant_id: i64,
    pub org_id: i64,                 // ★ 严格关联到某个 Organization
    pub parent_id: Option<i64>,      // 上级部门
    pub code: String,                // 部门编码：如 "TECH", "TECH-BACKEND"
    pub name: String,                // 部门名称
    pub full_name: Option<String>,   // 完整名称：如 "北京分公司/技术部/后端组"
    pub leader_id: Option<i64>,      // 部门主管（员工ID）
    pub level: i32,                  // 层级深度
    pub path: String,                // 路径：/100/101/102/
    pub status: i16,                 // 1=启用 0=禁用
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**约束**：
```sql
-- 部门必须属于 is_operational=true 的组织
ALTER TABLE department 
ADD CONSTRAINT fk_dept_operational_org
FOREIGN KEY (org_id) REFERENCES organization(id)
WHERE is_operational = 1;

-- 部门主管必须也属于同一组织
ALTER TABLE department
ADD CONSTRAINT fk_dept_leader_same_org
CHECK (leader_id IS NULL 
  OR EXISTS (
    SELECT 1 FROM employee_department ed
    WHERE ed.employee_id = department.leader_id
    AND EXISTS (
      SELECT 1 FROM employee e 
      WHERE e.id = ed.employee_id 
      AND e.org_id = department.org_id
    )
  ));
```

##### 3. Location 表（地理位置，新增）

```rust
pub struct Location {
    pub id: i64,
    pub tenant_id: i64,
    pub name: String,                // "北京总部", "上海浦东办公室"
    pub address: String,             // 详细地址
    pub city: String,                // 城市
    pub country: String,             // 国家
    pub latitude: Option<f64>,       // 纬度
    pub longitude: Option<f64>,      // 经度
    pub created_at: DateTime<Utc>,
}
```

##### 4. Employee 表（员工）

```rust
pub struct Employee {
    pub id: i64,
    pub tenant_id: i64,
    pub org_id: i64,                 // ★ 属于哪个组织（通常是可操作的分公司）
    pub primary_dept_id: i64,        // ★ 主部门（新增）
    pub user_id: i64,                // 关联用户
    pub employee_no: String,         // 工号
    pub name: String,
    pub email: String,
    pub mobile: String,
    pub location_id: Option<i64>,    // 工作地点
    pub status: i16,                 // 0=离职 1=在职 2=试用期
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**约束**：
```sql
-- 员工的 org_id 必须和其主部门的 org_id 相同
ALTER TABLE employee 
ADD CONSTRAINT fk_emp_primary_dept_same_org
FOREIGN KEY (org_id) REFERENCES organization(id)
  WHERE is_operational = 1;

ALTER TABLE employee
ADD CONSTRAINT check_emp_dept_org_match
CHECK (EXISTS (
  SELECT 1 FROM department d 
  WHERE d.id = employee.primary_dept_id
  AND d.org_id = employee.org_id
));
```

##### 5. EmployeeDepartment 表（员工多部门关联，保持现有）

```rust
pub struct EmployeeDepartment {
    pub id: i64,
    pub tenant_id: i64,
    pub employee_id: i64,
    pub department_id: i64,
    pub is_primary: i16,             // 1=主部门 0=兼职
    pub is_leader: i16,              // 1=此部门主管 0=普通成员
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}
```

**约束**：
```sql
-- 每个员工只能有一个主部门
ALTER TABLE employee_department
ADD CONSTRAINT uk_emp_primary_dept
UNIQUE(employee_id) WHERE is_primary = 1;

-- 员工的主部门必须匹配 employee.primary_dept_id
ALTER TABLE employee_department
ADD CONSTRAINT check_sync_primary_dept
CHECK (is_primary = 0 
  OR EXISTS (
    SELECT 1 FROM employee e
    WHERE e.id = employee_department.employee_id
    AND e.primary_dept_id = employee_department.department_id
  ));
```

#### 数据示例

```sql
-- 1. 创建组织树
INSERT INTO organization VALUES
(1, 1, NULL,    'GROUP',    '某科技集团',    1, NULL, 1, 1, '/1/'),
(2, 1, 1,       'HEAD',     '深圳总公司',    2, NULL, 1, 2, '/1/2/'),
(10, 1, 2,      'BJ-BRANCH', '北京分公司',   3, 20, 1, 3, '/1/2/10/'),  -- location_id=20
(11, 1, 2,      'SH-BRANCH', '上海分公司',   3, 21, 1, 3, '/1/2/11/'),
(12, 1, 2,      'SZ-HQ',     '深圳总部',     3, 19, 1, 3, '/1/2/12/'),
(30, 1, NULL,   'CHENGDU',   '成都子公司',   2, 22, 1, 2, '/30/');

-- 2. 创建地点
INSERT INTO location VALUES
(19, 1, '深圳总部', '深圳市南山区科技园', '深圳', '中国'),
(20, 1, '北京总部', '北京市朝阳区CBD', '北京', '中国'),
(21, 1, '上海浦东', '上海市浦东新区', '上海', '中国'),
(22, 1, '成都基地', '成都市高新区', '成都', '中国');

-- 3. 创建部门（只在 is_operational=true 的组织下）
INSERT INTO department VALUES
-- 北京分公司的部门
(100, 1, 10, NULL, 'TECH', '技术部', '北京分公司/技术部', 1001, 1, '/100/', 1),
(101, 1, 10, 100, 'TECH-BACKEND', '后端组', '北京分公司/技术部/后端组', 1002, 2, '/100/101/', 1),
(102, 1, 10, 100, 'TECH-FRONTEND', '前端组', '北京分公司/技术部/前端组', 1003, 2, '/100/102/', 1),
(103, 1, 10, NULL, 'SALES', '销售部', '北京分公司/销售部', 1004, 1, '/103/', 1),

-- 上海分公司的部门
(200, 1, 11, NULL, 'SALES', '销售部', '上海分公司/销售部', 2001, 1, '/200/', 1),
(201, 1, 11, 200, 'SALES-1', '销售一组', '上海分公司/销售部/销售一组', 2002, 2, '/200/201/', 1),

-- 深圳总部的部门
(300, 1, 12, NULL, 'ADMIN', '管理部', '深圳总部/管理部', 3001, 1, '/300/', 1),
(301, 1, 12, NULL, 'FINANCE', '财务部', '深圳总部/财务部', 3002, 1, '/301/', 1),

-- 成都子公司的部门
(400, 1, 30, NULL, 'OPERATION', '运营部', '成都子公司/运营部', 4001, 1, '/400/', 1);

-- 4. 创建员工
INSERT INTO employee VALUES
-- 北京分公司
(1001, 1, 10, 100, 1001, 'EMP001', '张三', 'zhangsan@company.com', '13800138001', 20, 1),
(1002, 1, 10, 101, 1002, 'EMP002', '李四', 'lisi@company.com', '13800138002', 20, 1),
(1003, 1, 10, 102, 1003, 'EMP003', '王五', 'wangwu@company.com', '13800138003', 20, 1),
-- 上海分公司
(2001, 1, 11, 200, 2001, 'EMP101', '陈六', 'chenlu@company.com', '13800138004', 21, 1),
(2002, 1, 11, 201, 2002, 'EMP102', '杨七', 'yangqi@company.com', '13800138005', 21, 1),
-- 深圳总部
(3001, 1, 12, 300, 3001, 'EMP201', '周八', 'zhousan@company.com', '13800138006', 19, 1),
(3002, 1, 12, 301, 3002, 'EMP202', '胡九', 'hujiu@company.com', '13800138007', 19, 1),
-- 成都子公司
(4001, 1, 30, 400, 4001, 'EMP301', '林十', 'linshi@company.com', '13800138008', 22, 1);

-- 5. 创建员工-部门关系
INSERT INTO employee_department VALUES
(1, 1, 1001, 100, 1, 0),  -- 张三是技术部成员（主部门）
(2, 1, 1002, 101, 1, 1),  -- 李四是后端组成员（主部门，部门主管）
(3, 1, 1003, 102, 1, 1),  -- 王五是前端组成员（主部门，部门主管）
(4, 1, 2001, 200, 1, 1),  -- 陈六是销售部主管
(5, 1, 2002, 201, 1, 0),  -- 杨七在销售一组
(6, 1, 3001, 300, 1, 1),  -- 周八是管理部主管
(7, 1, 3002, 301, 1, 1),  -- 胡九是财务部主管
(8, 1, 4001, 400, 1, 1);  -- 林十是运营部主管
```

#### 关键查询模式

##### 1. 查询某分公司的完整通讯录

```sql
-- 获取北京分公司的所有部门和员工（树形）
SELECT 
    d.id, d.name, d.path, d.full_name,
    e.id as emp_id, e.name as emp_name, e.email, e.mobile,
    l.name as location_name
FROM department d
LEFT JOIN employee_department ed ON d.id = ed.department_id AND ed.is_primary = 1
LEFT JOIN employee e ON ed.employee_id = e.id
LEFT JOIN location l ON e.location_id = l.id
WHERE d.org_id = 10  -- 北京分公司
ORDER BY d.path, e.employee_no;
```

##### 2. 查询员工的完整汇报链（从下到上）

```sql
-- 获取员工 1002（李四）的汇报链
WITH RECURSIVE emp_chain AS (
    -- 基础：李四
    SELECT 
        e.id, e.name, ed.department_id,
        d.name as dept_name, d.leader_id,
        1 as level
    FROM employee e
    JOIN employee_department ed ON e.id = ed.employee_id
    JOIN department d ON ed.department_id = d.id
    WHERE e.id = 1002 AND ed.is_primary = 1
    
    UNION ALL
    
    -- 递归：找部门主管
    SELECT 
        e.id, e.name, d.id,
        d.name, d.leader_id,
        ec.level + 1
    FROM emp_chain ec
    JOIN employee e ON ec.leader_id = e.id
    JOIN department d ON e.primary_dept_id = d.id
    WHERE ec.leader_id IS NOT NULL
        AND ec.level < 10  -- 防止无限递归
)
SELECT * FROM emp_chain;
```

##### 3. 跨组织搜索员工

```sql
-- 搜索所有包含"李"的员工（跨所有组织）
SELECT 
    e.*, 
    o.name as org_name,
    d.full_name as dept_path,
    l.city
FROM employee e
JOIN organization o ON e.org_id = o.id
JOIN department d ON e.primary_dept_id = d.id
LEFT JOIN location l ON e.location_id = l.id
WHERE e.name LIKE '%李%'
    AND e.tenant_id = 1
    AND e.status = 1  -- 仅在职员工
ORDER BY e.name;
```

##### 4. 获取组织树（带员工、部门统计）

```sql
WITH dept_stats AS (
    SELECT 
        department_id,
        COUNT(DISTINCT employee_id) as emp_count
    FROM employee_department
    WHERE is_primary = 1
    GROUP BY department_id
)
SELECT 
    o.id, o.name, o.org_type, o.path,
    d.id as dept_id, d.name as dept_name,
    COALESCE(ds.emp_count, 0) as employee_count,
    COUNT(DISTINCT d2.id) as child_dept_count
FROM organization o
LEFT JOIN department d ON o.id = d.org_id AND d.parent_id IS NULL
LEFT JOIN department d2 ON d.id = d2.parent_id
LEFT JOIN dept_stats ds ON d.id = ds.department_id
WHERE o.tenant_id = 1
    AND o.org_type IN (3, 2)  -- 只显示分公司和子公司
GROUP BY o.id, d.id, ds.emp_count
ORDER BY o.path, d.path;
```

---

### 方案C：混合方案（最灵活，推荐）

#### 核心思想
```
Organization 用途：
  ├── 法律实体管理（集团/子公司等）
  └── 权限隔离
  
Department 用途：
  ├── 具体的人员组织
  └── 通讯录主体结构
  
Employee 用途：
  └── 员工信息 + 部门关联
```

#### 关键改进
```rust
// Department 表改进
pub struct Department {
    pub id: i64,
    pub org_id: i64,           // 所属组织
    pub parent_id: Option<i64>, // 上级部门
    pub is_primary_dept: bool,  // ★ 是否主部门（新增）
    // ... 其他字段
}

// Employee 表改进
pub struct Employee {
    pub id: i64,
    pub org_id: i64,           // 所属组织（通常是公司/分公司ID）
    // 不再通过部门表查询部门，而是通过 primary_dept_id
    pub primary_dept_id: i64,  // ★ 主部门（新增）
    // ... 其他字段
}
```

#### 优势
- 快速查询员工的主部门
- Department 可以灵活建立多级结构
- Organization 保持独立的法律关系树
- 支持员工跨部门关联

---

## 4. 大型企业通讯录 API 设计

### 核心 API 集

#### 1️⃣ 组织相关

```bash
# 获取组织树（包含下级部门数、员工数）
GET /contacts/orgs/tree
返回：
{
  "id": 1, "name": "某科技集团", "org_type": 1,
  "org_type_name": "集团",
  "statistics": {
    "direct_employees": 3,
    "total_employees": 45,
    "department_count": 12,
    "child_org_count": 4
  },
  "children": [
    {
      "id": 10, "name": "北京分公司", "org_type": 3,
      "location": { "id": 20, "city": "北京", "address": "..." },
      "statistics": { "direct_employees": 12, "total_employees": 12, ... }
    }
  ]
}

# 获取某组织详情（含上级组织路径）
GET /contacts/orgs/{org_id}
返回：
{
  "id": 10, "name": "北京分公司", "org_type": 3,
  "parent_id": 2,
  "ancestor_path": "某科技集团 / 深圳总公司 / 北京分公司",
  "location": { "id": 20, "city": "北京", ... },
  "department_tree": [ ... ],
  "statistics": { "employee_count": 12, "department_count": 4 }
}
```

#### 2️⃣ 部门相关

```bash
# 获取某组织的部门树
GET /contacts/orgs/{org_id}/departments/tree
Query: ?include_employees=true&show_statistics=true
返回：
{
  "id": 10, "name": "北京分公司",
  "departments": [
    {
      "id": 100, "name": "技术部", "level": 1, "path": "/100/",
      "leader": { "id": 1001, "name": "张三", "email": "..." },
      "statistics": {
        "direct_employees": 2,
        "total_employees": 5  // 含子部门员工
      },
      "employees": [
        { "id": 1001, "name": "张三", "position": "部门总监", "is_leader": true }
      ],
      "children": [
        {
          "id": 101, "name": "后端组", "level": 2, "path": "/100/101/",
          "leader": { "id": 1002, "name": "李四", ... },
          "direct_employees": 3,
          "employees": [
            { "id": 1002, "name": "李四", "email": "lisi@company.com", ... }
          ]
        },
        {
          "id": 102, "name": "前端组", "level": 2, "path": "/100/102/",
          "direct_employees": 2,
          "employees": [ ... ]
        }
      ]
    }
  ]
}

# 获取部门详情及成员
GET /contacts/departments/{dept_id}
返回：
{
  "id": 101, "name": "后端组", "full_name": "北京分公司/技术部/后端组",
  "org": { "id": 10, "name": "北京分公司" },
  "parent": { "id": 100, "name": "技术部" },
  "leader": { "id": 1002, "name": "李四", "email": "lisi@company.com" },
  "members": [
    { "id": 1002, "name": "李四", "email": "lisi@company.com", "is_primary": true, "is_leader": true },
    { "id": 1005, "name": "钱八", "email": "qianba@company.com", "is_primary": false, "is_leader": false }
  ],
  "statistics": { "member_count": 2, "child_dept_count": 0 }
}

# 获取部门所有成员（包括子部门）
GET /contacts/departments/{dept_id}/members/all
Query: ?recursive=true&status=1
返回：员工列表（按部门树展示）
```

#### 3️⃣ 员工相关

```bash
# 搜索员工（跨组织）
GET /contacts/employees/search
Query: ?keyword=李&org_id=10&status=1&limit=20
返回：
{
  "total": 3,
  "items": [
    {
      "id": 1002, "name": "李四", "emoji": "😀",
      "email": "lisi@company.com", "mobile": "13800138002",
      "org": { "id": 10, "name": "北京分公司" },
      "primary_dept": { "id": 101, "name": "后端组", "full_path": "技术部/后端组" },
      "departments": [ // 所有关联部门
        { "id": 101, "name": "后端组", "is_primary": true, "is_leader": true },
        { "id": 100, "name": "技术部", "is_primary": false, "is_leader": false }
      ],
      "location": { "id": 20, "city": "北京", "address": "朝阳区CBD" }
    }
  ]
}

# 获取员工详情（含完整信息）
GET /contacts/employees/{emp_id}
返回：
{
  "id": 1002, "name": "李四", "email": "lisi@company.com",
  "employee_no": "EMP002", "mobile": "13800138002",
  "org": { "id": 10, "name": "北京分公司" },
  "org_path": "某科技集团 / 深圳总公司 / 北京分公司",
  "primary_dept": { "id": 101, "name": "后端组", "leader_id": 1002 },
  "dept_path": "技术部 / 后端组",
  "all_departments": [ ... ],
  "manager_chain": [
    { "id": 1002, "name": "李四", "title": "高级工程师" },
    { "id": 1001, "name": "张三", "title": "技术总监", "is_manager": true },
    { "id": 999, "name": "CEO", "title": "首席执行官", "is_ceo": true }
  ],
  "reports": [  // 直属下级
    { "id": 1006, "name": "赵九", "email": "zhaojiuempty.com" }
  ]
}

# 获取员工汇报链（向上）
GET /contacts/employees/{emp_id}/manager-chain
返回：员工 > 直属主管 > 部门主管 > 组织高管 > CEO

# 获取员工下属（向下）
GET /contacts/employees/{emp_id}/reports
返回：所有直属员工列表

# 获取员工名片
GET /contacts/employees/{emp_id}/card
返回：{ id, name, title, dept, email, mobile, phone, office_location, avatar }
```

#### 4️⃣ 位置相关

```bash
# 获取所有办公地点及在该地点的人数
GET /contacts/locations/summary
返回：
[
  { "id": 19, "city": "深圳", "address": "南山区科技园", "employee_count": 8 },
  { "id": 20, "city": "北京", "address": "朝阳区CBD", "employee_count": 12 },
  { "id": 21, "city": "上海", "address": "浦东新区", "employee_count": 7 }
]

# 获取某地点的所有员工
GET /contacts/locations/{loc_id}/employees
返回：该地点的所有员工列表
```

#### 5️⃣ 统计相关

```bash
# 获取组织统计（多维度）
GET /contacts/statistics/organizations
Query: ?level=all
返回：
{
  "total_org_count": 5,
  "total_employee_count": 45,
  "org_breakdown": {
    "group": 1,
    "company": 1,
    "branch": 3,
    "subsidiary": 0
  },
  "by_org": [
    { "org_id": 2, "org_name": "深圳总公司", "emp_count": 8, "dept_count": 3 }
  ]
}

# 获取部门统计
GET /contacts/statistics/departments?org_id=10
返回：按部门的人数、层级统计
```
## 5. 实现步骤和缓存策略

### Phase A: 基础通讯录（1-2周）

#### Step 1: 数据库变更（2天）
```sql
-- 1. 修改 Organization 表
ALTER TABLE organization ADD COLUMN (
    org_type TINYINT DEFAULT 2 COMMENT '1=集团 2=总公司 3=分公司 4=分支机构',
    is_operational BOOLEAN DEFAULT 1 COMMENT '是否可运营部门',
    location_id BIGINT,
    level INT GENERATED ALWAYS AS (char_length(path) - 1) / 2 STORED,
    path VARCHAR(500) GENERATED ALWAYS AS (CONCAT_WS('/', 
        SUBSTRING_INDEX(path, '/', 1),
        SUBSTRING_INDEX(path, '/', 2),
        ...)) STORED
);

-- 2. 修改 Department 表
ALTER TABLE department ADD COLUMN (
    full_name VARCHAR(255),
    level INT,
    path VARCHAR(500)
);

-- 3. 修改 Employee 表
ALTER TABLE employee ADD COLUMN (
    primary_dept_id BIGINT COMMENT '主部门'
);

-- 4. 创建 Location 表
CREATE TABLE location (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT,
    name VARCHAR(100),
    address VARCHAR(255),
    city VARCHAR(50),
    country VARCHAR(50),
    latitude DECIMAL(10, 8),
    longitude DECIMAL(11, 8),
    created_at TIMESTAMP
);

-- 5. 创建索引
CREATE INDEX idx_org_path ON organization(tenant_id, path);
CREATE INDEX idx_dept_org ON department(org_id, parent_id);
CREATE INDEX idx_emp_org_dept ON employee(org_id, primary_dept_id);
CREATE INDEX idx_emp_dept_primary ON employee_department(employee_id, is_primary);
```

#### Step 2: Service 实现（3-4天）
```rust
// 1. 组织查询服务
pub struct ContactsService {
    db_pool: Arc<DbPool>,
    redis_cache: Arc<RedisCache>,
}

impl ContactsService {
    // 获取组织树（带缓存）
    pub async fn get_org_tree(&self, tenant_id: i64) -> Result<OrgTreeResponse>;
    
    // 获取部门树（某组织）
    pub async fn get_dept_tree(&self, org_id: i64) -> Result<DeptTreeResponse>;
    
    // 搜索员工（全文）
    pub async fn search_employees(&self, query: EmployeeSearchQuery) -> Result<Vec<EmployeeCard>>;
    
    // 获取汇报链
    pub async fn get_manager_chain(&self, emp_id: i64) -> Result<Vec<Employee>>;
}

// 2. 缓存键设计
const CACHE_ORG_TREE = "contacts:org_tree:{tenant_id}";           // 1小时
const CACHE_DEPT_TREE = "contacts:dept_tree:{org_id}";            // 1小时
const CACHE_DEPT_MEMBERS = "contacts:dept:{dept_id}:members";     // 30分钟
const CACHE_EMP_CARD = "contacts:emp:{emp_id}:card";              // 30分钟
const CACHE_MANAGER_CHAIN = "contacts:emp:{emp_id}:manager";      // 1小时
```

#### Step 3: 缓存聚焦（1-2天）
- **热数据**：组织树、部门树（变化不频繁，可缓存）
- **温数据**：员工列表（可缓存，但需要监听更新）
- **冷数据**：搜索结果（不缓存，每次查询）

**缓存失效触发**：
```rust
// 当组织信息变更时
InvalidateCache::org_tree(tenant_id);
InvalidateCache::dept_tree(org_id);

// 当部门信息变更时
InvalidateCache::dept_tree(org_id);
InvalidateCache::dept_members(dept_id);

// 当员工信息变更时
InvalidateCache::emp_card(emp_id);
InvalidateCache::manager_chain(emp_id);
InvalidateCache::dept_members(dept_id);  // 若所属部门变更
```

---

### Phase B: 高级功能（2-3周）

#### 功能1: 搜索优化
```rust
// 支持多纬度搜索
pub async fn search_employees(&self, query: EmployeeSearchQuery) -> Result<SearchResponse> {
    // 支持的搜索维度：
    // - 名字（模糊）
    // - 工号（精确）
    // - 邮箱（模糊）
    // - 电话（精确）
    // - 部门（精确或路径）
    // - 位置（精确）
    // - 组织（精确）
    
    // 可选：集成 Elasticsearch 全文搜索
}
```

#### 功能2: 权限控制
```rust
// 通讯录权限模型
pub enum ContactsPermission {
    PublicContact,           // 不受限
    OrganizationContact,     // 仅看同组织的人
    DepartmentContact,       // 仅看同部门的人
    ManagerContact,          // 仅看下属
}

// 实现权限过滤
pub async fn get_org_tree(&self, tenant_id: i64, user_id: i64) -> Result<OrgTreeResponse> {
    let policy = self.get_permission_policy(user_id);
    let mut tree = self.get_full_org_tree(tenant_id).await?;
    tree = tree.filter_by_permission(policy);
    Ok(tree)
}
```

#### 功能3: 汇报关系可视化
```rust
pub async fn get_manager_org_chart(&self, emp_id: i64) -> Result<OrgChartResponse> {
    // 返回以该员工为中心的纵向汇报关系（上下级）
    // {
    //   "employee": { ... },
    //   "manager": { ... },
    //   "reports": [ ... ],
    //   "peers": [ ... ]  // 同部门同级
    // }
}
```

---

### Phase C: 用户体验（1-2周）

#### 功能1: 名片分享
```rust
// 生成员工名片二维码
pub async fn generate_contact_qrcode(&self, emp_id: i64) -> Result<QRCodeResponse> {
    // vcard 格式
    // QR code 指向：/contacts/cards/{emp_id}
}
```

#### 功能2: 通讯录订阅预警
```rust
// 监听通讯录变化，推送到客户端
pub async fn subscribe_contacts(&self, tenant_id: i64) -> WebSocketStream {
    // 事件：
    // - EmployeeJoined { emp_id, dept_id }
    // - EmployeeTransferred { emp_id, from_dept, to_dept }
    // - EmployeeLeft { emp_id }
    // - DepartmentRestructured { org_id }
}
```

#### 功能3: 移动端离线缓存
```rust
// 支持增量更新
pub async fn get_contacts_delta(&self, tenant_id: i64, since: DateTime<Utc>) 
    -> Result<ContactsDeltaResponse> {
    // 返回该时间点以后的所有变更
    // 移动端可增量同步
}
```

---

## 6. 关键性能优化

### 查询性能

#### 问题 1: 部门树查询速度
```
原始SQL每次都要遍历所有部门：
SELECT ... WHERE org_id = 10 ORDER BY path;

优化方案：
1. 使用 path 字段建立索引
2. 缓存整个部门树（树不变时）
3. 使用 Materialized Path 加快查询和修改
```

#### 问题 2: 员工搜索速度（数千员工情况）
```
原始SQL：SELECT * FROM employee WHERE name LIKE '%xx%'

优化方案：
1. 建立全文索引 FULLTEXT(name, email)
2. 支持拼音搜索（使用 pinyin 插件）
3. 集成 Elasticsearch
4. 三字符以上才允许搜索 + 限制返回条数
```

#### 问题 3: 跨组织统计查询
```
原始SQL：多表 JOIN，查询慢

优化方案：
1. 预计算统计数据（晚上离线计算）
2. 使用 materialized view
3. 缓存统计结果（每小时更新）
```

### 缓存策略

| 数据 | 缓存时间 | 失效条件 | 优先级 |
|------|---------|---------|--------|
| 组织树 | 1小时 | 组织新增/修改 | ⭐⭐⭐ |
| 部门树 | 1小时 | 部门新增/修改/删除 | ⭐⭐⭐ |
| 员工卡片 | 30分钟 | 员工信息变更 | ⭐⭐ |
| 汇报链 | 1小时 | 员工转移部门 | ⭐⭐ |
| 搜索结果 | 5分钟 | 员工新增/删除 | ⭐ |

---

## 7. 数据一致性检查

```sql
-- 定期运行（每周）

-- 检查1：员工的主部门必须在其所属组织内
SELECT e.id, e.name, e.org_id, e.primary_dept_id, d.org_id as dept_org_id
FROM employee e
LEFT JOIN department d ON e.primary_dept_id = d.id
WHERE e.primary_dept_id IS NOT NULL AND e.org_id != d.org_id;

-- 检查2：员工的所有部门关联必须在同一组织
SELECT ed.employee_id, ed.department_id, 
       e.org_id as emp_org_id, d.org_id as dept_org_id
FROM employee_department ed
JOIN employee e ON ed.employee_id = e.id
JOIN department d ON ed.department_id = d.id
WHERE e.org_id != d.org_id;

-- 检查3：部门主管必须在该部门
SELECT d.id, d.name, d.leader_id, ed.department_id
FROM department d
LEFT JOIN employee_department ed ON d.leader_id = ed.employee_id AND ed.department_id = d.id
WHERE d.leader_id IS NOT NULL AND ed.id IS NULL;

-- 检查4：部门必须属于 is_operational 的组织
SELECT d.id, d.name, d.org_id, o.is_operational
FROM department d
JOIN organization o ON d.org_id = o.id
WHERE o.is_operational = 0;
```

---

## 9. 与飞书设计的对比分析

### 当前设计 vs 飞书标准

让我对比一下现有的表结构与飞书的差异。

#### Organization 表对比

| 字段 | 当前实现 | 飞书标准 | 差异 | 影响 |
|------|--------|---------|------|------|
| **id** | ✅ | ✅ | - | - |
| **parent_id** | ✅ | ✅ | 相同 | ✅ 支持组织树 |
| **code** | ✅ | ✅ | 相同 | ✅ 编码管理 |
| **name** | ✅ | ✅ | 相同 | ✅ |
| **type** | ✅ (1,2,3,4) | ✅ (1,2,3,4) | 完全相同 | ✅ 类型区分 |
| **logo** | ✅ | ✅ (选项) | 相同 | ✅ |
| **description** | ✅ | ✅ (选项) | 相同 | ✅ |
| **location_id** | ❌ **缺少** | ✅ | ⚠️ | ❌ 无法关联地点 |
| **status** | ✅ (0/1) | ✅ (0/1) | 相同 | ✅ |
| **path** | ❌ **缺少** | ✅ | ⚠️ | ❌ 范围查询慢 |
| **level** | ❌ **缺少** | ✅ | ⚠️ | ❌ 必须递归计算 |
| **created_at/updated_at** | ✅ | ✅ | 相同 | ✅ |

**问题分析**：
- ❌ **缺少 location_id**：无法直接查询"某地所有分公司"
- ❌ **缺少 path**：跨级查询需要递归遍历（性能差）
- ❌ **缺少 level**：无法快速判断组织深度

**改进建议**：

```sql
ALTER TABLE organization ADD COLUMN (
    location_id BIGINT COMMENT '所在地点',
    path VARCHAR(500) COMMENT '路径: /1/2/5/',
    level INT COMMENT '深度: 1=集团 2=总公司 3=分公司',
    is_operational BOOL DEFAULT 1 COMMENT '是否可运营部门'
);
```

---

#### Department 表对比

| 字段 | 当前实现 | 飞书标准 | 差异 | 影响 |
|------|--------|---------|------|------|
| **id** | ✅ | ✅ | - | - |
| **org_id** | ✅ | ✅ | 相同 | ✅ 组织隶属 |
| **parent_id** | ✅ | ✅ | 相同 | ✅ 部门树 |
| **code** | ✅ | ✅ | 相同 | ✅ |
| **name** | ✅ | ✅ | 相同 | ✅ |
| **full_name** | ✅ | ✅ | 相同 | ✅ |
| **path** | ✅ | ✅ | 相同 | ✅ |
| **level** | ✅ | ✅ | 相同 | ✅ |
| **leader_employee_id** | ✅ | ✅ | 相同 | ✅ |
| **status** | ✅ (0/1) | ✅ (0/1) | 相同 | ✅ |
| **description** | ❌ **缺少** | ✅ (选项) | ⚠️ | ⚠️ 部门介绍 |
| **custom_fields** | ❌ **缺少** | ✅ | ⚠️ | ❌ 无法扩展 |
| **created_at/updated_at** | ✅ | ✅ | 相同 | ✅ |

**评价**：部门表设计 ✅ **已经很接近飞书标准**，主要缺少的是可选字段。

**改进建议**：

```sql
ALTER TABLE department ADD COLUMN (
    description VARCHAR(500) COMMENT '部门介绍',
    department_manager_id BIGINT COMMENT '部门经理（副主管）',
    created_by BIGINT COMMENT '创建人'
) AFTER leader_employee_id;
```

---

#### Employee 表对比

| 字段 | 当前实现 | 飞书标准 | 差异 | 影响 |
|------|--------|---------|------|------|
| **id** | ✅ | ✅ | - | - |
| **org_id** | ✅ | ✅ | 相同 | ✅ 法定组织 |
| **user_id** | ✅ | ✅ | 相同 | ✅ |
| **employee_no** | ✅ | ✅ | 相同 | ✅ |
| **name** | ✅ | ✅ | 相同 | ✅ |
| **email** | ✅ | ✅ | 相同 | ✅ |
| **mobile** | ✅ | ✅ | 相同 | ✅ |
| **avatar** | ✅ | ✅ | 相同 | ✅ |
| **gender** | ✅ | ✅ (可选) | 相同 | ✅ |
| **hire_date** | ✅ | ✅ | 相同 | ✅ |
| **leave_date** | ✅ | ✅ | 相同 | ✅ |
| **status** | ✅ (0,1,2,3) | ✅ (0,1,2,3) | 完全相同 | ✅ |
| **primary_dept_id** | ❌ **缺少** | ✅ | ⚠️ | ❌ 需要 JOIN 查询 |
| **work_location** | ❌ **缺少** | ✅ | ⚠️ | ❌ 工作地点 |
| **idcard** | ❌ **缺少** | ✅ (可选) | ⚠️ | ⚠️ HR 需要 |
| **phone** | ❌ **缺少** | ✅ (可选) | ⚠️ | ⚠️ 座机号 |
| **department_title** | ❌ **缺少** | ✅ (可选) | ⚠️ | ⚠️ 部门内职位 |
| **custom_fields** | ❌ **缺少** | ✅ | ⚠️ | ❌ 无扩展字段 |

**问题分析**：
- ❌ **缺少 primary_dept_id**：获取员工主部门需要 JOIN employee_department 表
- ❌ **缺少 work_location**：无法快速查询"北京地区的员工"
- ❌ **缺少部门内职位**：只有岗位，没有部门职位

**改进建议**：

```sql
ALTER TABLE employee ADD COLUMN (
    primary_dept_id BIGINT COMMENT '★ 主部门ID',
    work_location_id BIGINT COMMENT '★ 工作地点',
    phone VARCHAR(20) COMMENT '座机号',
    idcard VARCHAR(20) COMMENT '身份证号（encrypted）',
    department_title VARCHAR(100) COMMENT '部门内职位'
) AFTER org_id;

-- 创建索引加速查询
CREATE INDEX idx_employee_location ON employee(tenant_id, work_location_id);
CREATE INDEX idx_employee_primary_dept ON employee(tenant_id, primary_dept_id);
```

---

#### EmployeeDepartment 表对比

| 字段 | 当前实现 | 飞书标准 | 差异 | 影响 |
|------|--------|---------|------|------|
| **id** | ✅ | ✅ | - | - |
| **employee_id** | ✅ | ✅ | 相同 | ✅ |
| **department_id** | ✅ | ✅ | 相同 | ✅ |
| **is_primary** | ✅ | ✅ | 相同 | ✅ |
| **is_leader** | ✅ | ✅ | 相同 | ✅ |
| **join_date** | ✅ | ✅ | 相同 | ✅ |
| **leave_date** | ✅ | ✅ | 相同 | ✅ |
| **is_temporary** | ❌ **缺少** | ✅ | ⚠️ | ❌ 无法标记借调 |
| **secondment_id** | ❌ **缺少** | ✅ | ⚠️ | ❌ 借调链接 |
| **role** | ❌ **缺少** | ✅ | ⚠️ | ⚠️ 权限角色 |

**问题分析**：
- ❌ **缺少 is_temporary 和 secondment_id**：无法支持借调功能
- ❌ **缺少 role 字段**：权限定义不清楚

**改进建议**（已在借调方案中）：

```sql
ALTER TABLE employee_department ADD COLUMN (
    is_temporary TINYINT DEFAULT 0 COMMENT '★ 是否临时/借调',
    secondment_id BIGINT COMMENT '★ 关联借调ID',
    role VARCHAR(50) COMMENT '★ 角色: viewer/contributor/manager'
) AFTER is_leader;
```

---

### 差异总结表

```
等级    | 重要性    | 缺失项目              | 影响
--------|----------|----------------------|----------------------------------
🔴     | 必须修复   | Employee.primary_dept_id | O(n) 查询变 O(1)，性能关键
🔴     | 必须修复   | Org.path & Org.level    | 跨级查询从递归变范围查询
🔴     | 必须修复   | 借调支持               | 无法支持员工跨组织流动
🟡     | 应该加    | Employee.work_location | 地理位置管理很常用
🟡     | 应该加    | Department.description | ✅ 低优先级，可选
🟢     | 可以加    | 自定义字段             | 未来扩展性
```

---

### 改造行动计划

#### **Phase 1: 关键字段补齐（Week 1）**

```sql
-- 1. Organization 表补齐
ALTER TABLE organization ADD COLUMN (
    location_id BIGINT,
    path VARCHAR(500),
    level INT,
    is_operational BOOL DEFAULT 1
);

-- 2. Employee 表补齐
ALTER TABLE employee ADD COLUMN (
    primary_dept_id BIGINT,
    work_location_id BIGINT,
    phone VARCHAR(20),
    department_title VARCHAR(100)
);

-- 3. EmployeeDepartment 表补齐【借调必需】
ALTER TABLE employee_department ADD COLUMN (
    is_temporary TINYINT DEFAULT 0,
    secondment_id BIGINT,
    role VARCHAR(50)
);

-- 4. 创建必要索引
CREATE INDEX idx_org_path ON organization(tenant_id, path);
CREATE INDEX idx_dept_org ON department(org_id, parent_id);
CREATE INDEX idx_emp_primary_dept ON employee(tenant_id, primary_dept_id);
CREATE INDEX idx_emp_location ON employee(tenant_id, work_location_id);
CREATE INDEX idx_emp_dept_temp ON employee_department(is_temporary, secondment_id);
```

**影响**：
- ✅ 启用所有跨级查询
- ✅ 支持借调功能
- ✅ 性能提升 10-100 倍（某些查询）

#### **Phase 2: 数据初始化（Week 1）**

```sql
-- 1. 计算 Organization 树的 path 和 level
UPDATE organization o1
SET 
    level = (
        SELECT COUNT(*) + 1 
        FROM organization o2 
        WHERE o2.id = o1.parent_id
        OR (o2.parent_id = o1.parent_id 
            AND o2.id < o1.id)  -- 简化，实际需要递归
    ),
    path = (
        WITH RECURSIVE ancestors AS (
            SELECT o.id, o.parent_id, CAST(o.id AS CHAR(500)) as path
            FROM organization o WHERE o.id = o1.id
            UNION ALL
            SELECT o.id, o.parent_id, CONCAT(o.id, '/', a.path)
            FROM organization o 
            JOIN ancestors a ON o.id = a.parent_id
        )
        SELECT CONCAT('/', REVERSE(path), '/') FROM ancestors
        WHERE parent_id IS NULL LIMIT 1
    );

-- 2. 为现有员工设置 primary_dept_id
UPDATE employee e
SET primary_dept_id = (
    SELECT department_id 
    FROM employee_department 
    WHERE employee_id = e.id AND is_primary = 1
    LIMIT 1
)
WHERE primary_dept_id IS NULL;
```

#### **Phase 3: API 优化（Week 2）**

```rust
// 之前（需要 JOIN）
let dept = service.get_department_by_employee(emp_id).await?;

// 之后（直接读取）
let emp = service.get_employee(emp_id).await?;
let dept = service.get_department(emp.primary_dept_id).await?;  // ✅ O(1)

// 新增 API：获取地点的所有员工
let employees = service.get_employees_by_location(loc_id).await?;

// 新增 API：跨级查询
let all_employees = service.get_employees_by_org(org_id, recursive=true).await?;
```

---

### 与飞书的完整功能对比

| 功能 | 当前| 改造后 | 飞书 | 说明 |
|------|-----|--------|------|------|
| **组织树** | ✅ | ✅ | ✅ | 已支持 |
| **部门树** | ✅ | ✅ | ✅ | 已支持 |
| **员工主部门** | ⚠️ | ✅ | ✅ | 需要改造 |
| **借调关系** | ❌ | ✅ | ✅ | **新增** |
| **地理位置** | ❌ | ✅ | ✅ | **新增** |
| **汇报链** | ✅ | ✅ | ✅ | 可支持 |
| **多岗位** | ✅ | ✅ | ✅ | 需要扩展 |
| **权限分级** | ✅ | ✅ | ✅ | 可实现 |
| **跨级查询** | ⚠️ | ✅ | ✅ | 性能提升 |
| **实时更新** | ✅ | ✅ | ✅ | 可集成 |

---

### 实现成本评估

| 项目 | 工作量 | 风险 | 优先级 |
|------|--------|------|--------|
| **Officer path/level** | 2 天 | 低 | 🔴 必做 |
| **Employee.primary_dept_id** | 2 天 | 低 | 🔴 必做 |
| **Employee.work_location** | 1 天 | 低 | 🟡 建议 |
| **借调表结构** | 1 天 | 低 | 🔴 必做 |
| **借调 API** | 3 天 | 中 | 🔴 必做 |
| **数据初始化脚本** | 1 天 | 中 | 🔴 必做 |
| **查询优化** | 2 天 | 低 | 🟡 建议 |
| **权限规则实现** | 2 天 | 高 | 🟢 可选 |

**总计：14-16 天（2-3 周）**

---

### 结论

✅ **好消息**：当前设计已经 **70% 符合飞书标准**

⚠️ **差距**：
1. 缺少位置关联（location_id）
2. 缺少员工主部门快速访问字段（primary_dept_id）
3. 缺少组织树的 path/level 优化
4. 完全缺少借调支持

🎯 **建议优先级**：
- **Week 1**：补齐 path/level/primary_dept_id + 借调表结构
- **Week 2-3**：实现借调 API + 权限规则
- **Week 4+**：位置管理、权限细分、移动端支持

这样改造后，功能上会 **99% 接近飞书的标准**。

---

## 10. 跨组织借调设计方案（参考飞书）

### 业务背景

在大型企业中，跨组织借调很常见：
```
场景1：项目制
  集团发起某项目 → 从各分公司抽调骨干
  "李四（北京分公司后端组）" 借调到 "集团技术中心"

场景2：临时支援
  上海分公司人手不足 → 借调北京工程师 3 个月

场景3：总部支持
  财务部需要集团总部的支持 → CFO 作为虚拟成员

场景4：矩阵式组织
  员工既属于职能部门，也属于产品线部门
```

### 飞书的设计思路

飞书的员工模型有 3 个核心概念：

```
1. 主部门（Primary Department）
   └─ 员工的法定部门
   └─ 人事关系归属
   └─ 薪酬、考勤在这里

2. 虚拟部门成员（Virtual Member）
   └─ 员工可以加入其他部门（跨组织）
   └─ 作为"临时成员"
   └─ 有开始/结束日期
   └─ 可设置权限级别（viewer/editor/manager）

3. 岗位（Position）
   └─ 员工可以兼任多个岗位
   └─ 岗位对应权限（如"部门主管"）
   └─ 可跨部门、跨组织
```

### 推荐的数据库设计

#### 1. 新增表：Secondment（借调关系）

```rust
pub struct Secondment {
    pub id: i64,                           // 借调ID
    pub tenant_id: i64,
    pub employee_id: i64,                  // 员工ID
    pub from_org_id: i64,                  // 来源组织（法定隶属）
    pub from_dept_id: i64,                 // 来源部门（主部门）
    pub to_org_id: i64,                    // 借调到的组织
    pub to_dept_id: i64,                   // 借调到的部门
    pub role: String,                      // 角色：team-lead, contributor, viewer 等
    pub status: i16,                       // 1=生效 0=已结束 -1=已撤销
    pub start_date: NaiveDate,             // 借调开始日期
    pub end_date: Option<NaiveDate>,       // 借调结束日期（NULL=长期）
    pub reason: Option<String>,            // 借调原因
    pub approval_status: i16,              // 0=待审批 1=已批准 -1=已驳回
    pub approved_by: Option<i64>,          // 审批人
    pub approved_at: Option<DateTime<Utc>>,
    pub created_by: i64,                   // 申请人
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**关键字段说明**：
- `from_org_id/from_dept_id`：员工的法定部门（不变）
- `to_org_id/to_dept_id`：临时部门（可能来自其他组织）
- `role`：可控制权限（贡献者/查看者/经理等）
- `status`：区分生效/过期/撤销
- `approval_status`：是否需要审批流程

#### 2. 修改 EmployeeDepartment 表添加虚拟成员

```rust
pub struct EmployeeDepartment {
    pub id: i64,
    pub tenant_id: i64,
    pub employee_id: i64,
    pub department_id: i64,
    pub is_primary: i16,                   // 1=主部门 0=兼职/虚拟
    pub is_leader: i16,                    // 1=部门主管 0=普通成员
    // ★ 新增以下字段
    pub is_temporary: i16,                 // ★ 1=借调成员 0=常规成员
    pub secondment_id: Option<i64>,        // ★ 关联借调记录
    pub start_date: Option<NaiveDate>,     // ★ 加入日期（用于借调）
    pub end_date: Option<NaiveDate>,       // ★ 离开日期（用于借调）
    pub is_deleted: i16,                   // 0=正常 1=已删除
    pub created_at: DateTime<Utc>,
}
```

**说明**：
- `is_temporary=1` 表示这是借调关系
- `secondment_id` 指向 Secondment 表
- 员工可以同时有多个 EmployeeDepartment 记录（主部门 + 多个借调部门）

#### 3. 新增表：Position（岗位）

```rust
pub struct Position {
    pub id: i64,
    pub tenant_id: i64,
    pub department_id: i64,                // 岗位归属部门
    pub name: String,                      // "技术总监", "技术经理", "工程师"
    pub level: i16,                        // 级别：1=基层 2=中层 3=高管
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub struct EmployeePosition {
    pub id: i64,
    pub tenant_id: i64,
    pub employee_id: i64,
    pub position_id: i64,
    pub org_id: Option<i64>,               // ★ 如果职位跨组织时填写
    pub is_primary: i16,                   // 1=主岗位 0=兼任
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}
```

### 数据示例（跨组织借调）

```sql
-- 场景：李四（北京分公司后端组）被借调到集团技术中心 3 个月

-- 1. 员工李四的主部门（不变）
SELECT * FROM employee WHERE id = 1002;
-- 结果：org_id=10 (北京分公司), primary_dept_id=101 (后端组)

-- 2. 创建借调关系
INSERT INTO secondment VALUES (
    1,                              -- id
    1,                              -- tenant_id
    1002,                           -- employee_id (李四)
    10,                             -- from_org_id (北京分公司)
    101,                            -- from_dept_id (后端组)
    1,                              -- to_org_id (集团)
    50,                             -- to_dept_id (集团技术中心)
    'contributor',                  -- role
    1,                              -- status (生效)
    '2026-02-10',                   -- start_date
    '2026-05-10',                   -- end_date (3个月后)
    '临时支持集团技术项目',          -- reason
    1,                              -- approval_status (已批准)
    999,                            -- approved_by (批准人ID)
    NOW(),                          -- approved_at
    1001,                           -- created_by (申请人)
    NOW(),                          -- created_at
    NOW()                           -- updated_at
);

-- 3. 在 EmployeeDepartment 表中添加临时关系
INSERT INTO employee_department VALUES (
    201,                            -- id
    1,                              -- tenant_id
    1002,                           -- employee_id (李四)
    50,                             -- department_id (集团技术中心)
    0,                              -- is_primary (0 = 不是主部门)
    0,                              -- is_leader (0 = 不是主管)
    1,                              -- is_temporary (★ 1 = 借调成员)
    1,                              -- secondment_id (★ 关联借调ID)
    '2026-02-10',                   -- start_date
    '2026-05-10',                   -- end_date
    0,                              -- is_deleted
    NOW(),                          -- created_at
);

-- 查询结果：李四现在同时属于两个部门
-- 主部门：北京分公司/后端组（永久）
-- 借调部门：集团/技术中心（2026-02-10 到 2026-05-10）
```

### API 设计

#### 1. 创建借调请求

```bash
POST /contacts/secondments
{
  "employee_id": 1002,
  "from_dept_id": 101,              # 来源部门（自动读取员工主部门）
  "to_org_id": 1,                   # 目标组织
  "to_dept_id": 50,                 # 目标部门
  "start_date": "2026-02-10",
  "end_date": "2026-05-10",         # 可选，NULL=长期
  "role": "contributor",             # viewer/contributor/team-lead
  "reason": "临时支持集团技术项目",
  "require_approval": true           # 是否需要审批
}

返回：
{
  "id": 1,
  "status": "pending_approval",
  "created_at": "2026-02-10T10:00:00Z"
}
```

#### 2. 批准/驳回借调请求

```bash
PATCH /contacts/secondments/{secondment_id}/approve
{
  "action": "approve",  # approve/reject/revoke
  "reason": "同意"
}

返回：
{
  "id": 1,
  "status": "active",
  "approval_status": 1,
  "approved_by": 999,
  "approved_at": "2026-02-10T11:00:00Z"
}
```

#### 3. 获取员工的所有部门（含借调）

```bash
GET /contacts/employees/{emp_id}/departments
返回：
{
  "employee": {
    "id": 1002, "name": "李四"
  },
  "primary_dept": {
    "id": 101, "name": "后端组",
    "org": { "id": 10, "name": "北京分公司" },
    "status": "permanent"
  },
  "borrowed_depts": [
    {
      "id": 50, "name": "集团技术中心",
      "org": { "id": 1, "name": "集团" },
      "role": "contributor",
      "start_date": "2026-02-10",
      "end_date": "2026-05-10",
      "status": "active"
    }
  ]
}
```

#### 4. 获取部门成员（分类展示）

```bash
GET /contacts/departments/{dept_id}/members
Query: ?include_temporary=true
返回：
{
  "department": { "id": 50, "name": "集团技术中心" },
  "permanent_members": [
    { "id": 999, "name": "王总", "role": "manager" }
  ],
  "temporary_members": [  // 借调成员
    {
      "id": 1002, "name": "李四",
      "from_dept": "北京分公司/后端组",
      "role": "contributor",
      "end_date": "2026-05-10",
      "secondment_id": 1
    }
  ],
  "statistics": {
    "total": 8,
    "permanent": 6,
    "temporary": 2,
    "expiring_soon": 1  // 30天内到期的借调
  }
}
```

#### 5. 借调日历视图

```bash
GET /contacts/orgs/{org_id}/secondments
Query: ?from_date=2026-02-01&to_date=2026-02-28&status=active
返回：
{
  "period": { "from": "2026-02-01", "to": "2026-02-28" },
  "statistics": {
    "borrowed_out": 3,    // 借出人数
    "borrowed_in": 5,     // 借入人数
    "expiring": 2         // 本月到期的借调
  },
  "events": [
    {
      "date": "2026-02-10",
      "type": "borrow_in",
      "employee": "李四",
      "from_dept": "后端组 (北京分公司)",
      "to_dept": "集团技术中心",
      "end_date": "2026-05-10"
    }
  ]
}
```

#### 6. 获取员工的有效组织级别

```bash
GET /contacts/employees/{emp_id}/effective-organizations
返回：
{
  "primary": {
    "org_id": 10,
    "org_name": "北京分公司",
    "dept_id": 101,
    "dept_name": "后端组",
    "is_primary": true
  },
  "temporary": [
    {
      "org_id": 1,
      "org_name": "集团",
      "dept_id": 50,
      "dept_name": "集团技术中心",
      "end_date": "2026-05-10"
    }
  ]
}
```

### 权限和查询规则

#### 1. 通讯录可见性规则

```rust
// 员工A 能否看到 员工B 的联系方式？

if A.org_id == B.org_id {
    // 同组织员工，可以看到
    return true;
}

if A 在 B 的部门树中（直属 / 同部门 / 上级）{
    // 直接上下级关系，可以看到
    return true;
}

if exists(Secondment where (A.id 或 B.id 被借调)) {
    // 有借调关系时，按借调部门的可见性规则
    return check_dept_visibility(A, B);
}

// 默认不可见（私有部门）
return false;
```

#### 2. 汇报关系的处理

当员工被借调时，汇报关系的变化：

```
原始汇报关系：
  李四 (后端组) 
    └─ 向张三 (后端组负责人) 汇报

借调后的关系：
  李四 (后端组) 
    ├─ 关系1（永久）：向张三 (后端组负责人) 汇报
    └─ 关系2（临时）：向王总 (集团技术中心负责人) 汇报（到2026-05-10）

// 在借调期间内，两个汇报关系都生效
// 但主汇报关系仍然是原部门（因为主部门不变）
```

#### 3. 考勤/薪酬影响

```
考勤：
  - 借调期间，员工在原部门继续打卡
  - 但工作地点可能在借调组织
  - 可向两个部门提交工作日志

薪酬：
  - 由原部门继续发放
  - 借调组织可额外发放补贴（可选）
  - 一切按原 org_id 的薪酬体系
```

### 实现建议

#### 表结构变更（Week 1）
```sql
ALTER TABLE employee_department ADD COLUMN (
    is_temporary TINYINT DEFAULT 0,
    secondment_id BIGINT,
    actual_start_date DATE,
    actual_end_date DATE
);

CREATE TABLE secondment (
    id BIGINT PRIMARY KEY,
    tenant_id BIGINT,
    employee_id BIGINT,
    from_org_id BIGINT,
    from_dept_id BIGINT,
    to_org_id BIGINT, 
    to_dept_id BIGINT,
    role VARCHAR(50),
    status TINYINT,
    start_date DATE,
    end_date DATE,
    reason VARCHAR(255),
    approval_status TINYINT,
    approved_by BIGINT,
    approved_at TIMESTAMP,
    created_by BIGINT,
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    INDEX idx_employee (employee_id),
    INDEX idx_org (from_org_id, to_org_id),
    INDEX idx_status (status, end_date)
);
```

#### API 优先级
```
Week 2:
  ✅ POST /secondments (创建借调)
  ✅ GET /employees/{id}/departments (含借调)
  ✅ GET /departments/{id}/members (分类展示)

Week 3:
  ✅ PATCH /secondments/{id}/approve (审批流)
  ✅ GET /secondments (列表/搜索)

Week 4:
  ✅ 汇报关系自动调整
  ✅ 权限可见性规则
```

---

🔴 **必做（第1周）**
- [x] Organization 表加 `org_type`, `location_id`
- [x] Department 表加 `full_name`, `level`, `path`
- [x] Employee 表加 `primary_dept_id`
- [x] 创建 Location 表
- [x] 實現 `GET /contacts/orgs/{org_id}/departments/tree` API
- [x] 實現 `GET /contacts/employees/search` API
- [x] 部門樹緩存

🟡 **должны做（第2-3周）**
- [ ] 汇报链 API
- [ ] 位置统计 API
- [ ] 权限控制（OrgContact vs PublicContact）
- [ ] 搜索优化（全文索引）

🟢 **可以做（第4周+）**
- [ ] 名片二维码
- [ ] WebSocket 实时更新
- [ ] Elasticsearch 集成
- [ ] 拼音搜索
