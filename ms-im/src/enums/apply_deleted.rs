/// 申请删除状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApplyDeletedEnum {
    /// 未删除
    Normal = 0,
    /// 申请人删除
    ApplyDeleted = 1,
    /// 被申请人删除
    TargetDeleted = 2,
    /// 双方都删除了
    AllDeleted = 3,
}

impl ApplyDeletedEnum {
    /// 获取代码值
    pub fn as_i32(&self) -> i32 {
        *self as i32
    }

    /// 获取描述
    pub fn desc(&self) -> &'static str {
        match self {
            ApplyDeletedEnum::Normal => "未删除",
            ApplyDeletedEnum::ApplyDeleted => "申请人删除",
            ApplyDeletedEnum::TargetDeleted => "被申请人删除",
            ApplyDeletedEnum::AllDeleted => "双方都删除了",
        }
    }

    /// 申请方已经删除
    ///
    /// # 返回
    /// 返回申请方已删除的状态代码列表
    pub fn apply_deleted() -> Vec<i32> {
        vec![
            ApplyDeletedEnum::AllDeleted.as_i32(),
            ApplyDeletedEnum::ApplyDeleted.as_i32(),
        ]
    }

    /// 被申请方已经删除
    ///
    /// # 返回
    /// 返回被申请方已删除的状态代码列表
    pub fn target_deleted() -> Vec<i32> {
        vec![
            ApplyDeletedEnum::AllDeleted.as_i32(),
            ApplyDeletedEnum::TargetDeleted.as_i32(),
        ]
    }
}
