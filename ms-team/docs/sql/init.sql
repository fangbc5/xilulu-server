-- ============================================================
-- ms-team 数据库初始化脚本
-- 所有时间字段统一使用 bigint（毫秒时间戳），与其他微服务保持一致
-- ============================================================

CREATE DATABASE IF NOT EXISTS `ms_team` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci;
USE `ms_team`;

-- ========== 1. 组织表 ==========
CREATE TABLE IF NOT EXISTS `organization` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `parent_id` bigint NOT NULL DEFAULT 0 COMMENT '上级组织ID（支持集团-子公司结构）',
  `code` varchar(64) NOT NULL COMMENT '组织编码（租户内唯一）',
  `name` varchar(128) NOT NULL COMMENT '组织名称',
  `short_name` varchar(64) DEFAULT NULL COMMENT '简称',
  `type` tinyint DEFAULT '1' COMMENT '组织类型：1-集团 2-公司 3-分公司 4-子公司',
  `logo` varchar(512) DEFAULT NULL COMMENT '组织Logo',
  `description` text COMMENT '描述',
  `sort_order` int DEFAULT '0' COMMENT '排序',
  `status` tinyint DEFAULT '1' COMMENT '状态：0-禁用 1-启用',
  `location_id` bigint DEFAULT NULL COMMENT '所在地点（location 表 ID）',
  `path` varchar(500) DEFAULT NULL COMMENT '树路径，如 /1/2/5/，用于范围查询',
  `level` int DEFAULT NULL COMMENT '树深度：1-集团 2-总公司 3-分公司 4-分支机构',
  `is_operational` tinyint DEFAULT '1' COMMENT '是否可运营（能否有员工）',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `created_at` bigint DEFAULT NULL COMMENT '创建时间（毫秒时间戳）',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `updated_at` bigint DEFAULT NULL COMMENT '更新时间（毫秒时间戳）',
  `is_deleted` tinyint DEFAULT '0' COMMENT '是否删除：0-否 1-是',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_tenant_code` (`tenant_id`, `code`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_parent_id` (`parent_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='组织表';

-- ========== 2. 部门表 ==========
CREATE TABLE IF NOT EXISTS `department` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `org_id` bigint NOT NULL COMMENT '所属组织ID',
  `parent_id` bigint DEFAULT NULL COMMENT '上级部门ID（NULL表示顶级部门）',
  `code` varchar(64) NOT NULL COMMENT '部门编码（组织内唯一）',
  `name` varchar(128) NOT NULL COMMENT '部门名称',
  `full_name` varchar(512) DEFAULT NULL COMMENT '部门全称（自动生成：集团/子公司/部门）',
  `path` varchar(512) DEFAULT NULL COMMENT '部门路径（如 /1/2/3/，用于快速查询子树）',
  `level` int DEFAULT '1' COMMENT '层级深度',
  `leader_employee_id` bigint DEFAULT NULL COMMENT '部门负责人（员工ID）',
  `sort_order` int DEFAULT '0' COMMENT '排序',
  `status` tinyint DEFAULT '1' COMMENT '状态：0-禁用 1-启用',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `created_at` bigint DEFAULT NULL COMMENT '创建时间（毫秒时间戳）',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `updated_at` bigint DEFAULT NULL COMMENT '更新时间（毫秒时间戳）',
  `is_deleted` tinyint DEFAULT '0' COMMENT '是否删除：0-否 1-是',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_org_code` (`org_id`, `code`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_org_id` (`org_id`),
  KEY `idx_parent_id` (`parent_id`),
  KEY `idx_path` (`path`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='部门表';

-- ========== 3. 岗位表 ==========
CREATE TABLE IF NOT EXISTS `position` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `org_id` bigint NOT NULL COMMENT '所属组织ID',
  `code` varchar(64) NOT NULL COMMENT '岗位编码（组织内唯一）',
  `name` varchar(128) NOT NULL COMMENT '岗位名称',
  `category` varchar(64) DEFAULT NULL COMMENT '岗位类别（如：管理类、技术类、销售类）',
  `level` int DEFAULT '1' COMMENT '岗位级别（如：P1-P10）',
  `description` text COMMENT '岗位职责描述',
  `requirements` text COMMENT '任职要求',
  `sort_order` int DEFAULT '0' COMMENT '排序',
  `status` tinyint DEFAULT '1' COMMENT '状态：0-禁用 1-启用',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `created_at` bigint DEFAULT NULL COMMENT '创建时间（毫秒时间戳）',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `updated_at` bigint DEFAULT NULL COMMENT '更新时间（毫秒时间戳）',
  `is_deleted` tinyint DEFAULT '0' COMMENT '是否删除：0-否 1-是',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_org_code` (`org_id`, `code`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_org_id` (`org_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='岗位表';

-- ========== 4. 员工表 ==========
CREATE TABLE IF NOT EXISTS `employee` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `org_id` bigint NOT NULL COMMENT '所属组织ID',
  `user_id` bigint NOT NULL COMMENT '关联用户ID（ms-identity.user.id）',
  `employee_no` varchar(64) NOT NULL COMMENT '员工工号（组织内唯一）',
  `name` varchar(64) NOT NULL COMMENT '员工姓名',
  `avatar` varchar(512) DEFAULT NULL COMMENT '员工头像',
  `gender` tinyint DEFAULT '0' COMMENT '性别：0-未知 1-男 2-女',
  `mobile` varchar(20) DEFAULT NULL COMMENT '工作手机',
  `email` varchar(128) DEFAULT NULL COMMENT '工作邮箱',
  `phone` varchar(20) DEFAULT NULL COMMENT '座机号码',
  `hire_date` bigint DEFAULT NULL COMMENT '入职日期（毫秒时间戳）',
  `leave_date` bigint DEFAULT NULL COMMENT '离职日期（毫秒时间戳）',
  `status` tinyint DEFAULT '1' COMMENT '状态：0-离职 1-在职 2-试用期 3-停薪留职',
  `sort_order` int DEFAULT '0' COMMENT '排序',
  `primary_dept_id` bigint DEFAULT NULL COMMENT '主部门ID（冗余，快速查询用）',
  `work_location_id` bigint DEFAULT NULL COMMENT '工作地点（location 表 ID）',
  `department_title` varchar(100) DEFAULT NULL COMMENT '部门内职位（如：资深工程师）',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `created_at` bigint DEFAULT NULL COMMENT '创建时间（毫秒时间戳）',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `updated_at` bigint DEFAULT NULL COMMENT '更新时间（毫秒时间戳）',
  `is_deleted` tinyint DEFAULT '0' COMMENT '是否删除：0-否 1-是',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_org_employee_no` (`org_id`, `employee_no`),
  UNIQUE KEY `uk_org_user` (`org_id`, `user_id`) COMMENT '一个用户在一个组织只能有一个员工身份',
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_org_id` (`org_id`),
  KEY `idx_user_id` (`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='员工表';

-- ========== 5. 员工-部门关系表 ==========
CREATE TABLE IF NOT EXISTS `employee_department` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `employee_id` bigint NOT NULL COMMENT '员工ID',
  `department_id` bigint NOT NULL COMMENT '部门ID',
  `is_primary` tinyint DEFAULT '0' COMMENT '是否主部门：0-否 1-是',
  `is_leader` tinyint DEFAULT '0' COMMENT '是否部门负责人：0-否 1-是',
  `join_date` bigint DEFAULT NULL COMMENT '加入部门日期（毫秒时间戳）',
  `leave_date` bigint DEFAULT NULL COMMENT '离开部门日期（毫秒时间戳）',
  `is_temporary` tinyint DEFAULT '0' COMMENT '是否借调成员：0-常规 1-借调',
  `secondment_id` bigint DEFAULT NULL COMMENT '关联借调记录 ID',
  `role` varchar(50) DEFAULT NULL COMMENT '权限角色：viewer/contributor/team-lead',
  `actual_start_date` bigint DEFAULT NULL COMMENT '实际加入日期（毫秒时间戳）',
  `actual_end_date` bigint DEFAULT NULL COMMENT '实际离开日期（毫秒时间戳）',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `created_at` bigint DEFAULT NULL COMMENT '创建时间（毫秒时间戳）',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_employee_dept` (`employee_id`, `department_id`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_department_id` (`department_id`),
  KEY `idx_emp_dept_primary` (`employee_id`, `is_primary`),
  KEY `idx_emp_dept_temp` (`is_temporary`, `secondment_id`),
  KEY `idx_emp_dept_leader` (`department_id`, `is_leader`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='员工-部门关系表';

-- ========== 6. 员工-岗位关系表 ==========
CREATE TABLE IF NOT EXISTS `employee_position` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `employee_id` bigint NOT NULL COMMENT '员工ID',
  `position_id` bigint NOT NULL COMMENT '岗位ID',
  `is_primary` tinyint DEFAULT '0' COMMENT '是否主岗位：0-否 1-是',
  `start_date` bigint DEFAULT NULL COMMENT '任职开始日期（毫秒时间戳）',
  `end_date` bigint DEFAULT NULL COMMENT '任职结束日期（毫秒时间戳）',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `created_at` bigint DEFAULT NULL COMMENT '创建时间（毫秒时间戳）',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_employee_position` (`employee_id`, `position_id`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_position_id` (`position_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='员工-岗位关系表';

-- ========== 7. 地点管理表 ==========
CREATE TABLE IF NOT EXISTS `location` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `name` varchar(100) NOT NULL COMMENT '地点名称（如：北京总部）',
  `address` varchar(255) DEFAULT NULL COMMENT '详细地址',
  `city` varchar(50) DEFAULT NULL COMMENT '城市',
  `province` varchar(50) DEFAULT NULL COMMENT '省份',
  `country` varchar(50) DEFAULT NULL COMMENT '国家',
  `postal_code` varchar(20) DEFAULT NULL COMMENT '邮编',
  `latitude` decimal(10,8) DEFAULT NULL COMMENT '纬度',
  `longitude` decimal(11,8) DEFAULT NULL COMMENT '经度',
  `phone` varchar(20) DEFAULT NULL COMMENT '地点电话',
  `capacity` int DEFAULT NULL COMMENT '容纳人数',
  `manager_id` bigint DEFAULT NULL COMMENT '地点负责人（employee ID）',
  `status` tinyint DEFAULT '1' COMMENT '状态：0-禁用 1-启用',
  `description` varchar(500) DEFAULT NULL COMMENT '地点介绍',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `created_at` bigint DEFAULT NULL COMMENT '创建时间（毫秒时间戳）',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `updated_at` bigint DEFAULT NULL COMMENT '更新时间（毫秒时间戳）',
  `is_deleted` tinyint DEFAULT '0' COMMENT '是否删除：0-否 1-是',
  PRIMARY KEY (`id`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_city` (`city`),
  KEY `idx_status` (`status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='地点管理表';

-- ========== 8. 员工借调表 ==========
CREATE TABLE IF NOT EXISTS `secondment` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `employee_id` bigint NOT NULL COMMENT '员工ID',
  `from_org_id` bigint NOT NULL COMMENT '来源组织ID',
  `from_dept_id` bigint NOT NULL COMMENT '来源部门ID',
  `to_org_id` bigint NOT NULL COMMENT '借调到的组织ID',
  `to_dept_id` bigint NOT NULL COMMENT '借调到的部门ID',
  `role` varchar(50) NOT NULL DEFAULT 'contributor' COMMENT '角色：viewer/contributor/team-lead',
  `status` tinyint NOT NULL DEFAULT '1' COMMENT '状态：1-生效 0-已结束 -1-已撤销',
  `start_date` bigint NOT NULL COMMENT '借调开始日期（毫秒时间戳）',
  `end_date` bigint DEFAULT NULL COMMENT '借调结束日期（毫秒时间戳，NULL=长期）',
  `reason` varchar(500) DEFAULT NULL COMMENT '借调原因',
  `approval_status` tinyint DEFAULT '0' COMMENT '审批状态：0-待审批 1-已批准 -1-已驳回',
  `approved_by` bigint DEFAULT NULL COMMENT '审批人',
  `approved_at` bigint DEFAULT NULL COMMENT '批准时间（毫秒时间戳）',
  `rejected_reason` varchar(255) DEFAULT NULL COMMENT '驳回原因',
  `comment` varchar(500) DEFAULT NULL COMMENT '备注',
  `created_by` bigint NOT NULL COMMENT '申请人',
  `created_at` bigint DEFAULT NULL COMMENT '创建时间（毫秒时间戳）',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `updated_at` bigint DEFAULT NULL COMMENT '更新时间（毫秒时间戳）',
  `is_deleted` tinyint DEFAULT '0' COMMENT '是否删除：0-否 1-是',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_emp_org_temp` (`employee_id`, `to_org_id`, `end_date`),
  KEY `idx_employee` (`employee_id`),
  KEY `idx_org_from` (`from_org_id`),
  KEY `idx_org_to` (`to_org_id`),
  KEY `idx_dept_from` (`from_dept_id`),
  KEY `idx_dept_to` (`to_dept_id`),
  KEY `idx_status` (`status`),
  KEY `idx_approval` (`approval_status`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='员工借调表';