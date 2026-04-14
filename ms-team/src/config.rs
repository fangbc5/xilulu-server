use fbc_starter::Config as BaseConfig;
use serde::{Deserialize, Serialize};

/// 组织服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationConfig {
    /// 基础配置（继承自 fbc-starter）
    #[serde(flatten)]
    pub base: BaseConfig,
    /// 组织服务配置
    #[serde(default)]
    pub organization: OrganizationServiceConfig,
}

/// 组织服务配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OrganizationServiceConfig {
    /// 默认分页大小
    #[serde(default = "default_page_size")]
    pub default_page_size: u32,
    /// 最大分页大小
    #[serde(default = "default_max_page_size")]
    pub max_page_size: u32,
    /// 部门树最大深度
    #[serde(default = "default_max_dept_depth")]
    pub max_dept_depth: u32,
}

fn default_page_size() -> u32 {
    20
}

fn default_max_page_size() -> u32 {
    100
}

fn default_max_dept_depth() -> u32 {
    10
}

impl OrganizationConfig {
    /// 从基础配置加载
    pub fn new(base_config: BaseConfig) -> Result<Self, config::ConfigError> {
        let organization_config = OrganizationServiceConfig {
            default_page_size: std::env::var("APP__ORGANIZATION__DEFAULT_PAGE_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_page_size),
            max_page_size: std::env::var("APP__ORGANIZATION__MAX_PAGE_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_max_page_size),
            max_dept_depth: std::env::var("APP__ORGANIZATION__MAX_DEPT_DEPTH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_max_dept_depth),
        };

        Ok(Self {
            base: base_config,
            organization: organization_config,
        })
    }
}
