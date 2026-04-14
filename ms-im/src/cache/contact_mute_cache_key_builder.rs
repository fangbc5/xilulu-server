use fbc_starter::cache::{CacheKeyBuilder, ValueType};
use std::time::Duration;

pub struct ContactMuteCacheKeyBuilder;

impl CacheKeyBuilder for ContactMuteCacheKeyBuilder {
    fn get_modular(&self) -> Option<&str> {
        Some("chat")
    }

    fn get_table(&self) -> &str {
        "contact"
    }

    fn get_field(&self) -> Option<&str> {
        Some("is_mute")
    }

    fn get_value_type(&self) -> ValueType {
        ValueType::String
    }

    fn get_expire(&self) -> Option<Duration> {
        Some(Duration::from_secs(86400 * 7)) // 默认七天过期
    }
}
