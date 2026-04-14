# 数据库迁移执行验证检查清单

## 执行前准备

- [ ] 备份生产数据库
- [ ] 在开发/测试环境验证所有脚本
- [ ] 准备回滚方案
- [ ] 通知相关团队维护窗口
- [ ] 确认无关键业务进行中

## 迁移执行步骤

### 第1步：基础表结构增强 (V001-V003)

```bash
# 执行脚本
mysql -u root -p ms_team_db < V001_organization_enhancements.sql
mysql -u root -p ms_team_db < V002_employee_enhancements.sql
mysql -u root -p ms_team_db < V003_employee_department_enhancements.sql
```

**执行后验证：**

```sql
-- 检查 Organization 表新增列
DESC organization;
-- 应该看到: location_id, path, level, is_operational

-- 检查 Employee 表新增列
DESC employee;
-- 应该看到: primary_dept_id, work_location_id, phone, department_title

-- 检查 EmployeeDepartment 表新增列
DESC employee_department;
-- 应该看到: is_temporary, secondment_id, role, actual_start_date, actual_end_date
```

验证清单：
- [ ] Organization 表有 location_id 列，类型: BIGINT
- [ ] Organization 表有 path 列，类型: VARCHAR(500)
- [ ] Organization 表有 level 列，类型: INT
- [ ] Organization 表有 is_operational 列，类型: TINYINT
- [ ] Employee 表有 primary_dept_id 列，类型: BIGINT
- [ ] Employee 表有 work_location_id 列，类型: BIGINT
- [ ] Employee 表有 phone 列，类型: VARCHAR(20)
- [ ] Employee 表有 department_title 列，类型: VARCHAR(100)
- [ ] EmployeeDepartment 表有 is_temporary 列，类型: TINYINT
- [ ] EmployeeDepartment 表有 secondment_id 列，类型: BIGINT
- [ ] EmployeeDepartment 表有 role 列，类型: VARCHAR(50)
- [ ] EmployeeDepartment 表有 actual_start_date 列，类型: DATETIME
- [ ] EmployeeDepartment 表有 actual_end_date 列，类型: DATETIME

### 第2步：新增数据表 (V004-V005)

```bash
# 执行脚本
mysql -u root -p ms_team_db < V004_create_location_table.sql
mysql -u root -p ms_team_db < V005_create_secondment_table.sql
```

**执行后验证：**

```sql
-- 检查 Location 表创建
DESC location;
-- 应该看到: 16 列

-- 检查 Secondment 表创建
DESC secondment;
-- 应该看到: 22 列

-- 检查表是否为空
SELECT COUNT(*) FROM location;   -- 应该是 0
SELECT COUNT(*) FROM secondment; -- 应该是 0
```

验证清单：
- [ ] Location 表成功创建
- [ ] Location 表包含所有16个列
- [ ] Secondment 表成功创建
- [ ] Secondment 表包含所有22个列
- [ ] 两个新表都为空
- [ ] 索引已创建

### 第3步：数据初始化 (V006-V007)

```bash
# 执行脚本（需要V001完成）
mysql -u root -p ms_team_db < V006_init_organization_path_level.sql

# 执行脚本（需要V002完成）
mysql -u root -p ms_team_db < V007_init_employee_primary_dept.sql
```

**执行后验证 - V006：**

```sql
-- 检查 Organization path/level 是否已填充
SELECT COUNT(*) as total,
       COUNT(path) as with_path,
       COUNT(level) as with_level,
       COUNT(CASE WHEN path IS NULL THEN 1 END) as missing_path
FROM organization;

-- 示例查询：验证路径格式正确
SELECT id, parent_id, path, level 
FROM organization 
WHERE path IS NOT NULL 
ORDER BY level, id
LIMIT 10;

-- 示例：查询某个组织的所有子组织
SELECT * FROM organization 
WHERE path LIKE '/1/2/%'  -- 查询组织ID为2的所有子组织
ORDER BY path;

-- 检查数据一致性
SELECT parent_id, COUNT(*) as org_count
FROM organization
WHERE parent_id IS NOT NULL AND path IS NULL
GROUP BY parent_id;
-- 应该返回空结果（即所有有parent_id的都应该有path）
```

验证清单：
- [ ] 所有 Organization 记录都有 path 值（null count = 0）
- [ ] 所有 Organization 记录都有 level 值（null count = 0）
- [ ] path 格式正确，示例：/1/2/5/
- [ ] level 值与 path 中的层级一致
- [ ] 根组织的 path 为 `/{id}/`，level 为 1
- [ ] 无孤立记录（有 parent_id 的都有 path）

**执行后验证 - V007：**

```sql
-- 检查 Employee primary_dept_id 是否已填充
SELECT COUNT(*) as total,
       COUNT(primary_dept_id) as with_primary_dept,
       COUNT(CASE WHEN primary_dept_id IS NULL THEN 1 END) as missing_primary_dept
FROM employee
WHERE id IN (SELECT DISTINCT employee_id FROM employee_department WHERE is_primary = 1);

-- 示例：查看某个员工的主部门
SELECT e.id, e.user_id, e.primary_dept_id, ed.department_id, d.name
FROM employee e
LEFT JOIN department d ON e.primary_dept_id = d.id
WHERE e.id = 101;

-- 验证 primary_dept_id 与 employee_department 一致性
SELECT e.id, e.primary_dept_id, ed.department_id as actual_primary_dept
FROM employee e
LEFT JOIN employee_department ed ON e.id = ed.employee_id AND ed.is_primary = 1
WHERE e.primary_dept_id != ed.department_id
LIMIT 10;
-- 应该返回空结果
```

