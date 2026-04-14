use fbc_starter::Config as BaseConfig;
use serde::{Deserialize, Serialize};

/// IM 服务配置
/// 扩展 fbc-starter 的基础配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImConfig {
    /// 基础配置（继承自 fbc-starter）
    #[serde(flatten)]
    pub base: BaseConfig,
    /// IM 业务配置
    #[serde(default)]
    pub im: ImServiceConfig,
}

/// IM 业务配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ImServiceConfig {
    /// 群聊最大成员数
    #[serde(default = "default_max_group_members")]
    pub max_group_members: usize,
    /// 好友申请过期时间（秒）
    #[serde(default = "default_apply_expire_secs")]
    pub apply_expire_secs: u64,
}

fn default_max_group_members() -> usize {
    500
}

fn default_apply_expire_secs() -> u64 {
    604800 // 7天
}

impl ImConfig {
    /// 从环境变量加载配置
    #[allow(dead_code)]
    pub fn new(base_config: BaseConfig) -> Self {
        let im_config = ImServiceConfig {
            max_group_members: std::env::var("APP__IM__MAX_GROUP_MEMBERS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_max_group_members),
            apply_expire_secs: std::env::var("APP__IM__APPLY_EXPIRE_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_apply_expire_secs),
        };

        Self {
            base: base_config,
            im: im_config,
        }
    }
}
