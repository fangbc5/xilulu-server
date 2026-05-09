use `ms_identity`

-- ms_identity.application definition

CREATE TABLE `application` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT 'ID',
  `app_key` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '应用标识',
  `app_secret` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '应用秘钥',
  `name` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL DEFAULT '' COMMENT '应用名称',
  `version` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '版本',
  `type` char(2) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL DEFAULT '10' COMMENT '应用类型;[10-自建应用 20-第三方应用]@Echo(api = EchoApi.DICTIONARY_ITEM_FEIGN_CLASS, dictType = EchoDictType.System.APPLICATION_TYPE)',
  `redirect` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT NULL COMMENT '重定向地址',
  `introduce` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '简介',
  `remark` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '备注',
  `url` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '应用地址',
  `is_general` bit(1) DEFAULT b'0' COMMENT '是否公共应用;0-否 1-是',
  `is_visible` bit(1) DEFAULT b'1' COMMENT '是否可见;0-否 1-是',
  `sort_value` int DEFAULT '1' COMMENT '排序',
  `create_by` bigint DEFAULT NULL COMMENT '创建人',
  `create_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `update_by` bigint DEFAULT NULL COMMENT '最后更新人',
  `update_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '最后更新时间',
  `is_del` tinyint(1) NOT NULL DEFAULT '0' COMMENT '是否删除',
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `uk_application_key_is_del` (`app_key`,`is_del`)
) ENGINE=InnoDB AUTO_INCREMENT=101 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci ROW_FORMAT=DYNAMIC COMMENT='应用';

INSERT INTO ms_identity.application
(id, app_key, app_secret, name, version, `type`, redirect, introduce, remark, url, is_general, is_visible, sort_value, create_by, create_time, update_by, update_time, is_del)
VALUES(1, 'ms-identity-admin', '', '运营后台管理服务', 'v0.1.0', '10', NULL, '管理用户、租户、套餐、应用、角色、资源', 'admin用户服务权限最高', '', 0, 1, 1, NULL, '2026-01-05 13:00:37.041', NULL, '2026-04-10 03:07:28.637', 0);
INSERT INTO ms_identity.application
(id, app_key, app_secret, name, version, `type`, redirect, introduce, remark, url, is_general, is_visible, sort_value, create_by, create_time, update_by, update_time, is_del)
VALUES(2, 'xilulu-mobile', '', '嘻噜噜-手机端', 'v1.0.0', '10', NULL, '移动端应用', '', '', 1, 1, 2, 1, '2026-04-07 03:35:11.237', 1, '2026-04-10 03:07:38.013', 0);
INSERT INTO ms_identity.application
(id, app_key, app_secret, name, version, `type`, redirect, introduce, remark, url, is_general, is_visible, sort_value, create_by, create_time, update_by, update_time, is_del)
VALUES(3, 'xilulu-desktop', '', '嘻噜噜-桌面端', 'v1.0.0', '10', NULL, '桌面端应用', '', '', 1, 1, 3, 1, '2026-04-07 03:44:49.371', 1, '2026-04-10 03:07:38.081', 0);

-- ms_identity.plan definition

CREATE TABLE `plan` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT '套餐ID',
  `name` varchar(64) NOT NULL COMMENT '套餐名称，如 Free / Pro / Creator / Starter / Business / Advanced',
  `type` varchar(32) NOT NULL COMMENT '套餐类型, personal/enterprise',
  `price` varchar(32) NOT NULL DEFAULT '0.00' COMMENT '价格，单位：元',
  `billing_cycle` varchar(32) NOT NULL COMMENT '计费周期 monthly/quarterly/yearly/one_time',
  `description` varchar(255) DEFAULT NULL COMMENT '套餐描述',
  `is_active` tinyint(1) NOT NULL DEFAULT '1' COMMENT '是否可售',
  `sort_order` int NOT NULL DEFAULT '0' COMMENT '排序',
  `created_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `updated_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '更新时间',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `is_del` tinyint(1) NOT NULL DEFAULT '0' COMMENT '是否删除',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_plan_name_is_del` (`name`,`is_del`),
  KEY `idx_plan_type` (`type`)
) ENGINE=InnoDB AUTO_INCREMENT=101 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='商业化套餐定义表';

INSERT INTO ms_identity.plan
(id, name, `type`, price, billing_cycle, description, is_active, sort_order, created_at, created_by, updated_at, updated_by, is_del)
VALUES(1, 'Unlimit', 'enterprise', '999999.99', 'forever', '永久无限版， 0租户内部用户使用', 1, 0, '2026-01-05 12:53:08.168', 0, '2026-01-06 11:43:16.996', NULL, 0);
INSERT INTO ms_identity.plan
(id, name, `type`, price, billing_cycle, description, is_active, sort_order, created_at, created_by, updated_at, updated_by, is_del)
VALUES(2, 'Free', 'personal', '0.00', 'monthly', '个人免费套餐，基础功能体验', 1, 10, '2026-01-05 03:05:08.761', 0, '2026-01-06 11:43:17.003', NULL, 0);
INSERT INTO ms_identity.plan
(id, name, `type`, price, billing_cycle, description, is_active, sort_order, created_at, created_by, updated_at, updated_by, is_del)
VALUES(3, 'Pro Monthly', 'personal', '39.00', 'monthly', '个人专业版（月付）', 1, 20, '2026-01-05 03:05:08.761', 0, '2026-01-06 11:43:17.006', NULL, 0);
INSERT INTO ms_identity.plan
(id, name, `type`, price, billing_cycle, description, is_active, sort_order, created_at, created_by, updated_at, updated_by, is_del)
VALUES(4, 'Pro Yearly', 'personal', '399.00', 'yearly', '个人专业版（年付，优惠）', 1, 21, '2026-01-05 03:05:08.761', 0, '2026-01-06 11:43:17.007', NULL, 0);
INSERT INTO ms_identity.plan
(id, name, `type`, price, billing_cycle, description, is_active, sort_order, created_at, created_by, updated_at, updated_by, is_del)
VALUES(5, 'Creator Yearly', 'personal', '999.00', 'yearly', '创作者高级版，适合高频使用者', 1, 30, '2026-01-05 03:05:08.761', 0, '2026-01-06 11:43:17.009', NULL, 0);
INSERT INTO ms_identity.plan
(id, name, `type`, price, billing_cycle, description, is_active, sort_order, created_at, created_by, updated_at, updated_by, is_del)
VALUES(6, 'Starter Monthly', 'enterprise', '299.00', 'monthly', '企业入门版，适合小团队协作', 1, 40, '2026-01-05 03:05:08.761', 0, '2026-01-06 11:43:17.010', NULL, 0);
INSERT INTO ms_identity.plan
(id, name, `type`, price, billing_cycle, description, is_active, sort_order, created_at, created_by, updated_at, updated_by, is_del)
VALUES(7, 'Business Monthly', 'enterprise', '599.00', 'monthly', '企业标准版，支持完整协作与权限管理', 1, 50, '2026-01-05 03:05:08.761', 0, '2026-01-06 11:43:17.011', NULL, 0);
INSERT INTO ms_identity.plan
(id, name, `type`, price, billing_cycle, description, is_active, sort_order, created_at, created_by, updated_at, updated_by, is_del)
VALUES(8, 'Business Yearly', 'enterprise', '5999.00', 'yearly', '企业标准版（年付，优惠）', 1, 51, '2026-01-05 03:05:08.761', 0, '2026-01-06 11:43:17.012', NULL, 0);
INSERT INTO ms_identity.plan
(id, name, `type`, price, billing_cycle, description, is_active, sort_order, created_at, created_by, updated_at, updated_by, is_del)
VALUES(9, 'Advanced Yearly', 'enterprise', '12999.00', 'yearly', '企业高级版，支持高级权限、审计与定制能力', 1, 60, '2026-01-05 03:05:08.761', 0, '2026-01-06 11:43:17.013', 0, 0);
INSERT INTO ms_identity.plan
(id, name, `type`, price, billing_cycle, description, is_active, sort_order, created_at, created_by, updated_at, updated_by, is_del)
VALUES(10, 'Enterprise Custom', 'enterprise', '0.00', 'one_time', '企业定制套餐，价格与能力按需配置', 1, 70, '2026-01-05 03:05:08.761', 0, '2026-01-06 11:43:17.015', 0, 0);

-- ms_identity.plan_entitlement definition

CREATE TABLE `plan_entitlement` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT '权益ID',
  `plan_id` bigint NOT NULL COMMENT '套餐ID',
  `entitlement_key` varchar(64) NOT NULL COMMENT '权益key，如 max_user / enable_audit',
  `entitlement_value` varchar(128) NOT NULL COMMENT '权益值，如 10 / true / advanced',
  `value_type` varchar(32) NOT NULL COMMENT '权益类型 limit/boolean/enum',
  `description` varchar(255) DEFAULT NULL COMMENT '权益说明',
  `created_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `updated_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '更新时间',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `is_del` tinyint(1) NOT NULL DEFAULT '0' COMMENT '是否删除',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_plan_entitlement` (`entitlement_key`,`plan_id`,`is_del`),
  KEY `idx_entitlement_key` (`entitlement_key`)
) ENGINE=InnoDB AUTO_INCREMENT=1001 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='套餐权益表';

