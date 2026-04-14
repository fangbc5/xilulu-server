/// 朋友圈权限的枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 朋友圈权限
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedPermissionEnum {
    /// 私密
    Privacy,
    /// 公开
    Open,
    /// 不给谁看
    NotAnyone,
    /// 部分可见
    PartVisible,
}

impl FeedPermissionEnum {
    /// 获取类型值
    pub fn as_str(&self) -> &'static str {
        match self {
            FeedPermissionEnum::Privacy => "privacy",
            FeedPermissionEnum::Open => "open",
            FeedPermissionEnum::NotAnyone => "notAnyone",
            FeedPermissionEnum::PartVisible => "partVisible",
        }
    }

    /// 获取名称
    pub fn name(&self) -> &'static str {
        match self {
            FeedPermissionEnum::Privacy => "私密",
            FeedPermissionEnum::Open => "公开",
            FeedPermissionEnum::NotAnyone => "不给谁看",
            FeedPermissionEnum::PartVisible => "部分可见",
        }
    }
}

static CACHE: LazyLock<HashMap<&'static str, FeedPermissionEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("privacy", FeedPermissionEnum::Privacy);
    map.insert("open", FeedPermissionEnum::Open);
    map.insert("notAnyone", FeedPermissionEnum::NotAnyone);
    map.insert("partVisible", FeedPermissionEnum::PartVisible);
    map
});

impl FeedPermissionEnum {
    /// 根据当前枚举的 name 匹配
    pub fn match_val(val: &str) -> Self {
        CACHE.get(val).copied().unwrap_or(FeedPermissionEnum::Open)
    }

    /// 获取枚举（别名方法）
    pub fn get(val: &str) -> Self {
        Self::match_val(val)
    }
}
