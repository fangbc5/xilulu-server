use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 缓存 Value 类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValueType {
    /// 对象
    Obj,
    /// 字符串
    String,
    /// 数字
    Number,
}

impl ValueType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ValueType::Obj => "obj",
            ValueType::String => "string",
            ValueType::Number => "number",
        }
    }
}

/// 缓存 key 类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheKey {
    pub key: String,
    pub expire: Option<Duration>,
}

impl CacheKey {
    pub fn new(key: String, expire: Option<Duration>) -> Self {
        Self { key, expire }
    }
}

/// Hash 缓存 key 类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheHashKey {
    pub key: String,
    pub field: Option<String>,
    pub expire: Option<Duration>,
}

impl CacheHashKey {
    pub fn new(key: String, field: Option<String>, expire: Option<Duration>) -> Self {
        Self { key, field, expire }
    }
}
