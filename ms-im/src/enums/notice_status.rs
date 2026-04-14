/// 事件处理的枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 通知状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoticeStatusEnum {
    /// 待审批
    Untreated = 0,
    /// 已同意
    Accepted = 1,
    /// 已拒绝
    Rejected = 2,
    /// 已忽略
    Ignore = 3,
}

impl NoticeStatusEnum {
    /// 获取状态值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            NoticeStatusEnum::Untreated => "待审批",
            NoticeStatusEnum::Accepted => "已同意",
            NoticeStatusEnum::Rejected => "已拒绝",
            NoticeStatusEnum::Ignore => "已忽略",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, NoticeStatusEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0, NoticeStatusEnum::Untreated);
    map.insert(1, NoticeStatusEnum::Accepted);
    map.insert(2, NoticeStatusEnum::Rejected);
    map.insert(3, NoticeStatusEnum::Ignore);
    map
});

impl NoticeStatusEnum {
    /// 根据当前枚举的 name 匹配
    pub fn match_val(val: i32) -> Self {
        CACHE.get(&val).copied().unwrap_or(NoticeStatusEnum::Accepted)
    }

    /// 获取枚举（别名方法）
    pub fn get(val: i32) -> Self {
        Self::match_val(val)
    }
}

