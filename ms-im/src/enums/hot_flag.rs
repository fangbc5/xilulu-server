/// 热点标志枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 热点标志
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HotFlagEnum {
    /// 非热点
    Not = 0,
    /// 热点
    Yes = 1,
}

impl HotFlagEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            HotFlagEnum::Not => "非热点",
            HotFlagEnum::Yes => "热点",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, HotFlagEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0, HotFlagEnum::Not);
    map.insert(1, HotFlagEnum::Yes);
    map
});

impl HotFlagEnum {
    /// 根据类型值获取枚举
    pub fn of(type_val: i32) -> Option<Self> {
        CACHE.get(&type_val).copied()
    }
}
