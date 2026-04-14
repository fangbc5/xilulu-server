/// 事件通知的枚举
use std::collections::HashMap;
use std::sync::LazyLock;

/// 通知类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NoticeTypeEnum {
    /// 好友申请
    FriendApply = 1,
    /// 好友被申请
    AddMe = 6,
    /// 加群申请
    GroupApply = 2,
    /// 群邀请
    GroupInvite = 3,
    /// 移除群成员
    GroupMemberDelete = 5,
    /// 被邀请进群
    GroupInviteMe = 7,
    /// 设置群管理员
    GroupSetAdmin = 8,
    /// 取消群管理员
    GroupRecallAdmin = 9,
}

impl NoticeTypeEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            NoticeTypeEnum::FriendApply => "好友申请",
            NoticeTypeEnum::AddMe => "好友被申请",
            NoticeTypeEnum::GroupApply => "加群申请",
            NoticeTypeEnum::GroupInvite => "群邀请",
            NoticeTypeEnum::GroupMemberDelete => "移除群成员",
            NoticeTypeEnum::GroupInviteMe => "被邀请进群",
            NoticeTypeEnum::GroupSetAdmin => "设置群管理员",
            NoticeTypeEnum::GroupRecallAdmin => "取消群管理员",
        }
    }
}

static CACHE: LazyLock<HashMap<i32, NoticeTypeEnum>> = LazyLock::new(|| {
    let mut map = HashMap::new();
    map.insert(1, NoticeTypeEnum::FriendApply);
    map.insert(6, NoticeTypeEnum::AddMe);
    map.insert(2, NoticeTypeEnum::GroupApply);
    map.insert(3, NoticeTypeEnum::GroupInvite);
    map.insert(5, NoticeTypeEnum::GroupMemberDelete);
    map.insert(7, NoticeTypeEnum::GroupInviteMe);
    map.insert(8, NoticeTypeEnum::GroupSetAdmin);
    map.insert(9, NoticeTypeEnum::GroupRecallAdmin);
    map
});

impl NoticeTypeEnum {
    /// 根据当前枚举的 name 匹配
    pub fn match_val(val: i32) -> Self {
        CACHE.get(&val).copied().unwrap_or(NoticeTypeEnum::FriendApply)
    }

    /// 获取枚举（别名方法）
    pub fn get(val: i32) -> Self {
        Self::match_val(val)
    }
}

