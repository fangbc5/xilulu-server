/// 房间类型枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 房间类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoomTypeEnum {
    /// 群聊
    Group = 1,
    /// 单聊
    Friend = 2,
}

impl RoomTypeEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            RoomTypeEnum::Group => "群聊",
            RoomTypeEnum::Friend => "单聊",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, RoomTypeEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, RoomTypeEnum::Group);
    map.insert(2, RoomTypeEnum::Friend);
    map
});

impl RoomTypeEnum {
    /// 根据类型值获取枚举
    pub fn of(type_val: i32) -> Option<Self> {
        CACHE.get(&type_val).copied()
    }
}
