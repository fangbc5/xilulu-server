/// 成员角色枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 群成员角色
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GroupRoleEnum {
    /// 群主
    Leader = 1,
    /// 管理
    Manager = 2,
    /// 普通成员
    Member = 3,
}

impl GroupRoleEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            GroupRoleEnum::Leader => "群主",
            GroupRoleEnum::Manager => "管理",
            GroupRoleEnum::Member => "普通成员",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, GroupRoleEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, GroupRoleEnum::Leader);
    map.insert(2, GroupRoleEnum::Manager);
    map.insert(3, GroupRoleEnum::Member);
    map
});

impl GroupRoleEnum {
    /// 根据类型值获取枚举
    pub fn of(type_val: i32) -> Option<Self> {
        CACHE.get(&type_val).copied()
    }

    /// 返回角色的名称
    ///
    /// # 参数
    /// - `type_val`: 传入成员类型
    pub fn get(type_val: i32) -> Option<&'static str> {
        if type_val > GroupRoleEnum::Manager.as_i32() {
            return None;
        }
        Self::of(type_val).map(|e| e.desc())
    }
}

