-- ============================================================================
-- SQL Migration Scripts - README
-- Version: 2026-02-10
-- ============================================================================

-- ⚡ 快速执行方式（推荐）：
--
-- bash run_migrations.sh --dev          # 开发环境执行
-- bash run_migrations.sh --prod         # 生产环境执行（含备份）
-- bash run_migrations.sh --dev --dry-run # 干运行模式（仅查看）
--
-- ============================================================================

-- 执行顺序（必须按照以下顺序执行）：

-- 1. V001_organization_enhancements.sql
--    目的：增强 Organization 表
--    操作：ADD COLUMN (location_id, path, level, is_operational)
--    风险：低 - 仅添加列，不修改现有数据
--    估计时间：< 1 分钟

-- 2. V002_employee_enhancements.sql
--    目的：增强 Employee 表
--    操作：ADD COLUMN (primary_dept_id, work_location_id, phone, department_title)
--    风险：低 - 仅添加列，不修改现有数据
--    估计时间：< 1 分钟

-- 3. V003_employee_department_enhancements.sql
--    目的：增强 EmployeeDepartment 表，支持借调
--    操作：ADD COLUMN (is_temporary, secondment_id, role, actual_start_date, actual_end_date)
--    风险：低 - 仅添加列，不修改现有数据
--    估计时间：< 1 分钟

-- 4. V004_create_location_table.sql
--    目的：创建地点管理表
--    操作：CREATE TABLE location
--    风险：低 - 新建空表
--    估计时间：< 1 分钟

-- 5. V005_create_secondment_table.sql
--    目的：创建借调管理表
--    操作：CREATE TABLE secondment
--    风险：低 - 新建空表
--    估计时间：< 1 分钟

-- 6. V006_init_organization_path_level.sql
--    目的：初始化 Organization 表的 path 和 level
--    操作：UPDATE organization SET path = ..., level = ...
--    风险：中 - 修改现有数据，需要逻辑计算
--    警告：✅ 有数据验证，可快速回滚
--    估计时间：中等（取决于组织数量）
--    前置条件：V001 已执行

-- 7. V007_init_employee_primary_dept.sql
--    目的：初始化 Employee 表的 primary_dept_id
--    操作：UPDATE employee SET primary_dept_id = ...
--    风险：中 - 修改现有数据，需要 JOIN 查询
--    警告：✅ 有数据验证，可快速回滚
--    估计时间：中等（取决于员工数量）
--    前置条件：V002 已执行

-- ============================================================================
-- 执行步骤

-- Step 1: 在测试环境验证
--    1.1 备份数据库
--    1.2 顺序执行 V001 - V007
--    1.3 运行验证查询
--    1.4 检查性能指标

-- Step 2: 在生产环境执行
--    2.1 选择低流量时段（如早上 2 点）
--    2.2 加锁表，阻止业务操作（可选）
--    2.3 顺序执行 V001 - V007
--    2.4 运行验证查询
--    2.5 监控 CPU 和内存

-- Step 3: 回滚（如果出现问题）
--    3.1 如果只需回滚 V007 和 V006：
--        UPDATE employee SET primary_dept_id = NULL;
--        UPDATE organization SET path = NULL, level = NULL;
--    3.2 如果需要完全回滚：
--        使用之前的数据库备份

-- ============================================================================

-- 快速执行脚本：

-- 一键执行所有迁移（测试环境）
-- source /path/to/docs/sql/V001_organization_enhancements.sql;
-- source /path/to/docs/sql/V002_employee_enhancements.sql;
-- source /path/to/docs/sql/V003_employee_department_enhancements.sql;
-- source /path/to/docs/sql/V004_create_location_table.sql;
-- source /path/to/docs/sql/V005_create_secondment_table.sql;
-- source /path/to/docs/sql/V006_init_organization_path_level.sql;
-- source /path/to/docs/sql/V007_init_employee_primary_dept.sql;

-- ============================================================================
-- 验证脚本（提交后运行）

-- 检查所有表结构
DESC organization;
DESC employee;
DESC employee_department;
DESC location;
DESC secondment;

-- 检查数据一致性
SELECT 'Missing Organization Path' as check_name, COUNT(*) as issue_count 
FROM organization WHERE path IS NULL;

SELECT 'Missing Employee Primary Dept' as check_name, COUNT(*) as issue_count 
FROM employee WHERE primary_dept_id IS NULL AND id IN (
  SELECT DISTINCT employee_id FROM employee_department WHERE is_primary = 1
);

-- 检查性能
EXPLAIN SELECT * FROM organization WHERE path LIKE '/1/2/%';
EXPLAIN SELECT * FROM employee WHERE primary_dept_id = 101;

-- ============================================================================
-- 版本管理

-- 版本历史：
-- V001: 2026-02-10 - Organization 增强
-- V002: 2026-02-10 - Employee 增强
-- V003: 2026-02-10 - EmployeeDepartment 增强（借调）
-- V004: 2026-02-10 - 创建 Location 表
-- V005: 2026-02-10 - 创建 Secondment 表
-- V006: 2026-02-10 - 初始化 Organization path/level
-- V007: 2026-02-10 - 初始化 Employee primary_dept_id

-- ============================================================================
-- 一键执行脚本使用指南 (run_migrations.sh)
-- ============================================================================

-- 脚本位置：docs/sql/run_migrations.sh
-- 
-- 功能：
--  - 自动检查 MySQL 连接
--  - 按正确顺序执行所有迁移脚本
--  - 生产环境自动备份
--  - 详细的执行日志和错误报告
--  - 支持干运行模式（仅查看，不执行）

-- 使用示例：

-- 1. 开发环境执行（推荐用于测试）
bash run_migrations.sh --dev

-- 2. 生产环境执行（含自动备份）
bash run_migrations.sh --prod

-- 3. 干运行模式（查看脚本内容，不执行）
bash run_migrations.sh --dev --dry-run

-- 4. 自定义数据库连接参数
export DB_HOST=production-db.example.com
export DB_PORT=3306
export DB_USER=admin
export DB_PASSWORD=securepass
export DB_NAME=ms_team_db
bash run_migrations.sh --prod

-- 5. 手动执行单个脚本（备选方案）
mysql -h localhost -u root -p ms_team_db < V001_organization_enhancements.sql

-- 脚本输出示例：
-- ✅ MySQL 连接成功
-- ✅ V001_organization_enhancements.sql 执行成功
-- ✅ V002_employee_enhancements.sql 执行成功
-- ...
-- ✅ 所有迁移都执行成功！

-- ============================================================================
-- V007: 2026-02-10 - 初始化 Employee primary_dept_id

-- 当前版本：V007 (2026-02-10)
-- 向后兼容：✅ （只添加列和表，不删除）
-- 回滚策略：✅ 支持完全回滚或部分回滚

-- ============================================================================
