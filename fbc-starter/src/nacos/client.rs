use crate::config::NacosConfig;
use nacos_rust_client::client::AuthInfo;
use once_cell::sync::OnceCell;

/// Nacos 命名客户端（用于服务注册与发现）
static NAMING_CLIENT: OnceCell<
    std::sync::Arc<nacos_rust_client::client::naming_client::NamingClient>,
> = OnceCell::new();

/// Nacos 配置客户端（用于配置管理）
static CONFIG_CLIENT: OnceCell<
    std::sync::Arc<nacos_rust_client::client::config_client::ConfigClient>,
> = OnceCell::new();

/// 初始化 Nacos 客户端
pub async fn init_nacos(config: &NacosConfig) -> Result<(), anyhow::Error> {
    use nacos_rust_client::client::ClientBuilder;

    // 获取服务器地址（将数组转换为逗号分隔的字符串格式）
    let server_addr = if config.server_addrs.is_empty() {
        "127.0.0.1:8848".to_string()
    } else {
        config.server_addrs.join(",")
    };

    // 创建认证信息
    let auth_info: Option<AuthInfo> =
        if let (Some(username), Some(password)) = (&config.username, &config.password) {
            if !username.is_empty() && !password.is_empty() {
                Some(AuthInfo::new(username, password))
            } else {
                None
            }
        } else {
            None
        };

    // 使用 ClientBuilder 初始化客户端
    // 注意：nacos_rust_client 的 ClientBuilder 共享一个 tenant 设置
    // 当 naming 和 config 的命名空间不同时，优先使用 naming_namespace
    // （config 订阅时在 ConfigKey 中已包含各自的 namespace）
    let mut builder = ClientBuilder::new()
        .set_endpoint_addrs(&server_addr)
        .set_auth_info(auth_info.clone())
        .set_use_grpc(true);

    // 设置命名空间（优先使用 naming_namespace，因为 naming 客户端需要在 builder 级别设置）
    let naming_tenant = if let Some(ns) = config.effective_naming_namespace() {
        builder = builder.set_tenant(ns.to_string());
        ns.to_string()
    } else {
        "public".to_string()
    };

    let config_tenant = config
        .effective_config_namespace()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "public".to_string());

    // 设置应用名称（使用服务名称或环境变量）
    let app_name = if !config.service_name.is_empty() {
        config.service_name.clone()
    } else {
        std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "unknown-service".to_string())
    };
    builder = builder.set_app_name(app_name);

    // 构建命名客户端和配置客户端
    let (config_client, naming_client) = builder.build();

    NAMING_CLIENT
        .set(naming_client)
        .map_err(|_| anyhow::anyhow!("Nacos 命名客户端已初始化"))?;

    CONFIG_CLIENT
        .set(config_client)
        .map_err(|_| anyhow::anyhow!("Nacos 配置客户端已初始化"))?;

    tracing::info!(
        "Nacos 客户端初始化成功 (服务器: {}, 命名空间: naming={}, config={}, 认证: {})",
        server_addr,
        naming_tenant,
        config_tenant,
        if auth_info.is_some() {
            "已启用"
        } else {
            "未启用"
        }
    );
    Ok(())
}

/// 获取 Nacos 命名客户端
pub fn get_naming_client() -> Result<
    std::sync::Arc<nacos_rust_client::client::naming_client::NamingClient>,
    crate::error::AppError,
> {
    NAMING_CLIENT
        .get()
        .cloned()
        .ok_or(crate::error::AppError::BadRequest(
            "Nacos 命名客户端未初始化".to_string(),
        ))
}

/// 获取 Nacos 配置客户端
pub fn get_config_client() -> Result<
    std::sync::Arc<nacos_rust_client::client::config_client::ConfigClient>,
    crate::error::AppError,
> {
    CONFIG_CLIENT
        .get()
        .cloned()
        .ok_or(crate::error::AppError::BadRequest(
            "Nacos 配置客户端未初始化".to_string(),
        ))
}
