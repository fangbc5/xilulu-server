-- ============================================================
-- ms-media-processor 数据库初始化脚本
-- 版本: 1.0
-- 创建日期: 2026-04-23
-- ============================================================

-- 媒体处理任务表
CREATE TABLE IF NOT EXISTS `media_tasks` (
    `id`              VARCHAR(64)   NOT NULL            COMMENT '任务ID（UUID）',
    `source_bucket`   VARCHAR(64)   NOT NULL            COMMENT '源文件 Bucket',
    `source_key`      VARCHAR(512)  NOT NULL            COMMENT '源文件路径',
    `task_type`       VARCHAR(32)   NOT NULL            COMMENT '任务类型: VIDEO_SNAPSHOT / VIDEO_TRANSCODE / VIDEO_HLS / IMAGE_RESIZE / IMAGE_WATERMARK / AUDIO_EXTRACT',
    `parameters`      TEXT                              COMMENT '任务参数 JSON（codec、分辨率、码率等）',
    `status`          VARCHAR(20)   NOT NULL DEFAULT 'INIT'  COMMENT '任务状态: INIT / PROCESSING / DONE / FAILED',
    `priority`        TINYINT       NOT NULL DEFAULT 0  COMMENT '优先级: 0=普通, 1=高, 2=紧急',
    `retry_count`     INT           NOT NULL DEFAULT 0  COMMENT '已重试次数',
    `max_retry`       INT           NOT NULL DEFAULT 3  COMMENT '最大重试次数',
    `version`         INT           NOT NULL DEFAULT 1  COMMENT '乐观锁版本号',
    `result_key`      VARCHAR(512)                      COMMENT '主产物文件路径（如 master.m3u8）',
    `result_meta`     TEXT                              COMMENT '产物元信息 JSON（时长、格式等）',
    `error_message`   TEXT                              COMMENT '最后一次错误信息',
    `callback_topic`  VARCHAR(128)                      COMMENT '完成回调 Kafka topic',
    `created_by`      VARCHAR(64)                       COMMENT '提交方服务名',
    `created_at`      BIGINT        NOT NULL            COMMENT '创建时间（Unix 毫秒）',
    `updated_at`      BIGINT        NOT NULL            COMMENT '更新时间（Unix 毫秒）',
    PRIMARY KEY (`id`),
    INDEX `idx_status`     (`status`),
    INDEX `idx_task_type`  (`task_type`),
    INDEX `idx_priority`   (`priority`, `status`),
    INDEX `idx_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='媒体处理任务表';


-- 媒体任务产物表（一个任务可输出多个文件，如 HLS 切片）
CREATE TABLE IF NOT EXISTS `media_task_outputs` (
    `id`              BIGINT        AUTO_INCREMENT      COMMENT '自增主键',
    `task_id`         VARCHAR(64)   NOT NULL            COMMENT '关联任务ID',
    `output_key`      VARCHAR(512)  NOT NULL            COMMENT '产物 S3 路径',
    `output_type`     VARCHAR(32)                       COMMENT '产物类型: thumbnail / playlist / segment / transcode / audio',
    `content_type`    VARCHAR(64)                       COMMENT 'MIME 类型',
    `file_size`       BIGINT                            COMMENT '文件大小（字节）',
    `metadata`        TEXT                              COMMENT '额外元信息 JSON',
    `created_at`      BIGINT        NOT NULL            COMMENT '创建时间（Unix 毫秒）',
    PRIMARY KEY (`id`),
    INDEX `idx_task_id`    (`task_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='媒体任务产物表';
