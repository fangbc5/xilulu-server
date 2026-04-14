pub mod batch_cache;
pub mod cache_key;
pub mod cache_key_builder;
pub mod cache_key_modular;
pub mod cache_key_table;
#[cfg(feature = "redis")]
pub mod redis;
#[cfg(feature = "redis")]
pub mod token;

pub use batch_cache::BatchCache;
#[cfg(feature = "local_cache")]
pub use batch_cache::LocalBatchCache;
#[cfg(feature = "redis")]
pub use batch_cache::RedisBatchCache;
pub use cache_key::{CacheHashKey, CacheKey, ValueType};
pub use cache_key_builder::{CacheKeyBuilder, SimpleCacheKeyBuilder};
pub use cache_key_modular::{
    get_cache_prefix, get_cache_prefix_or, set_cache_prefix, BASE, CHAT, COMMON, FILE, FRIEND,
    GATEWAY, MSG, OAUTH, PRESENCE, SYSTEM, VIDEO_CALL,
};
pub use cache_key_table::{
    base, chat, friend, oauth, presence, system, video_call, CAPTCHA, LOGIN_LOG_BROWSER,
    LOGIN_LOG_SYSTEM, LOGIN_LOG_TEN_DAY, ONLINE, PARAMETER_KEY, REGISTER_USER, TODAY_LOGIN_IV,
    TODAY_LOGIN_PV, TODAY_PV, TOKEN, TOKEN_USER_ID, TOTAL_LOGIN_IV, TOTAL_LOGIN_PV, TOTAL_PV,
};

#[cfg(feature = "redis")]
pub use redis::*;

#[cfg(feature = "redis")]
pub use token::*;
