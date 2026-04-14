/// 角色枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 角色类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoleTypeEnum {
    /// 超级管理员
    Admin = 1,
    /// HuLa群聊管理
    ChatManager = 2,
}

impl RoleTypeEnum {
    /// 获取 ID
    pub fn as_i64(&self) -> i64 {
        *self as i64
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            RoleTypeEnum::Admin => "超级管理员",
            RoleTypeEnum::ChatManager => "HuLa群聊管理",
        }
    }
}

static CACHE: LazyLock<HashMap<i64, RoleTypeEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, RoleTypeEnum::Admin);
    map.insert(2, RoleTypeEnum::ChatManager);
    map
});

impl RoleTypeEnum {
    /// 根据类型值获取枚举
    pub fn of(type_val: i64) -> Option<Self> {
        CACHE.get(&type_val).copied()
    }
}

