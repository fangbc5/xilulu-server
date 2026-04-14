/// 消息标记动作类型枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 消息标记动作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageMarkActTypeEnum {
    /// 确认标记
    Mark = 1,
    /// 取消标记
    UnMark = 2,
}

impl MessageMarkActTypeEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            MessageMarkActTypeEnum::Mark => "确认标记",
            MessageMarkActTypeEnum::UnMark => "取消标记",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, MessageMarkActTypeEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, MessageMarkActTypeEnum::Mark);
    map.insert(2, MessageMarkActTypeEnum::UnMark);
    map
});

impl MessageMarkActTypeEnum {
    /// 根据类型值获取枚举
    pub fn of(type_val: i32) -> Option<Self> {
        CACHE.get(&type_val).copied()
    }
}
