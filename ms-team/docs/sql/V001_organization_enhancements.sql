-- ============================================================================
-- Version: V001
-- Date: 2026-02-10
-- Author: AI Assistant
-- Description: 增强 Organization 表，支持飞书标准的组织树查询
-- Changes:
--   - 添加 location_id（地点关联）
--   - 添加 path（组织树路径，用于范围查询）
--   - 添加 level（组织树深度）
--   - 添加 is_operational（是否可运营）
-- ============================================================================

-- 检查表是否存在
-- ALTER TABLE 会自动判断列是否已存在，但为了安全起见再加一层检查

-- 添加列
-- 注意：MySQL不支持 ADD COLUMN IF NOT EXISTS，所以我们只使用 ADD COLUMN
-- 如果列已存在，脚本会失败，这是预期行为，表明您已经运行过此脚本
ALTER TABLE organization 
ADD COLUMN location_id BIGINT COMMENT '所在地点（location 表 ID）',
ADD COLUMN path VARCHAR(500) COMMENT '树路径，如 /1/2/5/，用于范围查询',
ADD COLUMN level INT COMMENT '树深度：1=集团 2=总公司 3=分公司 4=分支机构',
ADD COLUMN is_operational TINYINT DEFAULT 1 COMMENT '是否可运营部门（能否有员工）';

-- 创建外键约束（可选，取决于业务）
-- ALTER TABLE organization 
-- ADD CONSTRAINT fk_org_location 
-- FOREIGN KEY (location_id) REFERENCES location(id);

-- 创建索引以加速查询
-- MySQL不支持 CREATE INDEX IF NOT EXISTS，所以如果索引已存在，此命令将失败
CREATE INDEX idx_org_path ON organization(tenant_id, path);
CREATE INDEX idx_org_location ON organization(tenant_id, location_id);
CREATE INDEX idx_org_type ON organization(tenant_id, `type`);
CREATE INDEX idx_org_parent ON organization(parent_id);

-- 添加备注
ALTER TABLE organization COMMENT '企业组织结构表（支持飞书标准）';

-- ============================================================================
-- 数据初始化脚本注意事项：
-- 1. 需要手动运行计算现有数据的 path 和 level
-- 2. 参考：docs/sql/INIT_DATA_organization_path_level.sql
-- ============================================================================

COMMIT;