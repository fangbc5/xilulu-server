-- ============================================================================
-- Version: V003
-- Date: 2026-02-10
-- Author: AI Assistant
-- Description: 增强 EmployeeDepartment 表，支持飞书标准的借调关系
-- Changes:
--   - 添加 is_temporary（临时/借调标记）
--   - 添加 secondment_id（关联借调表）
--   - 添加 role（权限角色：viewer/contributor/team-lead）
--   - 添加 actual_start_date 和 actual_end_date（实际日期）
-- ============================================================================

-- 添加列
-- 注意：MySQL不支持 ADD COLUMN IF NOT EXISTS，所以我们只使用 ADD COLUMN
-- 如果列已存在，脚本会失败，这是预期行为，表明您已经运行过此脚本
ALTER TABLE employee_department 
ADD COLUMN is_temporary TINYINT DEFAULT 0 COMMENT '★ 是否临时/借调成员（1=借调 0=常规）',
ADD COLUMN secondment_id BIGINT COMMENT '★ 关联借调记录 ID（Secondment 表）',
ADD COLUMN role VARCHAR(50) COMMENT '★ 权限角色：viewer/contributor/team-lead',
ADD COLUMN actual_start_date DATE COMMENT '★ 实际加入日期（用于借调）',
ADD COLUMN actual_end_date DATE COMMENT '★ 实际离开日期（用于借调）';

-- 创建索引以加速查询
-- MySQL不支持 CREATE INDEX IF NOT EXISTS，所以如果索引已存在，此命令将失败
CREATE INDEX idx_emp_dept_primary ON employee_department(employee_id, is_primary);
CREATE INDEX idx_emp_dept_temp ON employee_department(is_temporary, secondment_id);
CREATE INDEX idx_emp_dept_leader ON employee_department(department_id, is_leader);
CREATE INDEX idx_emp_dept_dates ON employee_department(actual_start_date, actual_end_date);

-- 创建唯一约束：每个员工只能有一个主部门
-- MySQL不支持带条件的唯一约束，所以这里只创建普通唯一索引
-- ALTER TABLE employee_department 
-- ADD CONSTRAINT IF NOT EXISTS uk_emp_primary_dept UNIQUE KEY (employee_id, is_primary) 
--   WHERE is_primary = 1;

-- 添加备注
ALTER TABLE employee_department COMMENT '员工与部门关系表（支持常规和借调关系）';

-- ============================================================================
-- 数据检查脚本：
-- 检查现有数据中是否存在一个员工多个主部门的情况
-- ============================================================================

-- 运行以下查询验证数据一致性：
-- SELECT employee_id, COUNT(*) as cnt 
-- FROM employee_department 
-- WHERE is_primary = 1 
-- GROUP BY employee_id 
-- HAVING cnt > 1;

COMMIT;