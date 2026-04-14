/// 缓存模块常量
///
/// 定义各种服务使用的缓存模块名称
use std::sync::OnceLock;

/// 多个服务都会使用的缓存
pub const COMMON: &str = "common";

/// 仅基础服务 base 使用的缓存
pub const BASE: &str = "base";

/// 仅消息服务 msg 使用的缓存
pub const MSG: &str = "msg";

/// 仅认证服务 oauth 使用的缓存
pub const OAUTH: &str = "oauth";

/// 聊天方面使用的缓存
pub const CHAT: &str = "chat";

/// 仅文件服务 file 使用的缓存
pub const FILE: &str = "file";

/// 仅在线用户服务 presence 使用的缓存
pub const PRESENCE: &str = "presence";

/// 好友的缓存
pub const FRIEND: &str = "friend";

/// 视频通话
pub const VIDEO_CALL: &str = "VideoCall";

/// 仅租户服务 tenant 使用的缓存
pub const SYSTEM: &str = "system";

/// 仅网关服务 gateway 使用的缓存
pub const GATEWAY: &str = "gateway";

/// 缓存 key 前缀
///
/// 可以在启动时覆盖该参数，系统启动时注入。
static PREFIX: OnceLock<String> = OnceLock::new();

/// 设置缓存 key 前缀
///
/// # 参数
/// - `prefix`: 要设置的前缀字符串
///
/// # 示例
/// ```
/// use fbc_starter::cache::set_cache_prefix;
///
/// set_cache_prefix("dev".to_string());
/// ```
pub fn set_cache_prefix(prefix: String) {
    let _ = PREFIX.set(prefix);
}

/// 获取缓存 key 前缀
///
/// 如果未设置，返回 `None`
///
/// # 示例
/// ```
/// use fbc_starter::cache::get_cache_prefix;
///
/// if let Some(prefix) = get_cache_prefix() {
///     println!("当前前缀: {}", prefix);
/// }
/// ```
pub fn get_cache_prefix() -> Option<&'static String> {
    PREFIX.get()
}

/// 获取缓存 key 前缀，如果未设置则返回默认值
///
/// # 参数
/// - `default`: 如果未设置前缀时返回的默认值
///
/// # 示例
/// ```
/// use fbc_starter::cache::get_cache_prefix_or;
///
/// let prefix = get_cache_prefix_or("prod".to_string());
/// ```
pub fn get_cache_prefix_or(default: String) -> String {
    PREFIX.get().cloned().unwrap_or(default)
}
