/// 朋友圈的枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 朋友圈类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedEnum {
    /// 纯文字
    Word = 0,
    /// 图片
    Image = 1,
    /// 视频
    Video = 2,
}

impl FeedEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取名称
    pub fn name(&self) -> &'static str {
        match self {
            FeedEnum::Word => "纯文字",
            FeedEnum::Image => "图片",
            FeedEnum::Video => "视频",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, FeedEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(0, FeedEnum::Word);
    map.insert(1, FeedEnum::Image);
    map.insert(2, FeedEnum::Video);
    map
});

impl FeedEnum {
    /// 根据当前枚举的 name 匹配
    pub fn match_val(val: i32) -> Self {
        CACHE.get(&val).copied().unwrap_or(FeedEnum::Word)
    }

    /// 获取枚举（别名方法）
    pub fn get(val: i32) -> Self {
        Self::match_val(val)
    }
}
