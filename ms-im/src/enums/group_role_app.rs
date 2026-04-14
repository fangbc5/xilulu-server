/// 群角色 APP 枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 群角色 APP
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupRoleAPPEnum {
    /// 群主
    Leader = 1,
    /// 管理
    Manager = 2,
    /// 普通成员
    Member = 3,
    /// 被移除的成员
    Remove = 4,
}

impl GroupRoleAPPEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            GroupRoleAPPEnum::Leader => "群主",
            GroupRoleAPPEnum::Manager => "管理",
            GroupRoleAPPEnum::Member => "普通成员",
            GroupRoleAPPEnum::Remove => "被移除的成员",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, GroupRoleAPPEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, GroupRoleAPPEnum::Leader);
    map.insert(2, GroupRoleAPPEnum::Manager);
    map.insert(3, GroupRoleAPPEnum::Member);
    map.insert(4, GroupRoleAPPEnum::Remove);
    map
});

impl GroupRoleAPPEnum {
    /// 根据类型值获取枚举
    pub fn of(type_val: i32) -> Option<Self> {
        CACHE.get(&type_val).copied()
    }
}
