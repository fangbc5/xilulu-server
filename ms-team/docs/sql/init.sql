use `ms_team`

-- ms_team.department definition

CREATE TABLE `department` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `org_id` bigint NOT NULL COMMENT '所属组织ID',
  `parent_id` bigint DEFAULT NULL COMMENT '上级部门ID（NULL表示顶级部门）',
  `code` varchar(64) NOT NULL COMMENT '部门编码（组织内唯一）',
  `name` varchar(128) NOT NULL COMMENT '部门名称',
  `full_name` varchar(512) DEFAULT NULL COMMENT '部门全称（自动生成：集团/子公司/部门/子部门）',
  `path` varchar(512) DEFAULT NULL COMMENT '部门路径（如：/1/2/3/，用于快速查询）',
  `level` int DEFAULT '1' COMMENT '层级深度',
  `leader_employee_id` bigint DEFAULT NULL COMMENT '部门负责人（员工ID）',
  `sort_order` int DEFAULT '0' COMMENT '排序',
  `status` tinyint DEFAULT '1' COMMENT '状态：0-禁用 1-启用',
  `created_by` bigint DEFAULT NULL,
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  `updated_by` bigint DEFAULT NULL,
  `updated_at` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `is_deleted` tinyint DEFAULT '0',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_org_code` (`org_id`,`code`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_org_id` (`org_id`),
  KEY `idx_parent_id` (`parent_id`),
  KEY `idx_path` (`path`)
) ENGINE=InnoDB AUTO_INCREMENT=2 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='部门表';


-- ms_team.employee definition

