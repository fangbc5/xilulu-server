/// 申请阅读状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApplyReadStatusEnum {
    /// 未读
    Unread = 1,
    /// 已读
    Read = 2,
}

impl ApplyReadStatusEnum {
    /// 获取代码值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            ApplyReadStatusEnum::Unread => "未读",
            ApplyReadStatusEnum::Read => "已读",
        }
    }
}
