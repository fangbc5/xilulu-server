-- ============================================================================
-- Version: V002
-- Date: 2026-02-10
-- Author: AI Assistant
-- Description: 增强 Employee 表，支持飞书标准的员工信息
-- Changes:
--   - 添加 primary_dept_id（主部门快速访问）
--   - 添加 work_location_id（工作地点）
--   - 添加 phone（座机号）
--   - 添加 department_title（部门内职位）
-- ============================================================================

-- 添加列
-- 注意：MySQL不支持 ADD COLUMN IF NOT EXISTS，所以我们只使用 ADD COLUMN
-- 如果列已存在，脚本会失败，这是预期行为，表明您已经运行过此脚本
ALTER TABLE employee 
ADD COLUMN primary_dept_id BIGINT COMMENT '★ 主部门 ID，用于快速查询（复制自 employee_department）',
ADD COLUMN work_location_id BIGINT COMMENT '工作地点（location 表 ID）',
ADD COLUMN phone VARCHAR(20) COMMENT '座机号码',
ADD COLUMN department_title VARCHAR(100) COMMENT '部门内职位（如：资深工程师、技术总监）';

-- 创建索引以加速查询
-- MySQL不支持 CREATE INDEX IF NOT EXISTS，所以如果索引已存在，此命令将失败
CREATE INDEX idx_emp_primary_dept ON employee(tenant_id, primary_dept_id);
CREATE INDEX idx_emp_work_location ON employee(tenant_id, work_location_id);
CREATE INDEX idx_emp_org_status ON employee(tenant_id, org_id, status);
CREATE INDEX idx_emp_user ON employee(tenant_id, user_id);

-- 添加备注
ALTER TABLE employee COMMENT '员工信息表（支持飞书标准）';

-- ============================================================================
-- 数据初始化脚本注意事项：
-- 1. 需要从 employee_department 表中填充 primary_dept_id
-- 2. 参考：docs/sql/INIT_DATA_employee_primary_dept.sql
-- 3. 可选：从第三方系统导入 work_location_id
-- ============================================================================

COMMIT;