CREATE TABLE `employee` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `org_id` bigint NOT NULL COMMENT '所属组织ID',
  `user_id` bigint NOT NULL COMMENT '关联的用户ID（ms-identity.user.id）',
  `employee_no` varchar(64) NOT NULL COMMENT '员工工号（组织内唯一）',
  `name` varchar(64) NOT NULL COMMENT '员工姓名（可与 User.nick_name 不同）',
  `avatar` varchar(512) DEFAULT NULL COMMENT '员工头像（可与 User.avatar 不同）',
  `gender` tinyint DEFAULT '0' COMMENT '性别：0-未知 1-男 2-女',
  `mobile` varchar(20) DEFAULT NULL COMMENT '工作手机',
  `email` varchar(128) DEFAULT NULL COMMENT '工作邮箱',
  `hire_date` date DEFAULT NULL COMMENT '入职日期',
  `leave_date` date DEFAULT NULL COMMENT '离职日期',
  `status` tinyint DEFAULT '1' COMMENT '状态：0-离职 1-在职 2-试用期 3-停薪留职',
  `sort_order` int DEFAULT '0' COMMENT '排序',
  `created_by` bigint DEFAULT NULL,
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  `updated_by` bigint DEFAULT NULL,
  `updated_at` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `is_deleted` tinyint DEFAULT '0',
  `primary_dept_id` bigint DEFAULT NULL COMMENT '★ 主部门 ID，用于快速查询（复制自 employee_department）',
  `work_location_id` bigint DEFAULT NULL COMMENT '工作地点（location 表 ID）',
  `phone` varchar(20) DEFAULT NULL COMMENT '座机号码',
  `department_title` varchar(100) DEFAULT NULL COMMENT '部门内职位（如：资深工程师、技术总监）',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_org_employee_no` (`org_id`,`employee_no`),
  UNIQUE KEY `uk_org_user` (`org_id`,`user_id`) COMMENT '一个用户在一个组织只能有一个员工身份',
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_org_id` (`org_id`),
  KEY `idx_user_id` (`user_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='员工表';


-- ms_team.employee_department definition

CREATE TABLE `employee_department` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL,
  `employee_id` bigint NOT NULL COMMENT '员工ID',
  `department_id` bigint NOT NULL COMMENT '部门ID',
  `is_primary` tinyint DEFAULT '0' COMMENT '是否主部门：0-否 1-是',
  `is_leader` tinyint DEFAULT '0' COMMENT '是否部门负责人：0-否 1-是',
  `join_date` date DEFAULT NULL COMMENT '加入部门日期',
  `leave_date` date DEFAULT NULL COMMENT '离开部门日期',
  `created_by` bigint DEFAULT NULL,
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  `is_temporary` tinyint DEFAULT '0' COMMENT '★ 是否临时/借调成员（1=借调 0=常规）',
  `secondment_id` bigint DEFAULT NULL COMMENT '★ 关联借调记录 ID（Secondment 表）',
  `role` varchar(50) DEFAULT NULL COMMENT '★ 权限角色：viewer/contributor/team-lead',
  `actual_start_date` date DEFAULT NULL COMMENT '★ 实际加入日期（用于借调）',
  `actual_end_date` date DEFAULT NULL COMMENT '★ 实际离开日期（用于借调）',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_employee_dept` (`employee_id`,`department_id`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_department_id` (`department_id`),
  KEY `idx_emp_dept_primary` (`employee_id`,`is_primary`),
  KEY `idx_emp_dept_temp` (`is_temporary`,`secondment_id`),
  KEY `idx_emp_dept_leader` (`department_id`,`is_leader`),
  KEY `idx_emp_dept_dates` (`actual_start_date`,`actual_end_date`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='员工与部门关系表（支持常规和借调关系）';


-- ms_team.employee_position definition

CREATE TABLE `employee_position` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL,
  `employee_id` bigint NOT NULL COMMENT '员工ID',
  `position_id` bigint NOT NULL COMMENT '岗位ID',
  `is_primary` tinyint DEFAULT '0' COMMENT '是否主岗位：0-否 1-是',
  `start_date` date DEFAULT NULL COMMENT '任职开始日期',
  `end_date` date DEFAULT NULL COMMENT '任职结束日期',
  `created_by` bigint DEFAULT NULL,
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_employee_position` (`employee_id`,`position_id`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_position_id` (`position_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='员工-岗位关系表';


-- ms_team.location definition

CREATE TABLE `location` (
  `id` bigint NOT NULL COMMENT '地点 ID',
  `tenant_id` bigint NOT NULL COMMENT '租户 ID',
  `name` varchar(100) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '地点名称，如"北京总部"、"上海浦东"',
  `address` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '详细地址',
  `city` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '城市',
  `province` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '省份',
  `country` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '国家',
  `postal_code` varchar(20) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '邮编',
  `latitude` decimal(10,8) DEFAULT NULL COMMENT '纬度',
  `longitude` decimal(11,8) DEFAULT NULL COMMENT '经度',
  `phone` varchar(20) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '地点电话',
  `capacity` int DEFAULT NULL COMMENT '容纳人数',
  `manager_id` bigint DEFAULT NULL COMMENT '地点负责人（employee ID）',
  `status` tinyint DEFAULT '1' COMMENT '状态：1=启用 0=禁用',
  `description` varchar(500) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '地点介绍',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  `is_deleted` tinyint DEFAULT '0' COMMENT '是否删除',
  PRIMARY KEY (`id`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_city` (`city`),
  KEY `idx_status` (`status`),
  KEY `idx_coordinates` (`latitude`,`longitude`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='地点管理表（办公地点、工作地点等）';


-- ms_team.organization definition

CREATE TABLE `organization` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID（关联 ms-identity.tenant）',
  `parent_id` bigint DEFAULT NULL COMMENT '上级组织ID（支持集团-子公司结构）',
  `code` varchar(64) NOT NULL COMMENT '组织编码（租户内唯一）',
  `name` varchar(128) NOT NULL COMMENT '组织名称',
  `short_name` varchar(64) DEFAULT NULL COMMENT '简称',
  `type` tinyint DEFAULT '1' COMMENT '组织类型：1-集团 2-公司 3-分公司 4-子公司',
  `logo` varchar(512) DEFAULT NULL COMMENT '组织Logo',
  `description` text COMMENT '描述',
  `sort_order` int DEFAULT '0' COMMENT '排序',
  `status` tinyint DEFAULT '1' COMMENT '状态：0-禁用 1-启用',
  `created_by` bigint DEFAULT NULL,
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  `updated_by` bigint DEFAULT NULL,
  `updated_at` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `is_deleted` tinyint DEFAULT '0',
  `location_id` bigint DEFAULT NULL COMMENT '所在地点（location 表 ID）',
  `path` varchar(500) DEFAULT NULL COMMENT '树路径，如 /1/2/5/，用于范围查询',
  `level` int DEFAULT NULL COMMENT '树深度：1=集团 2=总公司 3=分公司 4=分支机构',
  `is_operational` tinyint DEFAULT '1' COMMENT '是否可运营部门（能否有员工）',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_tenant_code` (`tenant_id`,`code`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_parent_id` (`parent_id`)
) ENGINE=InnoDB AUTO_INCREMENT=3 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='组织表';


-- ms_team.`position` definition

CREATE TABLE `position` (
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
  `created_by` bigint DEFAULT NULL,
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  `updated_by` bigint DEFAULT NULL,
  `updated_at` datetime DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  `is_deleted` tinyint DEFAULT '0',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_org_code` (`org_id`,`code`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_org_id` (`org_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='岗位表';


-- ms_team.secondment definition

CREATE TABLE `secondment` (
  `id` bigint NOT NULL COMMENT '借调记录 ID',
  `tenant_id` bigint NOT NULL COMMENT '租户 ID',
  `employee_id` bigint NOT NULL COMMENT '员工 ID',
  `from_org_id` bigint NOT NULL COMMENT '来源组织 ID（员工法定组织）',
  `from_dept_id` bigint NOT NULL COMMENT '来源部门 ID（员工主部门）',
  `to_org_id` bigint NOT NULL COMMENT '借调到的组织 ID',
  `to_dept_id` bigint NOT NULL COMMENT '借调到的部门 ID',
  `role` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL DEFAULT 'contributor' COMMENT '角色：viewer/contributor/team-lead',
  `status` tinyint NOT NULL DEFAULT '1' COMMENT '状态：1=生效 0=已结束 -1=已撤销',
  `start_date` date NOT NULL COMMENT '借调开始日期',
  `end_date` date DEFAULT NULL COMMENT '借调结束日期（NULL=长期）',
  `reason` varchar(500) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '借调原因',
  `approval_status` tinyint DEFAULT '0' COMMENT '审批状态：0=待审批 1=已批准 -1=已驳回',
  `approved_by` bigint DEFAULT NULL COMMENT '审批人',
  `approved_at` timestamp NULL DEFAULT NULL COMMENT '批准时间',
  `rejected_reason` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '驳回原因',
  `created_by` bigint NOT NULL COMMENT '申请人',
  `created_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP COMMENT '申请时间',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
  `comment` varchar(500) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '备注',
  `is_deleted` tinyint DEFAULT '0' COMMENT '是否删除',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_emp_org_temp` (`employee_id`,`to_org_id`,`end_date`),
  KEY `idx_employee` (`employee_id`),
  KEY `idx_org_from` (`from_org_id`),
  KEY `idx_org_to` (`to_org_id`),
  KEY `idx_dept_from` (`from_dept_id`),
  KEY `idx_dept_to` (`to_dept_id`),
  KEY `idx_status` (`status`,`end_date`),
  KEY `idx_approval` (`approval_status`),
  KEY `idx_date_range` (`start_date`,`end_date`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='员工借调关系表（支持跨组织临时调动）';