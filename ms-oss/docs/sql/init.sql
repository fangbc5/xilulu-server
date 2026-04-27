-- ============================================
-- ms-oss 文件中台服务 — 数据库初始化脚本
-- ============================================

CREATE DATABASE IF NOT EXISTS `ms_oss` DEFAULT CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;

USE `ms_oss`;

-- ============================================
-- 文件元数据表
-- 记录通过 ms-oss 签名上传的所有文件信息
-- ============================================
CREATE TABLE IF NOT EXISTS `file_meta` (
    `id`            BIGINT       NOT NULL AUTO_INCREMENT  COMMENT '主键 ID',
    `file_key`      VARCHAR(512) NOT NULL                 COMMENT '对象存储 Key（唯一路径）',
    `bucket`        VARCHAR(128) NOT NULL                 COMMENT 'Bucket 名称',
    `original_name` VARCHAR(256) DEFAULT NULL             COMMENT '原始文件名',
    `content_type`  VARCHAR(128) DEFAULT NULL             COMMENT '文件 MIME 类型',
    `size`          BIGINT       DEFAULT NULL             COMMENT '文件大小（字节）',
    `scene`         VARCHAR(64)  NOT NULL                 COMMENT '业务场景（chat_image / avatar / ai_content 等）',
    `uploader_id`   BIGINT       DEFAULT NULL             COMMENT '上传者用户 ID',
    `provider`      VARCHAR(32)  NOT NULL DEFAULT 'rustfs' COMMENT 'OSS 厂商（rustfs / aliyun / tencent / aws）',
    `status`        TINYINT      NOT NULL DEFAULT 0       COMMENT '状态：0=待上传 1=已上传 2=已删除',
    `watermark`     TINYINT      NOT NULL DEFAULT 0       COMMENT '水印状态：0=无 1=已添加',
    `thumbnail_key` VARCHAR(512) DEFAULT NULL             COMMENT '缩略图 Key（如有）',
    `created_at`    BIGINT     NOT NULL DEFAULT 0 COMMENT '创建时间',
    `updated_at`    BIGINT     NOT NULL DEFAULT 0 COMMENT '更新时间',
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_bucket_file_key` (`bucket`, `file_key`),
    KEY `idx_bucket_scene` (`bucket`, `scene`),
    KEY `idx_uploader_id` (`uploader_id`),
    KEY `idx_status` (`status`),
    KEY `idx_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='文件元数据表';
