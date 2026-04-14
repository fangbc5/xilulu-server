-- ============================================================================
-- Version: V007
-- Date: 2026-02-10
-- Author: AI Assistant
-- Description: 数据初始化 - 填充 Employee 表的 primary_dept_id
-- Usage: 在执行 V002 后运行此脚本
-- ============================================================================

-- Step 1: 检查列是否存在（这一步手动确认，因为MySQL不支持检查列是否存在的简单方法）
-- 确保在运行此脚本之前已执行 V002_employee_enhancements.sql

-- Step 2: 从 employee_department 表中填充 primary_dept_id
-- 只更新那些 primary_dept_id 尚未设置的记录
UPDATE employee e
SET primary_dept_id = (
    SELECT department_id 
    FROM employee_department ed
    WHERE ed.employee_id = e.id 
      AND ed.is_primary = 1
    LIMIT 1
)
WHERE e.primary_dept_id IS NULL
  AND e.id IN (
    SELECT DISTINCT employee_id 
    FROM employee_department 
    WHERE is_primary = 1
  );

-- Step 3: 验证数据一致性
SELECT '=== Employee primary_dept_id 初始化完成 ===' as message;

-- 检查：显示有多少员工已更新
SELECT COUNT(*) as employees_with_primary_dept 
FROM employee 
WHERE primary_dept_id IS NOT NULL;

-- 检查：查找还没有主部门的员工
SELECT COUNT(*) as employees_without_primary_dept 
FROM employee 
WHERE primary_dept_id IS NULL;

-- 检查：验证员工的 primary_dept_id 是否与 org_id 匹配
SELECT e.id, e.name, e.org_id, e.primary_dept_id, d.org_id as dept_org_id
FROM employee e
LEFT JOIN department d ON e.primary_dept_id = d.id
WHERE e.primary_dept_id IS NOT NULL 
  AND e.org_id != d.org_id
LIMIT 10;

-- 如果上面的查询返回行数 > 0，表示有数据不一致
-- 需要手动检查和修复

-- ============================================================================
-- 查询示例：
--
-- 1. 快速获取员工所在部门（无需 JOIN）
--    SELECT e.id, e.name, d.name as dept_name
--    FROM employee e
--    JOIN department d ON e.primary_dept_id = d.id
--    WHERE e.id = 1002;
--
-- 2. 查询某部门的所有员工
--    SELECT e.* FROM employee e
--    WHERE e.primary_dept_id = 101
--    ORDER BY e.employee_no;
--
-- ============================================================================

COMMIT;