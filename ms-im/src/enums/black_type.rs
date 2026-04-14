/// 黑名单类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlackTypeEnum {
    /// IP
    Ip = 1,
    /// UID
    Uid = 2,
}

impl BlackTypeEnum {
    /// 获取类型值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }
}
