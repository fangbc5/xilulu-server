-- ============================================================================
-- Version: V006
-- Date: 2026-02-10
-- Author: AI Assistant
-- Description: 数据初始化 - 计算 Organization 表的 path 和 level
-- Usage: 在执行 V001 后运行此脚本
-- ============================================================================

-- 分步更新 organization 表的 level 和 path

-- Step 1: 检查列是否存在（这一步手动确认，因为MySQL不支持检查列是否存在的简单方法）
-- 确保在运行此脚本之前已执行 V001_organization_enhancements.sql

-- Step 2: 计算 level（树深度）- 只更新尚未设置 level 的记录
UPDATE organization 
SET level = 1 
WHERE parent_id IS NULL 
  AND level IS NULL;

-- 使用递归CTE更新子节点的level
-- 注意：MySQL 8.0+才支持递归CTE
UPDATE organization o1
JOIN (
    WITH RECURSIVE org_levels AS (
        SELECT id, 1 as lvl FROM organization WHERE parent_id IS NULL
        UNION ALL
        SELECT o.id, ol.lvl + 1
        FROM organization o
        JOIN org_levels ol ON o.parent_id = ol.id
    )
    SELECT id, lvl FROM org_levels
) o2 ON o1.id = o2.id
SET o1.level = o2.lvl
WHERE o1.level IS NULL;

-- Step 3: 计算 path（使用递归CTE，仅适用于MySQL 8.0+）
-- 为根节点设置初始path
UPDATE organization 
SET path = CONCAT('/', id, '/') 
WHERE parent_id IS NULL 
  AND path IS NULL;

-- 递归更新子节点的path
UPDATE organization o1
JOIN (
    WITH RECURSIVE org_paths AS (
        -- 根节点
        SELECT id, parent_id, CAST(CONCAT('/', id, '/') AS CHAR(500)) as path
        FROM organization
        WHERE parent_id IS NULL
        UNION ALL
        -- 子节点
        SELECT o.id, o.parent_id, CONCAT(op.path, o.id, '/')
        FROM organization o
        JOIN org_paths op ON o.parent_id = op.id
    )
    SELECT id, path FROM org_paths
) o2 ON o1.id = o2.id
SET o1.path = o2.path;

-- Step 4: 验证数据正确性
SELECT '=== Organization Path/Level 初始化完成 ===' as message;

-- 检查：显示几个组织的 level 和 path
SELECT id, name, parent_id, level, path 
FROM organization 
LIMIT 10;

-- 检查：查找缺少 level 或 path 的记录
SELECT COUNT(*) as missing_level_count 
FROM organization 
WHERE level IS NULL;

SELECT COUNT(*) as missing_path_count 
FROM organization 
WHERE path IS NULL;

-- ============================================================================
-- 查询示例：
--
-- 1. 查询某组织的所有下级组织
--    SELECT * FROM organization 
--    WHERE path LIKE '/1/2/%' AND id != 2;
--
-- 2. 查询某组织的直接子组织
--    SELECT * FROM organization 
--    WHERE parent_id = 2;
--
-- 3. 查询深度为 3 的所有组织
--    SELECT * FROM organization WHERE level = 3;
--
-- ============================================================================

COMMIT;