验证清单：
- [ ] 所有有主部门记录的 Employee 都填充了 primary_dept_id
- [ ] primary_dept_id 值与 employee_department 中的主记录一致
- [ ] 无重复主部门（每个 employee 最多有一个 is_primary=1 的记录）
- [ ] 无孤立记录（primary_dept_id 对应的 department 存在）

## 性能验证

```sql
-- 测试新索引效果

-- 1. Organization path 索引测试
EXPLAIN SELECT * FROM organization WHERE path LIKE '/1/2/%';
-- 应该使用 org_path 索引

-- 2. Employee primary_dept 索引测试
EXPLAIN SELECT * FROM employee WHERE primary_dept_id = 101;
-- 应该使用 emp_primary_dept_id 索引

-- 3. 查询性能基准
-- before: 需要递归查询计算路径
-- after: 直接 LIKE 查询，使用索引

-- 测试查询耗时
SELECT COUNT(*) FROM organization WHERE path LIKE '/1/%';
-- 对于大多数表，应该 < 100ms

-- 4. 索引统计
SELECT TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX, COLUMN_NAME
FROM INFORMATION_SCHEMA.STATISTICS
WHERE TABLE_SCHEMA = 'ms_team_db'
  AND TABLE_NAME IN ('organization', 'employee', 'employee_department')
ORDER BY TABLE_NAME, INDEX_NAME, SEQ_IN_INDEX;
```

验证清单：
- [ ] 新索引已创建
- [ ] path/primary_dept_id 查询使用索引
- [ ] 查询耗时在可接受范围（< 200ms）

## 应用兼容性测试

### 代码检查

- [ ] 应用代码无直接引用已删除列（注：本次迁移仅新增列）
- [ ] 应用代码已更新以使用新的 path/primary_dept_id 字段
- [ ] 所有 ORM 映射已更新（Rust sqlx, Diesel 等）

### 功能测试

```sql
-- 测试Feishu功能兼容性

-- 1. 跨组织查询
SELECT e.id, e.user_id, o.name as org_name, d.name as dept_name
FROM employee e
JOIN organization o ON e.primary_org_id = o.id
JOIN department d ON e.primary_dept_id = d.id
WHERE e.status = 'ACTIVE'
LIMIT 10;

-- 2. 借调员工查询
SELECT e.id, ed.department_id, ed.is_temporary, s.from_org_id, s.to_org_id
FROM employee e
JOIN employee_department ed ON e.id = ed.employee_id
LEFT JOIN secondment s ON ed.secondment_id = s.id
WHERE ed.is_temporary = 1;

-- 3. 位置统计
SELECT l.name, COUNT(DISTINCT e.id) as employee_count
FROM location l
LEFT JOIN employee e ON e.work_location_id = l.id
GROUP BY l.id, l.name;
```

验证清单：
- [ ] 跨组织查询正常工作
- [ ] 借调员工可正确查询
- [ ] 位置统计功能正常
- [ ] 应用日志无异常报错

## 回滚计划

### 快速回滚（如果发现问题）

```bash
# 方案1：恢复备份（最安全）
mysql -u root -p < backup_20260210_120000.sql

# 方案2：部分回滚（仅删除新表，保留修改的列）
mysql -u root -p ms_team_db < rollback_v007.sql
mysql -u root -p ms_team_db < rollback_v006.sql
```

### 部分回滚脚本

**V007回滚：**
```sql
-- 仅清空 primary_dept_id，保留列结构
UPDATE employee SET primary_dept_id = NULL;
```

**V006回滚：**
```sql
-- 仅清空 path/level，保留列结构
UPDATE organization SET path = NULL, level = NULL;
```

**完整回滚：**
```sql
-- 删除新增列（需谨慎）
ALTER TABLE organization DROP COLUMN location_id, DROP COLUMN path, DROP COLUMN level, DROP COLUMN is_operational;
ALTER TABLE employee DROP COLUMN primary_dept_id, DROP COLUMN work_location_id, DROP COLUMN phone, DROP COLUMN department_title;
ALTER TABLE employee_department DROP COLUMN is_temporary, DROP COLUMN secondment_id, DROP COLUMN role, DROP COLUMN actual_start_date, DROP COLUMN actual_end_date;

-- 删除新表
DROP TABLE IF EXISTS secondment;
DROP TABLE IF EXISTS location;
```

验证清单：
- [ ] 备份已保存于安全位置
- [ ] 回滚脚本已编写并测试
- [ ] 团队成员了解回滚程序

## 最终检查

- [ ] 所有验证查询通过
- [ ] 性能测试通过
- [ ] 应用功能测试通过
- [ ] 日志无告警
- [ ] 用户报告无异常
- [ ] 文档已更新
- [ ] 迁移版本已记录

## 执行完成

**迁移完成时间：** ___________

**执行人员：** ___________

**验证人员：** ___________

**备注：** 

___________________________________________________________________________

___________________________________________________________________________

---

## 快速参考命令

```bash
# 一键执行（推荐）
bash run_migrations.sh --dev

# 手动执行每个步骤
for script in V001 V002 V003 V004 V005 V006 V007; do
  echo "Migrating $script..."
  mysql -u root -p ms_team_db < "${script}"*.sql
  echo "Done."
done

# 验证所有表结构
for table in organization employee employee_department location secondment; do
  echo "=== $table ==="
  mysql -u root -p ms_team_db -e "DESC $table;" | head -5
done
```
