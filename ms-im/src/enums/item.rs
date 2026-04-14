/// 物品枚举
use std::collections::HashMap;
use std::sync::LazyLock;

use super::item_type::ItemTypeEnum;

/// 物品
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemEnum {
    /// 改名卡
    ModifyNameCard = 1,
    /// 爆赞徽章
    LikeBadge = 2,
    /// 前十注册徽章
    RegTop10Badge = 3,
    /// 前100注册徽章
    RegTop100Badge = 4,
    /// 知识星球
    Planet = 5,
    /// 代码贡献者
    Contributor = 6,
}

impl ItemEnum {
    /// 获取 ID
    pub fn as_i64(&self) -> i64 {
        *self as i64
    }

    /// 获取类型枚举
    pub fn type_enum(&self) -> ItemTypeEnum {
        match self {
            ItemEnum::ModifyNameCard => ItemTypeEnum::ModifyNameCard,
            ItemEnum::LikeBadge
            | ItemEnum::RegTop10Badge
            | ItemEnum::RegTop100Badge
            | ItemEnum::Planet
            | ItemEnum::Contributor => ItemTypeEnum::Badge,
        }
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            ItemEnum::ModifyNameCard => "改名卡",
            ItemEnum::LikeBadge => "爆赞徽章",
            ItemEnum::RegTop10Badge => "前十注册徽章",
            ItemEnum::RegTop100Badge => "前100注册徽章",
            ItemEnum::Planet => "知识星球",
            ItemEnum::Contributor => "代码贡献者",
        }
    }
}

static CACHE: LazyLock<HashMap<i64, ItemEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, ItemEnum::ModifyNameCard);
    map.insert(2, ItemEnum::LikeBadge);
    map.insert(3, ItemEnum::RegTop10Badge);
    map.insert(4, ItemEnum::RegTop100Badge);
    map.insert(5, ItemEnum::Planet);
    map.insert(6, ItemEnum::Contributor);
    map
});

impl ItemEnum {
    /// 根据 ID 获取枚举
    pub fn of(id: i64) -> Option<Self> {
        CACHE.get(&id).copied()
    }
}
