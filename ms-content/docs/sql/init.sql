-- ============================================
-- ms-content 内容管理内核 — 数据库初始化脚本
-- Phase 1：5 张核心表
-- ============================================

CREATE DATABASE IF NOT EXISTS `ms_content` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_0900_ai_ci;

USE `ms_content`;

-- ============================================
-- 1. 内容类型 Schema 注册表
-- 为每种 content_type 的 ext_data 定义合法结构
-- Service 层写入前强制校验
-- ============================================
CREATE TABLE IF NOT EXISTS `content_schema` (
    `id`                BIGINT       NOT NULL AUTO_INCREMENT  COMMENT '主键 ID',
    `content_type`      VARCHAR(32)  NOT NULL             COMMENT '内容类型唯一标识，如 blog / product / moment',
    `display_name`      VARCHAR(64)  NOT NULL             COMMENT '类型中文名，如 博客 / 商品',
    `schema_definition` JSON         NOT NULL             COMMENT 'ext_data 的 JSON Schema 定义',
    `status`            TINYINT      NOT NULL DEFAULT 1   COMMENT '0=禁用 1=启用',
    `created_at`        BIGINT       NOT NULL DEFAULT 0   COMMENT '创建时间',
    `updated_at`        BIGINT       NOT NULL DEFAULT 0   COMMENT '更新时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_content_type` (`content_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容类型 Schema 注册表';

-- ============================================
-- 2. 内容主表（轻量路由层）
-- 仅存储路由、控制、排序所需的标量字段
-- ============================================
CREATE TABLE IF NOT EXISTS `content_main` (
    `id`            BIGINT       NOT NULL AUTO_INCREMENT  COMMENT '主键 ID',
    `content_id`    VARCHAR(36)  NOT NULL                 COMMENT '对外暴露的业务标识（UUID v4）',
    `content_type`  VARCHAR(32)  NOT NULL                 COMMENT '内容类型 → content_schema.content_type',
    `author_id`     BIGINT       NOT NULL                 COMMENT '作者/所有者 ID',
    `status`        TINYINT      NOT NULL DEFAULT 0       COMMENT '0=草稿 1=待审核 2=已发布 3=已下架 4=已删除',
    `visibility`    TINYINT      NOT NULL DEFAULT 0       COMMENT '0=公开 1=私密 2=仅关注者可见',
    `pinned`        TINYINT      NOT NULL DEFAULT 0       COMMENT '0=普通 1=置顶',
    `published_at`  BIGINT       NOT NULL DEFAULT 0       COMMENT '发布时间（用于排序）',
    `created_at`    BIGINT       NOT NULL DEFAULT 0       COMMENT '创建时间',
    `updated_at`    BIGINT       NOT NULL DEFAULT 0       COMMENT '更新时间',
    `version`       INT          NOT NULL DEFAULT 1       COMMENT '乐观锁版本号',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_content_id` (`content_id`),
    KEY `idx_author_status_pub` (`author_id`, `status`, `published_at`),
    KEY `idx_type_status_pub` (`content_type`, `status`, `published_at`),
    KEY `idx_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容主表（路由与控制）';

-- ============================================
-- 3. 内容详情表（Block DSL + 扩展）
-- 与 content_main 1:1，存储大文本内容
-- ============================================
CREATE TABLE IF NOT EXISTS `content_detail` (
    `id`            BIGINT       NOT NULL AUTO_INCREMENT  COMMENT '主键 ID',
    `content_id`    BIGINT       NOT NULL                 COMMENT '→ content_main.id',
    `title`         VARCHAR(255) DEFAULT NULL             COMMENT '标题（Moment 可为空）',
    `summary`       VARCHAR(500) DEFAULT NULL             COMMENT '摘要/简介',
    `cover_image`   VARCHAR(512) DEFAULT NULL             COMMENT '封面图 OSS Key',
    `body`          JSON         NOT NULL                 COMMENT '正文 Block DSL（结构化数组）',
    `body_text`     MEDIUMTEXT   DEFAULT NULL             COMMENT '正文纯文本（用于全文检索同步，Service 层自动提取）',
    `ext_data`      JSON         DEFAULT NULL             COMMENT '类型专属扩展字段（写入前必须通过 Schema 校验）',
    `word_count`    INT          DEFAULT 0                COMMENT '字数统计',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_content_detail_id` (`content_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容详情表（Block DSL + 扩展）';

-- ============================================
-- 4. 内容统计计数表
-- 与主表分离，避免高频更新引发行锁竞争
-- ============================================
CREATE TABLE IF NOT EXISTS `content_stats` (
    `id`            BIGINT   NOT NULL AUTO_INCREMENT      COMMENT '主键 ID',
    `content_id`    BIGINT   NOT NULL                     COMMENT '→ content_main.id',
    `view_count`    BIGINT   NOT NULL DEFAULT 0           COMMENT '浏览量',
    `like_count`    INT      NOT NULL DEFAULT 0           COMMENT '点赞数',
    `comment_count` INT      NOT NULL DEFAULT 0           COMMENT '评论数',
    `share_count`   INT      NOT NULL DEFAULT 0           COMMENT '分享数',
    `collect_count` INT      NOT NULL DEFAULT 0           COMMENT '收藏数',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_content_stats_id` (`content_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容统计计数表';

-- ============================================
-- 5. 内容关系图表
-- 图模型：评论、回复、引用、挂载、合集
-- ============================================
CREATE TABLE IF NOT EXISTS `content_relation` (
    `id`            BIGINT       NOT NULL AUTO_INCREMENT  COMMENT '主键 ID',
    `relation_id`   VARCHAR(36)  NOT NULL                 COMMENT '对外暴露的业务标识（UUID v4）',
    `source_id`     BIGINT       NOT NULL                 COMMENT '发起方内容 ID',
    `target_id`     BIGINT       NOT NULL                 COMMENT '目标方内容 ID',
    `relation_type` VARCHAR(32)  NOT NULL                 COMMENT '关系类型：comment / reply / attach / quote / collection',
    `direction`     TINYINT      NOT NULL DEFAULT 1       COMMENT '0=双向 1=单向（source→target）',
    `metadata`      JSON         DEFAULT NULL             COMMENT '边属性（置顶参数、排序权重等）',
    `created_at`    BIGINT       NOT NULL DEFAULT 0       COMMENT '创建时间',
    PRIMARY KEY (`id`),
    KEY `idx_source` (`source_id`, `relation_type`),
    KEY `idx_target` (`target_id`, `relation_type`),
    UNIQUE KEY `uk_relation_id` (`relation_id`),
    UNIQUE KEY `uk_relation_logic` (`source_id`, `target_id`, `relation_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容关系图表';

-- ============================================
-- 初始化 Schema 数据（基础内容类型）
-- ============================================
INSERT INTO `content_schema` (`content_type`, `display_name`, `schema_definition`, `created_at`, `updated_at`) VALUES
('blog', '博客', '{"type":"object","properties":{"tags":{"type":"array","items":{"type":"string"}},"reading_time":{"type":"integer"},"is_original":{"type":"boolean"}},"required":["tags"]}', UNIX_TIMESTAMP() * 1000, UNIX_TIMESTAMP() * 1000),
('moment', '动态', '{"type":"object","properties":{"location":{"type":"object","properties":{"name":{"type":"string"},"lat":{"type":"number"},"lng":{"type":"number"}}}}}', UNIX_TIMESTAMP() * 1000, UNIX_TIMESTAMP() * 1000),
('note', '笔记', '{"type":"object","properties":{"bgm":{"type":"object","properties":{"id":{"type":"string"},"name":{"type":"string"}}},"topic":{"type":"string"}}}', UNIX_TIMESTAMP() * 1000, UNIX_TIMESTAMP() * 1000),
('product', '商品', '{"type":"object","properties":{"price":{"type":"integer"},"currency":{"type":"string"},"stock":{"type":"integer"},"brand":{"type":"string"},"specs":{"type":"object"}},"required":["price","currency"]}', UNIX_TIMESTAMP() * 1000, UNIX_TIMESTAMP() * 1000),
('post', '帖子', '{"type":"object","properties":{"tags":{"type":"array","items":{"type":"string"}},"category":{"type":"string"}}}', UNIX_TIMESTAMP() * 1000, UNIX_TIMESTAMP() * 1000);
