-- ms-notify 初始化 SQL
-- 通知发送日志表

CREATE TABLE IF NOT EXISTS notify_log (
  id          BIGINT AUTO_INCREMENT PRIMARY KEY,
  channel     VARCHAR(32)  NOT NULL COMMENT '渠道: email/sms/im_feishu/im_dingding/im_wechat',
  sender      VARCHAR(255) DEFAULT '' COMMENT '发送者',
  receiver    VARCHAR(255) DEFAULT '' COMMENT '接收者',
  subject     VARCHAR(255) DEFAULT '' COMMENT '主题',
  body        TEXT         COMMENT '发送内容',
  status      TINYINT      NOT NULL DEFAULT 0 COMMENT '0=待发送 1=发送中 2=成功 3=失败',
  error_msg   VARCHAR(500) DEFAULT '' COMMENT '失败原因',
  retry_count INT          NOT NULL DEFAULT 0 COMMENT '重试次数',
  biz_type    VARCHAR(64)  DEFAULT '' COMMENT '业务类型',
  biz_id      VARCHAR(128) DEFAULT '' COMMENT '业务关联ID',
  created_at  DATETIME     DEFAULT CURRENT_TIMESTAMP,
  updated_at  DATETIME     DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  INDEX idx_channel_status (channel, status),
  INDEX idx_biz (biz_type, biz_id),
  INDEX idx_created_at (created_at)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COMMENT='通知发送日志';
