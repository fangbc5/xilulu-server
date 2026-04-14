use serde::{Deserialize, Serialize};

/// 系统级内置租户 ID 枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SystemTenant {
    /// 系统超管租户 (ID: 1)
    SuperAdmin = 1,
    /// 系统默认个人租户 (ID: 2)
    PersonalDefault = 2,
    /// 系统默认企业租户 (ID: 3)
    EnterpriseDefault = 3,
}

impl SystemTenant {
    /// 转换为数据库中实际的 i64 租户 ID
    pub fn id(&self) -> i64 {
        *self as i64
    }
}
