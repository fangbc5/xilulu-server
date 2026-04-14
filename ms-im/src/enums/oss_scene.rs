/// 场景枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// OSS 场景
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OssSceneEnum {
    /// 聊天
    Chat,
    /// 表情包
    Emoji,
    /// 头像
    Avatar,
}

impl OssSceneEnum {
    /// 获取类型值
    pub fn as_str(&self) -> &'static str {
        match self {
            OssSceneEnum::Chat => "chat",
            OssSceneEnum::Emoji => "emoji",
            OssSceneEnum::Avatar => "avatar",
        }
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            OssSceneEnum::Chat => "聊天",
            OssSceneEnum::Emoji => "表情包",
            OssSceneEnum::Avatar => "头像",
        }
    }

    /// 获取路径
    pub fn path(&self) -> &'static str {
        match self {
            OssSceneEnum::Chat => "/chat",
            OssSceneEnum::Emoji => "/emoji",
            OssSceneEnum::Avatar => "/avatar",
        }
    }
}

static CACHE: LazyLock<HashMap<&'static str, OssSceneEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert("chat", OssSceneEnum::Chat);
    map.insert("emoji", OssSceneEnum::Emoji);
    map.insert("avatar", OssSceneEnum::Avatar);
    map
});

impl OssSceneEnum {
    /// 根据类型值获取枚举
    pub fn of(type_val: &str) -> Option<Self> {
        CACHE.get(type_val).copied()
    }
}

