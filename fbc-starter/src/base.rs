use crate::constants::*;
use crate::utils::{deserialize_option_u32_from_str_or_num, deserialize_u32_from_str_or_num};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct R<T> {
    pub success: bool,
    pub code: i32,
    pub msg: Option<String>,
    pub data: Option<T>,
    #[serde(skip)]
    pub def_exec: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub base_version: String,
    pub timestamp: i64,
}

impl<T> R<T> {
    pub fn new(code: i32, msg: Option<String>, data: Option<T>) -> Self {
        Self {
            success: code == SUCCESS_CODE,
            code,
            msg,
            data,
            def_exec: false,
            path: String::new(),
            version: String::new(),
            base_version: String::new(),
            timestamp: chrono::Local::now().timestamp_millis(),
        }
    }

    pub fn ok() -> Self {
        Self::new(SUCCESS_CODE, Some(SUCCESS_MESSAGE.into()), None)
    }

    pub fn ok_with_data(data: T) -> Self {
        Self::new(SUCCESS_CODE, Some(SUCCESS_MESSAGE.into()), Some(data))
    }

    pub fn fail() -> Self {
        Self::new(OPERATION_EX_CODE, None, None)
    }

    pub fn fail_with_code(code: i32, msg: String) -> Self {
        Self::new(code, Some(msg.into()), None)
    }

    pub fn fail_with_message(msg: String) -> Self {
        Self::new(FAIL_CODE, Some(msg.into()), None)
    }
}

/// 游标分页基础响应
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CursorPageBaseResp<T> {
    /// 游标（下次翻页带上这参数）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<u32>,
    /// 是否有下一页
    pub has_next: bool,
    /// 数据列表
    pub list: Vec<T>,
    /// 总数
    pub total: i64,
}

impl<T> CursorPageBaseResp<T> {
    /// 初始化游标分页响应
    ///
    /// # 参数
    /// - `cursor_page`: 游标分页请求（用于获取 cursor 和 has_next）
    /// - `list`: 数据列表
    /// - `total`: 总数
    pub fn init(cursor: Option<u32>, has_next: bool, list: Vec<T>, total: i64) -> Self {
        Self {
            cursor,
            has_next,
            list,
            total,
        }
    }

    /// 判断是否为空
    pub fn is_empty(&self) -> bool {
        self.list.is_empty()
    }

    /// 创建空的分页响应
    pub fn empty() -> Self {
        Self {
            cursor: None,
            has_next: false,
            list: Vec::new(),
            total: 0,
        }
    }
}

impl<T> Default for CursorPageBaseResp<T> {
    fn default() -> Self {
        Self {
            cursor: None,
            has_next: false,
            list: Vec::new(),
            total: 0,
        }
    }
}

/// 游标翻页基础请求
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct CursorPageBaseReq {
    /// 页面大小（默认 10，最大 100）
    #[serde(
        default = "CursorPageBaseReq::default_page_size",
        deserialize_with = "deserialize_u32_from_str_or_num"
    )]
    pub page_size: u32,
    /// 游标（初始为 None，后续请求附带上次翻页的游标）
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_option_u32_from_str_or_num"
    )]
    pub cursor: Option<u32>,
}

impl CursorPageBaseReq {
    fn default_page_size() -> u32 {
        10
    }

    /// 是否是第一页（cursor 为空或 None 或者 0）
    pub fn is_first_page(&self) -> bool {
        self.cursor
            .as_ref()
            .map(|s| *s == 0 || *s < self.page_size)
            .unwrap_or(true)
    }

    /// 计算分页参数，返回 (页码, page_size)
    ///
    /// 对应 Java 中的 plusPage()，这里固定页码为 1，因为采用游标分页。
    pub fn plus_page(&self) -> (u32, u32) {
        (1, self.page_size)
    }
}

impl Default for CursorPageBaseReq {
    fn default() -> Self {
        Self {
            page_size: Self::default_page_size(),
            cursor: None,
        }
    }
}
