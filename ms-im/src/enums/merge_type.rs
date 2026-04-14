/// 转发类型的枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 合并转发类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MergeTypeEnum {
    /// 单一转发
    Single = 1,
    /// 合并转发
    Merge = 2,
}

impl MergeTypeEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            MergeTypeEnum::Single => "单一转发",
            MergeTypeEnum::Merge => "合并转发",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, MergeTypeEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, MergeTypeEnum::Single);
    map.insert(2, MergeTypeEnum::Merge);
    map
});

impl MergeTypeEnum {
    /// 根据当前枚举的 name 匹配
    pub fn match_val(val: i32) -> Self {
        CACHE.get(&val).copied().unwrap_or(MergeTypeEnum::Single)
    }

    /// 获取枚举（别名方法）
    pub fn get(val: i32) -> Self {
        Self::match_val(val)
    }
}
