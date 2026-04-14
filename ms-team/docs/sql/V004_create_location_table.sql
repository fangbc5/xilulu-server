-- ============================================================================
-- Version: V004
-- Date: 2026-02-10
-- Author: AI Assistant
-- Description: 创建 Location 表（地点管理）
-- Changes:
--   - 新建 location 表，用于管理办公地点、工作地点
-- ============================================================================

-- 创建 Location 表
CREATE TABLE IF NOT EXISTS location (
    id BIGINT PRIMARY KEY COMMENT '地点 ID',
    tenant_id BIGINT NOT NULL COMMENT '租户 ID',
    name VARCHAR(100) NOT NULL COMMENT '地点名称，如"北京总部"、"上海浦东"',
    address VARCHAR(255) COMMENT '详细地址',
    city VARCHAR(50) COMMENT '城市',
    province VARCHAR(50) COMMENT '省份',
    country VARCHAR(50) COMMENT '国家',
    postal_code VARCHAR(20) COMMENT '邮编',
    latitude DECIMAL(10, 8) COMMENT '纬度',
    longitude DECIMAL(11, 8) COMMENT '经度',
    phone VARCHAR(20) COMMENT '地点电话',
    capacity INT COMMENT '容纳人数',
    manager_id BIGINT COMMENT '地点负责人（employee ID）',
    status TINYINT DEFAULT 1 COMMENT '状态：1=启用 0=禁用',
    description VARCHAR(500) COMMENT '地点介绍',
    created_by BIGINT COMMENT '创建人',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP COMMENT '创建时间',
    updated_by BIGINT COMMENT '更新人',
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP COMMENT '更新时间',
    is_deleted TINYINT DEFAULT 0 COMMENT '是否删除',
    
    KEY idx_tenant_id (tenant_id),
    KEY idx_city (city),
    KEY idx_status (status),
    KEY idx_coordinates (latitude, longitude)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci 
COMMENT='地点管理表（办公地点、工作地点等）';

-- ============================================================================
-- 使用示例：
-- INSERT INTO location VALUES (
--   1, 1, '深圳总部', '深圳市南山区科技园', '深圳', '广东', '中国', '518000',
--   22.5344, 113.9287, '0755-88881234', 500, 999, 1, '深圳总部大楼', 1, NOW(), NULL, NOW(), 0
-- );
-- ============================================================================

COMMIT;