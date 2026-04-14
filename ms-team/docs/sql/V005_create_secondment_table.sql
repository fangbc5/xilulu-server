-- ============================================================================
-- Version: V005
-- Date: 2026-02-10
-- Author: AI Assistant
-- Description: 创建 Secondment 表（员工跨组织借调）
-- Changes:
--   - 新建 secondment 表，用于管理员工的借调关系
-- ============================================================================

-- 创建 Secondment 表
CREATE TABLE IF NOT EXISTS secondment (
    id BIGINT PRIMARY KEY COMMENT '借调记录 ID',
    tenant_id BIGINT NOT NULL COMMENT '租户 ID',
    employee_id BIGINT NOT NULL COMMENT '员工 ID',
    from_org_id BIGINT NOT NULL COMMENT '来源组织 ID（员工法定组织）',
    from_dept_id BIGINT NOT NULL COMMENT '来源部门 ID（员工主部门）',
    to_org_id BIGINT NOT NULL COMMENT '借调到的组织 ID',
    to_dept_id BIGINT NOT NULL COMMENT '借调到的部门 ID',
    role VARCHAR(50) NOT NULL DEFAULT 'contributor' COMMENT '角色：viewer/contributor/team-lead',
    status TINYINT NOT NULL DEFAULT 1 COMMENT '状态：1=生效 0=已结束 -1=已撤销',
    start_date DATE NOT NULL COMMENT '借调开始日期',
    end_date DATE COMMENT '借调结束日期（NULL=长期）',
    reason VARCHAR(500) COMMENT '借调原因',
    approval_status TINYINT DEFAULT 0 COMMENT '审批状态：0=待审批 1=已批准 -1=已驳回',
    approved_by BIGINT COMMENT '审批人',
    approved_at TIMESTAMP COMMENT '批准时间',
    rejected_reason VARCHAR(255) COMMENT '驳回原因',
    created_by BIGINT NOT NULL COMMENT '申请人',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '申请时间',
    updated_by BIGINT COMMENT '更新人',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    comment VARCHAR(500) COMMENT '备注',
    is_deleted TINYINT DEFAULT 0 COMMENT '是否删除',
    
    KEY idx_employee (employee_id),
    KEY idx_org_from (from_org_id),
    KEY idx_org_to (to_org_id),
    KEY idx_dept_from (from_dept_id),
    KEY idx_dept_to (to_dept_id),
    KEY idx_status (status, end_date),
    KEY idx_approval (approval_status),
    KEY idx_date_range (start_date, end_date),
    UNIQUE KEY uk_emp_org_temp (employee_id, to_org_id, end_date)  -- 同一员工不能同时借调到同一组织多次
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci 
COMMENT='员工借调关系表（支持跨组织临时调动）';

-- ============================================================================
-- 使用示例：
-- INSERT INTO secondment VALUES (
--   1, 1, 1002, 10, 101, 1, 50, 'contributor', 1,
--   '2026-02-10', '2026-05-10', '项目支持', 1, 999, NOW(), NULL,
--   1001, NOW(), NULL, NOW(), NULL, 0
-- );
--
-- 说明：
--   - 员工 1002（李四）
--   - 来自北京分公司(10)/后端组(101)
--   - 借调到集团(1)/技术中心(50)
--   - 角色：贡献者
--   - 期限：2026-02-10 到 2026-05-10
-- ============================================================================

COMMIT;