/// 用户摘要信息 DTO
///
/// 对应 Java: SummeryInfoDTO
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 用户摘要信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct SummaryInfoDTO {
    /// 用户ID
    pub uid: i64,
    /// 是否需要刷新（默认 true）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub need_refresh: Option<bool>,
    /// 用户名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// 头像
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    /// 账号（Hula号）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
    /// 用户状态ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_state_id: Option<i64>,
    /// 位置（城市，归属地）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc_place: Option<String>,
    /// 佩戴的徽章ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wearing_item_id: Option<i64>,
    /// 拥有的物品ID列表（用户拥有的徽章id列表）
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub item_ids: Vec<i64>,
    /// 用户类型
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_type: Option<i32>,
    /// 邮箱（序列化时忽略，对应 Java 的 @JsonIgnore）
    #[serde(skip)]
    pub email: Option<String>,
    /// 微信openId（序列化时忽略，对应 Java 的 @JsonIgnore）
    #[serde(skip)]
    pub open_id: Option<String>,
    /// 性别 1为男性，2为女性
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sex: Option<i32>,
    /// 个人简介
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<String>,
    /// 最后操作时间（最后一次上下线时间）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_opt_time: Option<DateTime<Utc>>,
}

impl SummaryInfoDTO {
    /// 创建一个跳过刷新的 DTO
    ///
    /// 对应 Java: `SummeryInfoDTO.skip(Long uid)`
    ///
    /// # 参数
    /// - `uid`: 用户ID
    ///
    /// # 返回
    /// 返回一个 `need_refresh` 为 `false` 的 `SummaryInfoDTO`
    pub fn skip(uid: i64) -> Self {
        Self {
            uid,
            need_refresh: Some(false),
            name: None,
            avatar: None,
            account: None,
            user_state_id: None,
            loc_place: None,
            wearing_item_id: None,
            item_ids: Vec::new(),
            user_type: None,
            email: None,
            open_id: None,
            sex: None,
            resume: None,
            last_opt_time: None,
        }
    }

    /// 创建默认的 DTO（need_refresh 默认为 true）
    ///
    /// # 参数
    /// - `uid`: 用户ID
    ///
    /// # 返回
    /// 返回一个 `need_refresh` 为 `true` 的 `SummaryInfoDTO`
    pub fn new(uid: i64) -> Self {
        Self {
            uid,
            need_refresh: Some(true),
            name: None,
            avatar: None,
            account: None,
            user_state_id: None,
            loc_place: None,
            wearing_item_id: None,
            item_ids: Vec::new(),
            user_type: None,
            email: None,
            open_id: None,
            sex: None,
            resume: None,
            last_opt_time: None,
        }
    }
}

impl Default for SummaryInfoDTO {
    fn default() -> Self {
        Self {
            uid: 0,
            need_refresh: Some(true), // 默认需要刷新
            name: None,
            avatar: None,
            account: None,
            user_state_id: None,
            loc_place: None,
            wearing_item_id: None,
            item_ids: Vec::new(),
            user_type: None,
            email: None,
            open_id: None,
            sex: None,
            resume: None,
            last_opt_time: None,
        }
    }
}
