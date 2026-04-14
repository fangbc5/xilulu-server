// 用户租户关系角色代码枚举

use serde::{Deserialize, Serialize};

/// 用户租户关系角色代码
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoleCode {
    /// 所有者（owner）
    Owner,
    /// 管理员（admin）
    Admin,
    /// 成员（member）
    Member,
}

impl RoleCode {
    /// 获取角色代码的字符串值
    pub fn as_str(&self) -> &'static str {
        match self {
            RoleCode::Owner => "owner",
            RoleCode::Admin => "admin",
            RoleCode::Member => "member",
        }
    }

    /// 从字符串创建角色代码
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "owner" => Some(RoleCode::Owner),
            "admin" => Some(RoleCode::Admin),
            "member" => Some(RoleCode::Member),
            _ => None,
        }
    }
}

impl From<RoleCode> for String {
    fn from(role_code: RoleCode) -> Self {
        role_code.as_str().to_string()
    }
}

impl std::fmt::Display for RoleCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
