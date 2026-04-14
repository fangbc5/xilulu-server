// Plan 模块枚举定义

/// 计费周期
///
/// 对应数据库中的 `billing_cycle` 字段:
/// - monthly
/// - quarterly
/// - yearly
/// - one_time
/// - forever
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingCycle {
    Monthly,
    Quarterly,
    Yearly,
    OneTime,
    Forever,
}



impl From<&str> for BillingCycle {
    fn from(value: &str) -> Self {
        match value {
            "monthly" => BillingCycle::Monthly,
            "quarterly" => BillingCycle::Quarterly,
            "yearly" => BillingCycle::Yearly,
            // 历史兼容：one_time / one-time 都认为是一次性套餐
            "one_time" | "one-time" => BillingCycle::OneTime,
            "forever" => BillingCycle::Forever,
            _ => BillingCycle::Monthly,
        }
    }
}

/// 订阅状态
///
/// 对应数据库中的 `status` 字段:
/// - active: 当前生效套餐（唯一）
/// - scheduled: 已预约，尚未生效（如到期升级）
/// - expired: 已过期
/// - canceled: 提前取消
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubscriptionStatus {
    Active,
    Scheduled,
    Expired,
    Canceled,
}

impl SubscriptionStatus {
    /// 将枚举转换为数据库中存储的字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            SubscriptionStatus::Active => "active",
            SubscriptionStatus::Scheduled => "scheduled",
            SubscriptionStatus::Expired => "expired",
            SubscriptionStatus::Canceled => "canceled",
        }
    }
}

impl From<&str> for SubscriptionStatus {
    fn from(value: &str) -> Self {
        match value {
            "active" => SubscriptionStatus::Active,
            "scheduled" => SubscriptionStatus::Scheduled,
            "expired" => SubscriptionStatus::Expired,
            "canceled" | "cancelled" => SubscriptionStatus::Canceled, // 兼容 cancelled 拼写
            _ => SubscriptionStatus::Active,
        }
    }
}
