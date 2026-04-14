/// 物品类型枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 物品类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemTypeEnum {
    /// 改名卡
    ModifyNameCard = 1,
    /// 徽章
    Badge = 2,
}

impl ItemTypeEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            ItemTypeEnum::ModifyNameCard => "改名卡",
            ItemTypeEnum::Badge => "徽章",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, ItemTypeEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, ItemTypeEnum::ModifyNameCard);
    map.insert(2, ItemTypeEnum::Badge);
    map
});

impl ItemTypeEnum {
    /// 根据类型值获取枚举
    pub fn of(type_val: i32) -> Option<Self> {
        CACHE.get(&type_val).copied()
    }
}

