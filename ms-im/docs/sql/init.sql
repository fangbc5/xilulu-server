-- ============================================
-- ms-im 数据库初始化脚本
-- ============================================

USE `ms_im`;

-- 好友关系
CREATE TABLE IF NOT EXISTS `user_friend` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `uid` BIGINT NOT NULL COMMENT '用户ID',
    `friend_uid` BIGINT NOT NULL COMMENT '好友ID',
    `remark` VARCHAR(64) DEFAULT NULL COMMENT '好友备注',
    `status` TINYINT DEFAULT 1 COMMENT '1正常 2删除',
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_uid_friend` (`uid`, `friend_uid`),
    KEY `idx_uid` (`uid`)
) COMMENT='好友关系';

-- 好友申请
CREATE TABLE IF NOT EXISTS `user_apply` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `uid` BIGINT NOT NULL COMMENT '申请人ID',
    `target_id` BIGINT NOT NULL COMMENT '目标ID',
    `msg` VARCHAR(256) DEFAULT NULL COMMENT '申请消息',
    `type` TINYINT DEFAULT 1 COMMENT '1好友申请 2群申请',
    `status` TINYINT DEFAULT 0 COMMENT '0待审批 1同意 2拒绝',
    `read_status` TINYINT DEFAULT 0 COMMENT '0未读 1已读',
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_uid` (`uid`),
    KEY `idx_target_id` (`target_id`)
) COMMENT='好友申请';

-- 房间（统一单聊和群聊的抽象层）
CREATE TABLE IF NOT EXISTS `room` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `type` TINYINT NOT NULL COMMENT '1单聊 2群聊',
    `hot_flag` TINYINT DEFAULT 0 COMMENT '是否热点群',
    `last_msg_id` BIGINT DEFAULT NULL COMMENT '最新消息ID',
    `active_time` DATETIME DEFAULT CURRENT_TIMESTAMP COMMENT '最后活跃时间',
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`)
) COMMENT='房间';

-- 单聊房间扩展
CREATE TABLE IF NOT EXISTS `room_friend` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `room_id` BIGINT NOT NULL,
    `uid1` BIGINT NOT NULL COMMENT '较小的uid',
    `uid2` BIGINT NOT NULL COMMENT '较大的uid',
    `room_key` VARCHAR(64) NOT NULL COMMENT '拼接的roomKey: uid1_uid2',
    `status` TINYINT DEFAULT 1 COMMENT '1正常 2禁用',
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_room_key` (`room_key`),
    KEY `idx_room_id` (`room_id`)
) COMMENT='单聊房间';

-- 群聊房间扩展
CREATE TABLE IF NOT EXISTS `room_group` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `room_id` BIGINT NOT NULL,
    `name` VARCHAR(64) NOT NULL COMMENT '群名',
    `avatar` VARCHAR(256) DEFAULT NULL COMMENT '群头像',
    `notice` TEXT DEFAULT NULL COMMENT '群公告',
    `is_deleted` TINYINT NOT NULL DEFAULT 0 COMMENT '0正常 1已解散',
    `created_by` BIGINT NOT NULL DEFAULT 0 COMMENT '创建人UID',
    `updated_by` BIGINT NOT NULL DEFAULT 0 COMMENT '修改人UID',
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_room_id` (`room_id`)
) COMMENT='群聊房间';

-- 群成员
CREATE TABLE IF NOT EXISTS `group_member` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `group_id` BIGINT NOT NULL COMMENT 'room_group.id',
    `uid` BIGINT NOT NULL COMMENT '用户ID',
    `role` TINYINT DEFAULT 3 COMMENT '1群主 2管理员 3普通成员',
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_group_uid` (`group_id`, `uid`)
) COMMENT='群成员';

-- 会话（用户维度的房间视图）
CREATE TABLE IF NOT EXISTS `contact` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `uid` BIGINT NOT NULL,
    `room_id` BIGINT NOT NULL,
    `read_time` DATETIME DEFAULT CURRENT_TIMESTAMP COMMENT '已读到的时间',
    `active_time` DATETIME DEFAULT CURRENT_TIMESTAMP COMMENT '最后活跃时间',
    `last_msg_id` BIGINT DEFAULT NULL COMMENT '最后一条消息ID',
    `read_msg_id` BIGINT DEFAULT NULL COMMENT '最后一次已读的消息ID',
    `clear_msg_id` BIGINT DEFAULT 0 COMMENT '清空聊天记录的最后游标ID',
    `is_mute` TINYINT DEFAULT 0 COMMENT '是否免打扰',
    `is_top` TINYINT DEFAULT 0 COMMENT '是否置顶',
    `is_deleted` TINYINT DEFAULT 0 COMMENT '是否删除',
    `unread_count` INT NOT NULL DEFAULT 0 COMMENT '未读消息数',
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_uid_room` (`uid`, `room_id`),
    KEY `idx_uid` (`uid`)
) COMMENT='会话';

-- 消息
CREATE TABLE IF NOT EXISTS `message` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `room_id` BIGINT NOT NULL,
    `from_uid` BIGINT NOT NULL COMMENT '发送者',
    `content` TEXT DEFAULT NULL COMMENT '消息内容(JSON)',
    `type` TINYINT NOT NULL COMMENT '消息类型(1文本 2图片 3文件 ...)',
    `reply_msg_id` BIGINT DEFAULT NULL COMMENT '回复的消息ID',
    `status` TINYINT DEFAULT 0 COMMENT '0正常 1撤回',
    `extra` JSON DEFAULT NULL COMMENT '扩展信息',
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    `updated_at` DATETIME DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    KEY `idx_room_created` (`room_id`, `created_at`),
    KEY `idx_room_id` (`room_id`)
) COMMENT='消息';

-- 消息标记
CREATE TABLE IF NOT EXISTS `message_mark` (
    `id` BIGINT NOT NULL AUTO_INCREMENT,
    `msg_id` BIGINT NOT NULL,
    `uid` BIGINT NOT NULL COMMENT '标记的用户',
    `type` TINYINT NOT NULL COMMENT '1点赞 2举报',
    `status` TINYINT DEFAULT 0 COMMENT '0正常 1取消',
    `created_at` DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uk_msg_uid_type` (`msg_id`, `uid`, `type`)
) COMMENT='消息标记';
