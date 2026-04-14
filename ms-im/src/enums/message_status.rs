/// 消息状态枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 消息状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageStatusEnum {
    /// 正常
    Normal = 0,
    /// 删除
    Delete = 1,
}

impl MessageStatusEnum {
    /// 获取状态值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            MessageStatusEnum::Normal => "正常",
            MessageStatusEnum::Delete => "删除",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, MessageStatusEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0, MessageStatusEnum::Normal);
    map.insert(1, MessageStatusEnum::Delete);
    map
});

impl MessageStatusEnum {
    /// 根据状态值获取枚举
    pub fn of(status: i32) -> Option<Self> {
        CACHE.get(&status).copied()
    }
}

