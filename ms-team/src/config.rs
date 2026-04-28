use fbc_starter::Config as BaseConfig;
use serde::{Deserialize, Serialize};

/// ms-team 服务配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationConfig {
    /// 基础配置（继承自 fbc-starter）
    #[serde(flatten)]
    pub base: BaseConfig,
    /// 组织服务配置
    #[serde(default)]
    pub organization: OrganizationServiceConfig,
    /// Meilisearch 配置
    #[serde(default)]
    pub meilisearch: MeilisearchConfig,
    /// 通讯录配置
    #[serde(default)]
    pub contacts: ContactsConfig,
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

/// Meilisearch 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeilisearchConfig {
    /// Meilisearch 服务地址
    pub url: String,
    /// Meilisearch API Key
    pub api_key: String,
}

impl Default for MeilisearchConfig {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:7700".to_string(),
            api_key: String::new(),
        }
    }
}

/// 通讯录业务限制配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactsConfig {
    /// 单个部门下最多可创建的子部门数
    #[serde(default = "default_max_sub_departments")]
    pub max_sub_departments: u32,
    /// 单个部门最多可挂靠的直属成员数
    #[serde(default = "default_max_dept_members")]
    pub max_dept_members: u32,
    /// 部门树最大层级深度
    #[serde(default = "default_max_dept_depth")]
    pub max_dept_depth: u32,
    /// 单个组织下最多根部门数
    #[serde(default = "default_max_org_root_depts")]
    pub max_org_root_depts: u32,
    /// 部门展开时默认加载的直属成员数
    #[serde(default = "default_dept_preview_members")]
    pub dept_preview_members: u32,
    /// include_children=true 时最多返回的成员数
    #[serde(default = "default_include_children_max")]
    pub include_children_max: u32,
}

impl Default for ContactsConfig {
    fn default() -> Self {
        Self {
            max_sub_departments: default_max_sub_departments(),
            max_dept_members: default_max_dept_members(),
            max_dept_depth: default_max_dept_depth(),
            max_org_root_depts: default_max_org_root_depts(),
            dept_preview_members: default_dept_preview_members(),
            include_children_max: default_include_children_max(),
        }
    }
}

fn default_page_size() -> u32 {
    20
}

fn default_max_page_size() -> u32 {
    100
}

fn default_max_dept_depth() -> u32 {
    8
}

fn default_max_sub_departments() -> u32 {
    8
}

fn default_max_dept_members() -> u32 {
    200
}

fn default_max_org_root_depts() -> u32 {
    20
}

fn default_dept_preview_members() -> u32 {
    20
}

fn default_include_children_max() -> u32 {
    2000
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

        let meilisearch_config = MeilisearchConfig {
            url: std::env::var("APP__TEAM__MEILISEARCH_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:7700".to_string()),
            api_key: std::env::var("APP__TEAM__MEILISEARCH_API_KEY")
                .unwrap_or_default(),
        };

        let contacts_config = ContactsConfig {
            max_sub_departments: std::env::var("APP__TEAM__CONTACTS__MAX_SUB_DEPARTMENTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_max_sub_departments),
            max_dept_members: std::env::var("APP__TEAM__CONTACTS__MAX_DEPT_MEMBERS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_max_dept_members),
            max_dept_depth: std::env::var("APP__TEAM__CONTACTS__MAX_DEPT_DEPTH")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_max_dept_depth),
            max_org_root_depts: std::env::var("APP__TEAM__CONTACTS__MAX_ORG_ROOT_DEPTS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_max_org_root_depts),
            dept_preview_members: std::env::var("APP__TEAM__CONTACTS__DEPT_PREVIEW_MEMBERS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_dept_preview_members),
            include_children_max: std::env::var("APP__TEAM__CONTACTS__INCLUDE_CHILDREN_MAX")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or_else(default_include_children_max),
        };

        Ok(Self {
            base: base_config,
            organization: organization_config,
            meilisearch: meilisearch_config,
            contacts: contacts_config,
        })
    }
}