INSERT INTO ms_identity.plan_entitlement
(id, plan_id, entitlement_key, entitlement_value, value_type, description, created_at, created_by, updated_at, updated_by, is_del)
VALUES(1, 1, 'max_users', '999999', 'number', '最大人数无限制', '2026-01-06 07:20:18.861', 0, '2026-04-10 03:11:19.471', 0, 0);

-- ms_identity.resource definition

CREATE TABLE `resource` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT 'ID',
  `application_id` bigint NOT NULL COMMENT '应用ID;#def_application',
  `code` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '编码;唯一编码，用于区分资源',
  `name` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '名称',
  `resource_type` char(2) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL DEFAULT '20' COMMENT '类型;[20-菜单 40-按钮 50-字段 60-数据]@Echo(api = EchoApi.DICTIONARY_ITEM_FEIGN_CLASS,dictType = EchoDictType.System.RESOURCE_TYPE)菜单即左侧显示的菜单视图即隐藏的菜单(需要配置在路由中)和页面上点击后需要通过路由打开的页面功能即页面上的非视图的按钮字段即列表页或编辑页的字段接口即后台的访问接口',
  `parent_id` bigint NOT NULL COMMENT '父级ID',
  `open_with` char(2) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '01' COMMENT '打开方式;[01-组件 02-内链 03-外链]\n@Echo(api = EchoApi.DICTIONARY_ITEM_FEIGN_CLASS, dictType = EchoDictType.System.RESOURCE_OPEN_WITH)',
  `describe_` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '描述;resource_type=接口时表示接口说明',
  `path` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '地址栏路径;用于resource_type=菜单和视图和接口.resource_type=菜单和视图，表示地址栏地址, http开头表示外链, is_frame_src 为true表示在框架类打开.resource_type=接口，表示后端接口请求地址.',
  `component` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '页面路径;用于resource_type=菜单和视图. 前端页面在src/views目录下的相对地址.',
  `redirect` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '重定向;用于resource_type=菜单和视图',
  `icon` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '图标',
  `is_hidden` bit(1) DEFAULT b'0' COMMENT '是否隐藏菜单;\nresource_type=20时生效',
  `is_general` bit(1) DEFAULT b'0' COMMENT '是否公共资源;1-无需分配所有人就可以访问的',
  `state` bit(1) NOT NULL DEFAULT b'1' COMMENT '状态;[0-禁用 1-启用]',
  `sort_value` int DEFAULT '1' COMMENT '排序;默认升序',
  `sub_group` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '' COMMENT '分组',
  `field_is_secret` bit(1) DEFAULT b'0' COMMENT '是否脱敏;显示时是否需要脱敏实现 (用于resource_type=字段)',
  `field_is_edit` bit(1) DEFAULT b'1' COMMENT '是否可以编辑;是否可以编辑(用于resource_type=字段)',
  `data_scope` char(2) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT NULL COMMENT '数据范围;[01-全部 02-本单位及子级 03-本单位 04-本部门及子级 05-本部门 06-个人 07-自定义]',
  `custom_class` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT NULL COMMENT '实现类;自定义实现类全类名',
  `is_def` bit(1) DEFAULT b'0' COMMENT '是否默认',
  `tree_path` varchar(512) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '/' COMMENT '树路径',
  `tree_grade` int DEFAULT '0' COMMENT '树层级',
  `meta_json` varchar(512) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT '{}' COMMENT '元数据;菜单视图的元数据',
  `create_by` bigint DEFAULT NULL COMMENT '创建人id',
  `create_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `update_by` bigint DEFAULT NULL COMMENT '更新人id',
  `update_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '更新时间',
  `is_del` tinyint(1) NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `uk_resource_code` (`code`,`is_del`)
) ENGINE=InnoDB AUTO_INCREMENT=49 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci ROW_FORMAT=DYNAMIC COMMENT='资源';

INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(2, 1, 'identity-admin:menus:dashboard', '仪表盘', '20', 0, '01', '', '/dashboard', 'Dashboard', '', 'HomeOutline', 0, 0, 1, 1, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.081', NULL, '2026-04-10 03:43:06.022', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(3, 1, 'identity-admin:menus:users', '用户管理', '20', 0, '01', '', '/users', 'Users', '', 'PeopleOutline', 0, 0, 1, 2, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.095', NULL, '2026-04-10 03:43:06.033', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(4, 1, 'identity-admin:menus:tenants', '租户管理', '20', 0, '01', '', '/tenants', 'Tenants', '', 'BusinessOutline', 0, 0, 1, 3, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.099', NULL, '2026-04-10 03:43:06.040', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(5, 1, 'identity-admin:menus:roles', '角色管理', '20', 0, '01', '', '/roles', 'Roles', '', 'ShieldCheckmarkOutline', 0, 0, 1, 4, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.104', NULL, '2026-04-10 03:43:06.047', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(6, 1, 'identity-admin:menus:resources', '资源管理', '20', 0, '01', '', '/resources', 'Resources', '', 'DocumentTextOutline', 0, 0, 1, 5, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.106', NULL, '2026-04-10 03:43:06.050', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(7, 1, 'identity-admin:menus:applications', '应用管理', '20', 0, '01', '', '/applications', 'Applications', '', 'AppsOutline', 0, 0, 1, 6, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.109', NULL, '2026-04-10 03:43:06.054', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(8, 1, 'identity-admin:menus:plans', '套餐管理', '20', 0, '01', '', '/plans', 'Plans', '', 'CubeOutline', 0, 0, 1, 7, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.111', NULL, '2026-04-10 03:43:06.056', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(9, 1, 'identity-admin:users:search', '搜索', '40', 3, '01', '', '/api/v1/users', '', '', '', 0, 0, 1, 1, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.112', NULL, '2026-04-10 03:43:06.058', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(10, 1, 'identity-admin:users:create', '创建用户', '40', 3, '01', '', '/api/v1/users', '', '', '', 0, 0, 1, 2, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.112', NULL, '2026-04-10 03:43:06.060', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(11, 1, 'identity-admin:users:edit', '编辑', '40', 3, '01', '', '/api/v1/users', '', '', '', 0, 0, 1, 3, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.112', NULL, '2026-04-10 03:43:06.062', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(12, 1, 'identity-admin:users:delete', '删除', '40', 3, '01', '', '/api/v1/users', '', '', '', 0, 0, 1, 4, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.112', NULL, '2026-04-10 03:43:06.064', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(13, 1, 'identity-admin:users:save', '保存', '40', 3, '01', '', '/api/v1/users', '', '', '', 0, 0, 1, 5, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.118', NULL, '2026-04-10 03:43:06.066', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(14, 1, 'identity-admin:users:changePassword', '修改密码', '40', 3, '01', '', '/api/v1/users', '', '', '', 0, 0, 1, 6, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.118', NULL, '2026-04-10 03:43:06.067', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(15, 1, 'identity-admin:users:resetPassword', '重置密码', '40', 3, '01', '', '/api/v1/users', '', '', '', 0, 0, 1, 7, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.118', NULL, '2026-04-10 03:43:06.069', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(16, 1, 'identity-admin:users:addTenant', '添加租户', '40', 3, '01', '', '/api/v1/users', '', '', '', 0, 0, 1, 8, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.118', NULL, '2026-04-10 03:43:06.070', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(17, 1, 'identity-admin:users:removeTenant', '移除租户', '40', 3, '01', '', '/api/v1/users', '', '', '', 0, 0, 1, 9, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.118', NULL, '2026-04-10 03:43:06.073', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(18, 1, 'identity-admin:tenants:search', '搜索', '40', 4, '01', '', '/api/v1/tenants', '', '', '', 0, 0, 1, 1, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.120', NULL, '2026-04-10 03:43:06.075', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(19, 1, 'identity-admin:tenants:create', '创建租户', '40', 4, '01', '', '/api/v1/tenants', '', '', '', 0, 0, 1, 2, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.120', NULL, '2026-04-10 03:43:06.077', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(20, 1, 'identity-admin:tenants:edit', '编辑', '40', 4, '01', '', '/api/v1/tenants', '', '', '', 0, 0, 1, 3, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.120', NULL, '2026-04-10 03:43:06.078', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(21, 1, 'identity-admin:tenants:delete', '删除', '40', 4, '01', '', '/api/v1/tenants', '', '', '', 0, 0, 1, 4, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.120', NULL, '2026-04-10 03:43:06.084', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(22, 1, 'identity-admin:tenants:save', '保存', '40', 4, '01', '', '/api/v1/tenants', '', '', '', 0, 0, 1, 5, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.122', NULL, '2026-04-10 03:43:06.086', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(23, 1, 'identity-admin:tenants:addSubscription', '添加订阅', '40', 4, '01', '', '/api/v1/tenants', '', '', '', 0, 0, 1, 6, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.122', NULL, '2026-04-10 03:43:06.090', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(24, 1, 'identity-admin:tenants:cancelSubscription', '取消订阅', '40', 4, '01', '', '/api/v1/tenants', '', '', '', 0, 0, 1, 7, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.122', NULL, '2026-04-10 03:43:06.091', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(25, 1, 'identity-admin:tenants:addApplication', '添加应用', '40', 4, '01', '', '/api/v1/tenants', '', '', '', 0, 0, 1, 8, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.122', NULL, '2026-04-10 03:43:06.093', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(26, 1, 'identity-admin:tenants:removeApplication', '移除应用', '40', 4, '01', '', '/api/v1/tenants', '', '', '', 0, 0, 1, 9, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.122', NULL, '2026-04-10 03:43:06.094', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(27, 1, 'identity-admin:resources:search', '搜索', '40', 6, '01', '', '/api/v1/auth/resources', '', '', '', 0, 0, 1, 1, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.125', NULL, '2026-04-10 03:43:06.097', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(28, 1, 'identity-admin:resources:create', '创建资源', '40', 6, '01', '', '/api/v1/auth/resources', '', '', '', 0, 0, 1, 2, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.125', NULL, '2026-04-10 03:43:06.100', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(29, 1, 'identity-admin:resources:edit', '编辑', '40', 6, '01', '', '/api/v1/auth/resources', '', '', '', 0, 0, 1, 3, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.125', NULL, '2026-04-10 03:43:06.101', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(30, 1, 'identity-admin:resources:delete', '删除', '40', 6, '01', '', '/api/v1/auth/resources', '', '', '', 0, 0, 1, 4, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.125', NULL, '2026-04-10 03:43:06.104', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(31, 1, 'identity-admin:resources:save', '保存', '40', 6, '01', '', '/api/v1/auth/resources', '', '', '', 0, 0, 1, 5, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.127', NULL, '2026-04-10 03:43:06.106', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(32, 1, 'identity-admin:applications:search', '搜索', '40', 7, '01', '', '/api/v1/auth/applications', '', '', '', 0, 0, 1, 1, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.128', NULL, '2026-04-10 03:43:06.109', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(33, 1, 'identity-admin:applications:create', '创建应用', '40', 7, '01', '', '/api/v1/auth/applications', '', '', '', 0, 0, 1, 2, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.128', NULL, '2026-04-10 03:43:06.111', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(34, 1, 'identity-admin:applications:edit', '编辑', '40', 7, '01', '', '/api/v1/auth/applications', '', '', '', 0, 0, 1, 3, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.128', NULL, '2026-04-10 03:43:06.114', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(35, 1, 'identity-admin:applications:delete', '删除', '40', 7, '01', '', '/api/v1/auth/applications', '', '', '', 0, 0, 1, 4, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.128', NULL, '2026-04-10 03:43:06.116', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(36, 1, 'identity-admin:applications:save', '保存', '40', 7, '01', '', '/api/v1/auth/applications', '', '', '', 0, 0, 1, 5, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.130', NULL, '2026-04-10 03:43:06.118', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(37, 1, 'identity-admin:plans:search', '搜索', '40', 8, '01', '', '/api/v1/plans', '', '', '', 0, 0, 1, 1, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.133', NULL, '2026-04-10 03:43:06.119', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(38, 1, 'identity-admin:plans:create', '创建套餐', '40', 8, '01', '', '/api/v1/plans', '', '', '', 0, 0, 1, 2, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.133', NULL, '2026-04-10 03:43:06.123', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(39, 1, 'identity-admin:plans:edit', '编辑', '40', 8, '01', '', '/api/v1/plans', '', '', '', 0, 0, 1, 3, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.133', NULL, '2026-04-10 03:43:06.124', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(40, 1, 'identity-admin:plans:manageEntitlements', '管理权益', '40', 8, '01', '', '/api/v1/plans', '', '', '', 0, 0, 1, 4, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.133', NULL, '2026-04-10 03:43:06.126', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(41, 1, 'identity-admin:plans:delete', '删除', '40', 8, '01', '', '/api/v1/plans', '', '', '', 0, 0, 1, 5, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.133', NULL, '2026-04-10 03:43:06.127', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(42, 1, 'identity-admin:plans:save', '保存', '40', 8, '01', '', '/api/v1/plans', '', '', '', 0, 0, 1, 6, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.134', NULL, '2026-04-10 03:43:06.130', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(43, 1, 'identity-admin:plans:addEntitlement', '添加权益', '40', 8, '01', '', '/api/v1/plans', '', '', '', 0, 0, 1, 7, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.134', NULL, '2026-04-10 03:43:06.133', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(44, 1, 'identity-admin:plans:editEntitlement', '编辑权益', '40', 8, '01', '', '/api/v1/plans', '', '', '', 0, 0, 1, 8, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.134', NULL, '2026-04-10 03:43:06.134', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(45, 1, 'identity-admin:plans:deleteEntitlement', '删除权益', '40', 8, '01', '', '/api/v1/plans', '', '', '', 0, 0, 1, 9, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 08:17:39.134', NULL, '2026-04-10 03:43:06.136', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(48, 1, 'identity-admin:settings', '系统设置', '20', 0, '01', '系统设置', '/settings', 'Settings', '', 'SettingsOutline', 0, 0, 1, 8, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-01-07 11:59:04.678', NULL, '2026-04-10 03:47:04.039', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(49, 2, 'xilulu-mobile:menus:chat', '消息', '20', 0, '01', '底部消息菜单', '/app/chat', 'Chat', '', 'message', 0, 0, 1, 1, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-04-09 12:18:53.784', NULL, '2026-04-10 03:47:04.045', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(50, 2, 'xilulu-mobile:menus:contact', '通讯录', '20', 0, '01', '底部通讯录菜单', '/app/contact', 'Contact', '', 'contact', 0, 0, 1, 2, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-04-09 12:21:21.499', NULL, '2026-04-10 03:47:04.047', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(51, 2, 'xilulu-mobile:menus:social', '朋友圈', '20', 0, '01', '底部朋友圈菜单', '/app/social', 'Social', '', 'discover', 0, 0, 1, 3, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-04-10 03:46:51.169', NULL, '2026-04-10 03:46:51.169', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(52, 2, 'xilulu-mobile:menus:user', '我', '20', 0, '01', '底部我菜单', '/app/user', 'User', '', 'user', 0, 0, 1, 4, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-04-10 03:46:51.178', NULL, '2026-04-10 03:46:51.178', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(53, 3, 'xilulu-desktop:menus:chat', '消息', '20', 0, '01', '左侧消息菜单', '/app/chat', 'Chat', '', 'message', 0, 0, 1, 1, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-04-10 03:46:51.178', NULL, '2026-04-10 03:46:51.178', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(55, 3, 'xilulu-desktop:menus:contact', '通讯录', '20', 0, '01', '左侧通讯录菜单', '/app/contact', 'Contact', '', 'contact', 0, 0, 1, 2, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-04-10 03:50:49.359', NULL, '2026-04-10 03:52:16.084', 0);
INSERT INTO ms_identity.resource
(id, application_id, code, name, resource_type, parent_id, open_with, describe_, `path`, component, redirect, icon, is_hidden, is_general, state, sort_value, sub_group, field_is_secret, field_is_edit, data_scope, custom_class, is_def, tree_path, tree_grade, meta_json, create_by, create_time, update_by, update_time, is_del)
VALUES(56, 3, 'xilulu-desktop:menus:social', '朋友圈', '20', 0, '01', '左侧朋友圈菜单', '/app/social', 'Social', '', 'discover', 0, 0, 1, 3, '', 0, 1, NULL, NULL, 0, '/', 0, '{}', NULL, '2026-04-10 03:50:49.366', NULL, '2026-04-10 03:52:16.086', 0);

-- ms_identity.`role` definition

CREATE TABLE `role` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT 'ID',
  `category` char(2) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL DEFAULT '10' COMMENT '角色类别;[10-功能角色 20-桌面角色 30-数据角色]@Echo(api = EchoApi.DICTIONARY_ITEM_FEIGN_CLASS, dictType = EchoDictType.Base.ROLE_CATEGORY)',
  `type_` char(2) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL DEFAULT '20' COMMENT '角色类型;[10-系统角色 20-自定义角色]; \n@Echo(api = EchoApi.DICTIONARY_ITEM_FEIGN_CLASS, dictType = EchoDictType.Global.DATA_TYPE)',
  `name` varchar(50) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '名称',
  `code` varchar(20) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '编码',
  `biz_id` bigint DEFAULT NULL COMMENT '业务ID',
  `remarks` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT NULL COMMENT '备注',
  `state` bit(1) DEFAULT b'1' COMMENT '状态',
  `readonly_` bit(1) DEFAULT b'0' COMMENT '内置角色',
  `create_by` bigint DEFAULT NULL COMMENT '创建人',
  `create_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `update_by` bigint DEFAULT NULL COMMENT '更新人',
  `update_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '更新时间',
  `created_org_id` bigint DEFAULT NULL COMMENT '创建人组织',
  `is_del` tinyint(1) NOT NULL DEFAULT '0',
  `tenant_id` bigint NOT NULL,
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `uk_code` (`code`) USING BTREE
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci ROW_FORMAT=DYNAMIC COMMENT='角色';

INSERT INTO ms_identity.`role`
(id, category, type_, name, code, remarks, state, readonly_, create_by, create_time, update_by, update_time, created_org_id, is_del, tenant_id)
VALUES(1, '10', '10', '拥有者', 'owner', '拥有所有权限，包括用户管理、租户设置等', 1, 1, NULL, '2026-01-07 08:23:55.804', NULL, '2026-01-07 08:47:46.743', NULL, 0, 1);
INSERT INTO ms_identity.`role`
(id, category, type_, name, code, remarks, state, readonly_, create_by, create_time, update_by, update_time, created_org_id, is_del, tenant_id)
VALUES(2, '10', '10', '管理员', 'admin', '拥有管理权限，可以管理用户、角色、资源等', 1, 1, NULL, '2026-01-07 08:23:55.809', NULL, '2026-01-07 08:23:55.809', NULL, 0, 1);
INSERT INTO ms_identity.`role`
(id, category, type_, name, code, remarks, state, readonly_, create_by, create_time, update_by, update_time, created_org_id, is_del, tenant_id)
VALUES(3, '10', '10', '普通用户', 'member', '基础权限，只能查看和操作自己的数据', 1, 1, NULL, '2026-01-07 08:23:55.812', NULL, '2026-01-07 08:23:55.812', NULL, 0, 1);
INSERT INTO ms_identity.`role`
(id, category, type_, name, code, remarks, state, readonly_, create_by, create_time, update_by, update_time, created_org_id, is_del, tenant_id)
VALUES(4, '10', '10', '拥有者', 'owner', '拥有所有权限，包括用户管理、租户设置等', 1, 1, NULL, '2026-04-10 06:58:29.273', NULL, '2026-04-10 07:00:51.986', NULL, 0, 2);
INSERT INTO ms_identity.`role`
(id, category, type_, name, code, remarks, state, readonly_, create_by, create_time, update_by, update_time, created_org_id, is_del, tenant_id)
VALUES(5, '10', '10', '管理员', 'admin', '拥有管理权限，可以管理用户、角色、资源等', 1, 1, NULL, '2026-04-10 06:58:29.281', NULL, '2026-04-10 07:00:51.990', NULL, 0, 2);
INSERT INTO ms_identity.`role`
(id, category, type_, name, code, remarks, state, readonly_, create_by, create_time, update_by, update_time, created_org_id, is_del, tenant_id)
VALUES(6, '10', '10', '普通用户', 'member', '基础权限，只能查看和操作自己的数据', 1, 1, NULL, '2026-04-10 06:58:29.283', NULL, '2026-04-10 07:00:51.992', NULL, 0, 2);
INSERT INTO ms_identity.`role`
(id, category, type_, name, code, remarks, state, readonly_, create_by, create_time, update_by, update_time, created_org_id, is_del, tenant_id)
VALUES(7, '10', '10', '拥有者', 'owner', '拥有所有权限，包括用户管理、租户设置等', 1, 1, NULL, '2026-04-10 06:58:29.286', NULL, '2026-04-10 07:00:51.994', NULL, 0, 3);
INSERT INTO ms_identity.`role`
(id, category, type_, name, code, remarks, state, readonly_, create_by, create_time, update_by, update_time, created_org_id, is_del, tenant_id)
VALUES(8, '10', '10', '管理员', 'admin', '拥有管理权限，可以管理用户、角色、资源等', 1, 1, NULL, '2026-04-10 06:58:29.290', NULL, '2026-04-10 07:00:51.995', NULL, 0, 3);
INSERT INTO ms_identity.`role`
(id, category, type_, name, code, remarks, state, readonly_, create_by, create_time, update_by, update_time, created_org_id, is_del, tenant_id)
VALUES(9, '10', '10', '普通用户', 'member', '基础权限，只能查看和操作自己的数据', 1, 1, NULL, '2026-04-10 06:58:29.292', NULL, '2026-04-10 07:00:51.996', NULL, 0, 3);

-- ms_identity.role_resource_rel definition

CREATE TABLE `role_resource_rel` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键',
  `tenant_id` bigint NOT NULL,
  `resource_id` bigint NOT NULL COMMENT '拥有资源;#def_resource',
  `application_id` bigint NOT NULL COMMENT '所属应用;#def_application',
  `role_id` bigint NOT NULL COMMENT '所属角色;#base_role',
  `create_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `create_by` bigint DEFAULT NULL COMMENT '创建人',
  `update_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '最后更新时间',
  `update_by` bigint DEFAULT NULL COMMENT '最后更新人',
  `created_org_id` bigint DEFAULT NULL COMMENT '创建人组织',
  `is_del` tinyint(1) NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `uk_role_resource` (`resource_id`,`role_id`) USING BTREE
) ENGINE=InnoDB AUTO_INCREMENT=1001 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci ROW_FORMAT=DYNAMIC COMMENT='角色的资源';

-- 为租户1的超级管理员角色 (role_id=1) 订阅所有资源
INSERT INTO ms_identity.role_resource_rel (tenant_id, resource_id, application_id, role_id, create_time, update_time)
SELECT 1, id, application_id, 1, CURRENT_TIMESTAMP(3), CURRENT_TIMESTAMP(3) 
FROM ms_identity.resource;

-- 为租户2的个人拥有者角色 (role_id=4) 订阅 xilulu-mobile(id=2) 和 xilulu-desktop(id=3) 的所有资源
INSERT INTO ms_identity.role_resource_rel (tenant_id, resource_id, application_id, role_id, create_time, update_time)
SELECT 2, id, application_id, 4, CURRENT_TIMESTAMP(3), CURRENT_TIMESTAMP(3) 
FROM ms_identity.resource WHERE application_id IN (2, 3);

-- 为租户3的企业拥有者角色 (role_id=7) 订阅 xilulu-mobile(id=2) 和 xilulu-desktop(id=3) 的所有资源
INSERT INTO ms_identity.role_resource_rel (tenant_id, resource_id, application_id, role_id, create_time, update_time)
SELECT 3, id, application_id, 7, CURRENT_TIMESTAMP(3), CURRENT_TIMESTAMP(3) 
FROM ms_identity.resource WHERE application_id IN (2, 3);


-- ms_identity.tenant definition

CREATE TABLE `tenant` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT '租户编号',
  `name` varchar(30) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '租户名',
  `tenant_type` tinyint NOT NULL DEFAULT '1' COMMENT '租户类型: 1-个人租户, 2-团队租户',
  `contact_user_id` bigint DEFAULT NULL COMMENT '联系人的用户编号',
  `contact_name` varchar(30) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL COMMENT '联系人',
  `contact_mobile` varchar(500) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT NULL COMMENT '联系手机',
  `pid` bigint NOT NULL DEFAULT '0' COMMENT '父租户编号,用于继承父租户权限，防止rbac权限膨胀',
  `status` tinyint NOT NULL DEFAULT '0' COMMENT '租户状态（0正常 1停用）',
  `website` varchar(256) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci DEFAULT '' COMMENT '绑定域名',
  `package_id` bigint NOT NULL COMMENT '租户套餐编号',
  `expire_time` timestamp(3) NOT NULL COMMENT '过期时间',
  `account_count` int NOT NULL COMMENT '账号数量',
  `create_by` bigint DEFAULT NULL COMMENT '创建人id',
  `create_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `update_by` bigint DEFAULT NULL COMMENT '更新人id',
  `update_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '更新时间',
  `is_del` tinyint(1) NOT NULL DEFAULT '0',
  PRIMARY KEY (`id`) USING BTREE
) ENGINE=InnoDB AUTO_INCREMENT=4 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci ROW_FORMAT=DYNAMIC COMMENT='租户表';

INSERT INTO ms_identity.tenant
(id, name, contact_user_id, contact_name, contact_mobile, pid, status, website, package_id, expire_time, account_count, create_by, create_time, update_by, update_time, tenant_type, is_del)
VALUES(1, '系统超管租户', 1, 'admin', '13800138000', 0, 0, '', 1, '2038-01-19 03:14:07', 10, 0, NULL, 1, '2026-04-10 03:04:43.090', 0, 0);
INSERT INTO ms_identity.tenant
(id, name, contact_user_id, contact_name, contact_mobile, pid, status, website, package_id, expire_time, account_count, create_by, create_time, update_by, update_time, tenant_type, is_del)
VALUES(2, '系统默认个人租户', 1, 'admin', '13800138000', 0, 0, '', 2, '2038-01-01 15:59:59', 1, 1, '2026-04-10 02:57:40.216', NULL, '2026-04-10 03:04:21.974', 1, 0);
INSERT INTO ms_identity.tenant
(id, name, contact_user_id, contact_name, contact_mobile, pid, status, website, package_id, expire_time, account_count, create_by, create_time, update_by, update_time, tenant_type, is_del)
VALUES(3, '系统默认企业租户', 1, 'admin', '13800138000', 0, 0, '', 6, '2038-01-01 15:59:59', 50, 1, '2026-04-10 03:02:32.053', NULL, '2026-04-10 03:04:21.979', 2, 0);

-- ms_identity.tenant_application_rel definition

CREATE TABLE `tenant_application_rel` (
  `id` bigint NOT NULL COMMENT 'ID',
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `application_id` bigint NOT NULL COMMENT '应用ID',
  `expiration_time` timestamp(3) NULL DEFAULT NULL COMMENT '过期时间',
  `create_by` bigint DEFAULT NULL COMMENT '创建人',
  `create_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `update_by` bigint DEFAULT NULL COMMENT '最后更新人',
  `update_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '最后更新时间',
  PRIMARY KEY (`id`) USING BTREE,
  UNIQUE KEY `uk_tar_tenant_application` (`tenant_id`,`application_id`) USING BTREE
) ENGINE=InnoDB AUTO_INCREMENT=101 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci ROW_FORMAT=DYNAMIC COMMENT='租户的应用';

INSERT INTO ms_identity.tenant_application_rel
(id, tenant_id, application_id, expiration_time, create_by, create_time, update_by, update_time)
VALUES(1, 1, 1, NULL, 1, '2026-01-05 13:01:41.081', NULL, '2026-01-06 11:44:53.655');
INSERT INTO ms_identity.tenant_application_rel
(id, tenant_id, application_id, expiration_time, create_by, create_time, update_by, update_time)
VALUES(2, 1, 2, NULL, 1, '2026-04-07 06:19:09.648', NULL, '2026-04-10 03:24:01.020');
INSERT INTO ms_identity.tenant_application_rel
(id, tenant_id, application_id, expiration_time, create_by, create_time, update_by, update_time)
VALUES(3, 1, 3, NULL, 1, '2026-04-09 12:03:28.231', NULL, '2026-04-10 03:24:01.026');
INSERT INTO ms_identity.tenant_application_rel
(id, tenant_id, application_id, expiration_time, create_by, create_time, update_by, update_time)
VALUES(4, 2, 2, NULL, 1, '2026-04-10 03:24:46.831', NULL, '2026-04-10 03:24:46.831');
INSERT INTO ms_identity.tenant_application_rel
(id, tenant_id, application_id, expiration_time, create_by, create_time, update_by, update_time)
VALUES(5, 2, 3, NULL, 1, '2026-04-10 03:24:46.836', NULL, '2026-04-10 03:24:46.836');
INSERT INTO ms_identity.tenant_application_rel
(id, tenant_id, application_id, expiration_time, create_by, create_time, update_by, update_time)
VALUES(6, 3, 2, NULL, 1, '2026-04-10 03:24:46.840', NULL, '2026-04-10 03:24:46.840');
INSERT INTO ms_identity.tenant_application_rel
(id, tenant_id, application_id, expiration_time, create_by, create_time, update_by, update_time)
VALUES(7, 3, 3, NULL, 1, '2026-04-10 03:24:46.842', NULL, '2026-04-10 03:24:46.842');

-- ms_identity.tenant_subscription definition

CREATE TABLE `tenant_subscription` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT '订阅ID',
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `plan_id` bigint NOT NULL COMMENT '当前套餐ID',
  `status` varchar(32) NOT NULL COMMENT '订阅状态 active/expired/cancelled/suspended',
  `start_at` timestamp(3) NOT NULL COMMENT '订阅开始时间',
  `expire_at` timestamp(3) NOT NULL COMMENT '订阅到期时间',
  `auto_renew` tinyint(1) NOT NULL DEFAULT '0' COMMENT '是否自动续费',
  `created_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `updated_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '更新时间',
  `updated_by` bigint DEFAULT NULL COMMENT '更新人',
  `deleted_at` timestamp(3) NULL DEFAULT NULL COMMENT '软删除时间',
  PRIMARY KEY (`id`),
  KEY `idx_tenant_id` (`tenant_id`),
  KEY `idx_plan_id` (`plan_id`),
  KEY `idx_expire_at` (`expire_at`)
) ENGINE=InnoDB AUTO_INCREMENT=101 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='租户套餐订阅表';

INSERT INTO ms_identity.tenant_subscription
(id, tenant_id, plan_id, status, start_at, expire_at, auto_renew, created_at, created_by, updated_at, updated_by, deleted_at)
VALUES(1, 1, 1, 'active', '1993-09-27 16:00:00', '2038-01-18 19:14:07', 0, '2026-01-05 12:55:52.162', NULL, '2026-04-10 03:27:43.548', NULL, NULL);
INSERT INTO ms_identity.tenant_subscription
(id, tenant_id, plan_id, status, start_at, expire_at, auto_renew, created_at, created_by, updated_at, updated_by, deleted_at)
VALUES(2, 2, 2, 'active', '1993-09-27 16:00:00', '2038-01-18 19:14:07', 0, '2026-01-05 08:24:34.628', NULL, '2026-04-10 03:27:56.220', NULL, NULL);
INSERT INTO ms_identity.tenant_subscription
(id, tenant_id, plan_id, status, start_at, expire_at, auto_renew, created_at, created_by, updated_at, updated_by, deleted_at)
VALUES(3, 3, 6, 'active', '1993-09-27 16:00:00', '2038-01-18 19:14:07', 0, '2026-01-06 12:52:26.927', NULL, '2026-04-10 03:27:56.227', NULL, NULL);

-- ms_identity.tenant_usage definition

CREATE TABLE `tenant_usage` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `plan_id` bigint NOT NULL COMMENT '套餐ID',
  `entitlement_key` varchar(64) NOT NULL COMMENT '用量项标识，如 api_calls / doc_count',
  `cycle_type` varchar(32) NOT NULL COMMENT 'monthly / quarterly / yearly',
  `cycle_start` date NOT NULL COMMENT '周期开始',
  `cycle_end` date NOT NULL COMMENT '周期结束',
  `used_value` bigint NOT NULL DEFAULT '0' COMMENT '已使用量',
  `created_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  `updated_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_tenant_usage` (`tenant_id`,`entitlement_key`,`cycle_start`),
  KEY `idx_tenant_cycle` (`tenant_id`,`cycle_start`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='租户套餐用量统计表';


-- ms_identity.tenant_usage_log definition

CREATE TABLE `tenant_usage_log` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `tenant_id` bigint NOT NULL,
  `entitlement_key` varchar(64) NOT NULL,
  `delta` bigint NOT NULL COMMENT '本次消耗量',
  `source` varchar(64) NOT NULL COMMENT '来源：api / job / import',
  `ref_id` varchar(128) DEFAULT NULL COMMENT '业务ID',
  `created_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  PRIMARY KEY (`id`),
  KEY `idx_tenant_entitlement` (`tenant_id`,`entitlement_key`),
  KEY `idx_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='租户用量明细日志';


-- ms_identity.tenant_user_rel definition

CREATE TABLE `tenant_user_rel` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键',
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `is_owner` tinyint NOT NULL DEFAULT '0' COMMENT '是否租户所有者',
  `status` tinyint NOT NULL DEFAULT '1' COMMENT '状态: 0-禁用 1-正常 2-待审核 3-已退出',
  `join_time` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '加入时间',
  `leave_time` timestamp(3) NULL DEFAULT NULL COMMENT '退出时间',
  `invited_by` bigint DEFAULT NULL COMMENT '邀请人用户ID',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  `created_time` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
  `updated_time` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_tenant_user` (`tenant_id`,`user_id`),
  KEY `idx_user` (`user_id`),
  KEY `idx_tenant` (`tenant_id`)
) ENGINE=InnoDB AUTO_INCREMENT=5 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci ROW_FORMAT=DYNAMIC COMMENT='租户对应的用户';

INSERT INTO ms_identity.tenant_user_rel
(id, tenant_id, user_id, role_code, is_owner, status, join_time, leave_time, invited_by, created_by, created_time, updated_time)
VALUES(1, 1, 1, 'owner', 1, 1, '2026-01-04 08:17:11.194', NULL, NULL, NULL, '2026-01-04 08:17:11.194', '2026-01-06 11:44:38.903');

-- ms_identity.`user` definition

CREATE TABLE `user` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `system_type` tinyint NOT NULL DEFAULT '1' COMMENT '系统类型: 1-后台登录; 2-IM系统登录',
  `user_type` tinyint DEFAULT '3' COMMENT '用户类型: 1-系统用户,2-机器人,3-普通用户',
  `username` varchar(255) DEFAULT NULL COMMENT '用户名（认证系统用）',
  `nick_name` varchar(255) DEFAULT NULL COMMENT '昵称/长名',
  `real_name` varchar(255) DEFAULT NULL COMMENT '真实姓名',
  `avatar` varchar(255) NOT NULL DEFAULT '' COMMENT '头像',
  `avatar_update_time` timestamp(3) NULL DEFAULT NULL COMMENT '头像修改时间',
  `email` varchar(255) DEFAULT NULL COMMENT '邮箱',
  `region` varchar(5) DEFAULT NULL COMMENT '国家码',
  `mobile` varchar(11) DEFAULT NULL COMMENT '手机号',
  `id_card` varchar(18) DEFAULT NULL COMMENT '身份证',
  `wx_open_id` varchar(255) DEFAULT NULL COMMENT '微信OpenId',
  `dd_open_id` varchar(255) DEFAULT NULL COMMENT '钉钉OpenId',
  `sex` tinyint DEFAULT '0' COMMENT '性别 1-男 2-女 3-未知',
  `state` tinyint DEFAULT '1' COMMENT '状态: 0-禁用/拉黑, 1-启用/正常',
  `user_state_id` bigint DEFAULT NULL COMMENT '用户状态ID (IM用)',
  `resume` varchar(200) DEFAULT NULL COMMENT '个人简介',
  `work_describe` varchar(255) DEFAULT NULL COMMENT '工作描述',
  `item_id` bigint DEFAULT NULL COMMENT '徽章ID',
  `context` tinyint DEFAULT '0' COMMENT 'AI上下文开关',
  `num` bigint DEFAULT '10' COMMENT 'AI模块相关字段',
  `password` varchar(128) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL DEFAULT '' COMMENT '用户密码',
  `salt` varchar(20) DEFAULT NULL COMMENT '密码盐',
  `password_error_num` int DEFAULT '0' COMMENT '密码错误次数',
  `password_error_last_time` timestamp(3) NULL DEFAULT NULL COMMENT '密码错误最后时间',
  `password_expire_time` timestamp(3) NULL DEFAULT NULL COMMENT '密码过期时间',
  `last_opt_time` timestamp(3) NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '最后上下线时间',
  `last_login_time` timestamp(3) NULL DEFAULT NULL COMMENT '最后登录时间',
  `ip_info` json DEFAULT NULL COMMENT 'IP信息',
  `create_time` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `update_time` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '更新时间',
  `create_by` bigint DEFAULT '1' COMMENT '创建人ID',
  `update_by` bigint DEFAULT NULL COMMENT '更新人ID',
  `is_del` tinyint(1) NOT NULL DEFAULT '0' COMMENT '是否删除',
  `readonly` tinyint(1) DEFAULT '0' COMMENT '内置用户标记',
  PRIMARY KEY (`id`),
  UNIQUE KEY `uk_username_system` (`system_type`,`username`),
  UNIQUE KEY `uk_email` (`email`),
  UNIQUE KEY `uk_mobile_system` (`system_type`,`mobile`),
  UNIQUE KEY `uk_id_card` (`id_card`),
  KEY `idx_create_time` (`create_time`),
  KEY `idx_update_time` (`update_time`),
  KEY `idx_last_opt_time` (`last_opt_time`)
) ENGINE=InnoDB AUTO_INCREMENT=101 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='用户表';

INSERT INTO ms_identity.`user`
(id, system_type, user_type, username, nick_name, real_name, avatar, avatar_update_time, email, region, mobile, id_card, wx_open_id, dd_open_id, sex, state, user_state_id, resume, work_describe, item_id, context, num, password, salt, password_error_num, password_error_last_time, password_expire_time, last_opt_time, last_login_time, ip_info, create_time, update_time, create_by, update_by, is_del, readonly)
VALUES(1, 1, 1, 'admin', '超级管理员', '超级管理员', '', NULL, 'admin@123456.com', NULL, '18888888888', NULL, NULL, NULL, 0, 1, NULL, NULL, NULL, NULL, 0, 10, '$argon2id$v=19$m=19456,t=2,p=1$kOP9fpxYPJafWCj2WkrAKg$EuM1vSc3UFhpzLFFFHf46KJ3PieTAOuoB7DX7CZGoaM', NULL, 0, '2026-04-09 11:52:08.336', NULL, '2026-04-10 02:44:54.160', '2026-04-10 02:44:54.160', NULL, '2026-01-04 07:36:16.466', '2026-04-10 02:44:54.160', 0, 47, 0, 1);

-- ms_identity.user_role definition

CREATE TABLE `user_role` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `user_id` bigint NOT NULL COMMENT '用户ID',
  `role_id` bigint NOT NULL COMMENT '角色ID',
  `role_code` varchar(20) NOT NULL COMMENT '角色编码',
  `tenant_id` bigint NOT NULL COMMENT '租户ID',
  `created_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `created_by` bigint DEFAULT NULL COMMENT '创建人',
  PRIMARY KEY (`id`),
  UNIQUE KEY `idx_user_role_tenant` (`user_id`,`role_id`,`tenant_id`)
) ENGINE=InnoDB AUTO_INCREMENT=101 DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='用户角色表';

INSERT INTO ms_identity.user_role
(id, user_id, role_id, role_code, tenant_id, created_at, created_by)
VALUES(1, 1, 1, 'owner', 1, '2026-01-09 06:37:04.153', 1);
-- ms_identity.user_device definition

CREATE TABLE `user_device` (
  `id` bigint NOT NULL AUTO_INCREMENT COMMENT '主键ID',
  `uid` bigint NOT NULL COMMENT '用户ID',
  `client_id` varchar(64) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '设备指纹',
  `device_token` varchar(512) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '推送Token',
  `platform` varchar(16) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci NOT NULL COMMENT '平台类型: ios/android',
  `app_version` varchar(32) CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci DEFAULT NULL COMMENT '客户端版本号',
  `is_active` smallint NOT NULL DEFAULT '1' COMMENT '是否有效: 1=有效 0=无效',
  `created_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) COMMENT '创建时间',
  `updated_at` timestamp(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3) COMMENT '更新时间',
  PRIMARY KEY (`id`),
  KEY `idx_uid` (`uid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='用户推送设备表';