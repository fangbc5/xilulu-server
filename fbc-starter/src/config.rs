use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// 服务器配置
    pub server: ServerConfig,
    /// 日志配置
    pub log: LogConfig,
    /// CORS 配置
    pub cors: CorsConfig,
    /// 数据库配置（可选，需要启用 mysql/postgres/sqlite 任一特性）
    #[serde(default)]
    #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
    pub database: Option<DatabaseConfig>,
    /// Redis 配置（可选，需要启用 redis 特性）
    #[serde(default)]
    #[cfg(feature = "redis")]
    pub redis: Option<RedisConfig>,
    /// Nacos 配置（可选，需要启用 nacos 特性）
    #[serde(default)]
    #[cfg(feature = "nacos")]
    pub nacos: Option<NacosConfig>,
    /// Kafka 配置（可选，需要启用 kafka 特性）
    #[serde(default)]
    #[cfg(feature = "kafka")]
    pub kafka: Option<KafkaConfig>,
}

/// 服务器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 监听地址
    pub addr: String,
    /// 端口
    pub port: u16,
    /// 工作线程数（0 表示使用默认值）
    pub workers: Option<usize>,
    /// 上下文路径（可选），例如 "/api"，如果不配置则为空
    #[serde(default)]
    pub context_path: Option<String>,
}

impl ServerConfig {
    /// 获取完整的 SocketAddr
    pub fn socket_addr(&self) -> Result<SocketAddr, std::net::AddrParseError> {
        format!("{}:{}", self.addr, self.port).parse()
    }
}

/// 日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// 日志级别 (trace, debug, info, warn, error)
    pub level: String,
    /// 是否使用 JSON 格式
    pub json: bool,
    /// 时区偏移（小时）。默认 8（东八区）。例如：8 表示 UTC+8, -5 表示 UTC-5
    #[serde(default = "default_log_timezone")]
    pub timezone: i32,
    /// 文件日志配置（可选）
    #[serde(default)]
    pub file: Option<FileLogConfig>,
}

fn default_log_timezone() -> i32 {
    8 // 默认东八区
}

/// 文件日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileLogConfig {
    /// 日志目录（例如：./logs）。默认 ./logs
    #[serde(default = "default_log_directory")]
    pub directory: String,
    /// 日志文件名前缀（例如：app，最终文件名会是 app.log）
    #[serde(default = "default_log_filename")]
    pub filename: String,
    /// 日志输出格式：plain 或 json
    #[serde(default = "default_log_format")]
    pub format: String,
    /// 单个日志文件大小限制（MB）。0 表示不限制。默认 100MB
    #[serde(default = "default_log_size_limit_mb")]
    pub size_limit_mb: u64,
    /// 保留的最大日志文件数量。0 表示不限制。默认 10 个
    #[serde(default = "default_log_count_limit")]
    pub count_limit: u32,
    /// 滚动策略：daily（按天） 或 size（按大小）。默认 daily
    #[serde(default = "default_log_rotation")]
    pub rotation: String,
}

fn default_log_directory() -> String {
    "./logs".to_string()
}

fn default_log_filename() -> String {
    "app".to_string()
}

fn default_log_format() -> String {
    "plain".to_string()
}

fn default_log_size_limit_mb() -> u64 {
    100
}

fn default_log_count_limit() -> u32 {
    10
}

fn default_log_rotation() -> String {
    "daily".to_string()
}

/// CORS 配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// 允许的源（* 表示允许所有）
    pub allowed_origins: Vec<String>,
    /// 允许的方法
    pub allowed_methods: Vec<String>,
    /// 允许的请求头
    pub allowed_headers: Vec<String>,
    /// 是否允许凭证
    pub allow_credentials: bool,
}

/// 数据库配置（需要启用 mysql/postgres/sqlite 任一特性）
#[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库 URL（例如：postgres://user:password@localhost/dbname）
    pub url: String,
    /// 最大连接数
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    /// 最小连接数
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,
}

#[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
fn default_max_connections() -> u32 {
    100
}

#[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
fn default_min_connections() -> u32 {
    10
}

/// Redis 配置（需要启用 redis 特性）
#[cfg(feature = "redis")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// Redis URL（例如：redis://127.0.0.1:6379 或 redis://:password@127.0.0.1:6379）
    pub url: String,
    /// Redis 密码（可选，如果 URL 中已包含密码则不需要）
    #[serde(default)]
    pub password: Option<String>,
    /// 连接池大小
    #[serde(default = "default_pool_size")]
    pub pool_size: usize,
}

#[cfg(feature = "redis")]
fn default_pool_size() -> usize {
    10
}

/// Nacos 配置（需要启用 nacos 特性）
#[cfg(feature = "nacos")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NacosConfig {
    /// Nacos 服务器地址列表（例如：["http://127.0.0.1:8848"]）
    #[serde(default = "default_nacos_server_addrs")]
    pub server_addrs: Vec<String>,
    /// 全局命名空间（可选，向后兼容）
    /// 当 naming_namespace / config_namespace 未设置时，回退到此字段
    pub namespace: Option<String>,
    /// 服务注册/发现专用命名空间（优先级高于 namespace）
    #[serde(default)]
    pub naming_namespace: Option<String>,
    /// 配置管理专用命名空间（优先级高于 namespace）
    #[serde(default)]
    pub config_namespace: Option<String>,
    /// 用户名（可选，用于认证，默认为 "nacos"）
    #[serde(default = "default_nacos_username")]
    pub username: Option<String>,
    /// 密码（可选，用于认证，默认为 "nacos"）
    #[serde(default = "default_nacos_password")]
    pub password: Option<String>,
    /// 服务名称（用于服务注册，如果为空则使用环境变量 CARGO_PKG_NAME）
    #[serde(default)]
    pub service_name: String,
    /// 全局服务组名（可选，默认为 DEFAULT_GROUP）
    /// 当 naming_group / config_group 未设置时，回退到此字段
    #[serde(default = "default_nacos_group")]
    pub group_name: String,
    /// 服务注册/发现专用组名（优先级高于 group_name）
    #[serde(default)]
    pub naming_group: Option<String>,
    /// 配置管理专用组名（优先级高于 group_name）
    #[serde(default)]
    pub config_group: Option<String>,
    /// 服务 IP（可选，默认使用服务器配置的地址）
    #[serde(default)]
    pub service_ip: Option<String>,
    /// 服务端口（可选，默认使用服务器配置的端口）
    #[serde(default)]
    pub service_port: Option<u32>,
    /// 健康检查路径（可选，默认为 "/health"）
    #[serde(default = "default_nacos_health_check_path")]
    pub health_check_path: Option<String>,
    /// 元数据（可选）
    #[serde(default)]
    pub metadata: Option<std::collections::HashMap<String, String>>,
    /// 订阅的服务列表（可选，用于服务发现）
    /// 环境变量支持逗号分隔：APP__NACOS__SUBSCRIBE_SERVICES=im-server,user-service
    #[serde(
        default,
        deserialize_with = "crate::utils::serde_helpers::deserialize_string_or_vec"
    )]
    pub subscribe_services: Vec<String>,
    /// 订阅的配置列表（可选，用于配置管理）
    #[serde(default)]
    pub subscribe_configs: Vec<NacosConfigItem>,
}

#[cfg(feature = "nacos")]
impl NacosConfig {
    /// 获取服务注册/发现使用的有效命名空间
    /// 优先级: naming_namespace > namespace > None
    pub fn effective_naming_namespace(&self) -> Option<&String> {
        self.naming_namespace.as_ref().or(self.namespace.as_ref())
    }

    /// 获取配置管理使用的有效命名空间
    /// 优先级: config_namespace > namespace > None
    pub fn effective_config_namespace(&self) -> Option<&String> {
        self.config_namespace.as_ref().or(self.namespace.as_ref())
    }

    /// 获取服务注册/发现使用的有效组名
    /// 优先级: naming_group > group_name
    pub fn effective_naming_group(&self) -> &str {
        self.naming_group.as_deref().unwrap_or(&self.group_name)
    }

    /// 获取配置管理使用的有效组名
    /// 优先级: config_group > group_name
    pub fn effective_config_group(&self) -> &str {
        self.config_group.as_deref().unwrap_or(&self.group_name)
    }

    /// 命名空间是否分离（naming 和 config 使用不同命名空间）
    pub fn is_namespace_separated(&self) -> bool {
        let naming_ns = self.effective_naming_namespace();
        let config_ns = self.effective_config_namespace();
        naming_ns != config_ns
    }
}

/// Nacos 配置项
#[cfg(feature = "nacos")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NacosConfigItem {
    /// 配置的 Data ID
    pub data_id: String,
    /// 配置的 Group（可选，默认为 DEFAULT_GROUP）
    #[serde(default = "default_nacos_group")]
    pub group: String,
    /// 命名空间（可选）
    #[serde(default = "default_nacos_namespace")]
    pub namespace: String,
}

#[cfg(feature = "nacos")]
fn default_nacos_group() -> String {
    "DEFAULT_GROUP".to_string()
}

#[cfg(feature = "nacos")]
fn default_nacos_server_addrs() -> Vec<String> {
    vec!["127.0.0.1:8848".to_string()]
}

#[cfg(feature = "nacos")]
fn default_nacos_health_check_path() -> Option<String> {
    Some("/health".to_string())
}

#[cfg(feature = "nacos")]
fn default_nacos_namespace() -> String {
    "public".to_string()
}

#[cfg(feature = "nacos")]
fn default_nacos_username() -> Option<String> {
    Some("nacos".to_string())
}

#[cfg(feature = "nacos")]
fn default_nacos_password() -> Option<String> {
    Some("nacos".to_string())
}

/// Kafka 配置（需要启用 kafka 特性）
#[cfg(feature = "kafka")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConfig {
    /// Kafka 集群地址（例如：localhost:9092 或 10.0.0.1:9092,10.0.0.2:9092）
    pub brokers: String,
    /// 生产者配置（可选，需要启用 producer 特性）
    #[serde(default)]
    pub producer: Option<KafkaProducerConfig>,
    /// 消费者配置（可选，需要启用 consumer 特性）
    #[serde(default)]
    pub consumer: Option<KafkaConsumerConfig>,
}

/// Kafka 生产者配置
#[cfg(feature = "kafka")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaProducerConfig {
    /// 生产者重试次数
    #[serde(default = "default_producer_retries")]
    pub retries: i32,
    /// 是否启用幂等性
    #[serde(default = "default_producer_idempotence")]
    pub enable_idempotence: bool,
    /// ACK 模式 (all, 1, 0)
    #[serde(default = "default_producer_acks")]
    pub acks: String,
}

/// Kafka 消费者配置
#[cfg(feature = "kafka")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KafkaConsumerConfig {
    /// 是否自动提交偏移量
    #[serde(default = "default_consumer_auto_commit")]
    pub enable_auto_commit: bool,
}

// Kafka 生产者默认值
#[cfg(feature = "kafka")]
fn default_producer_retries() -> i32 {
    3
}

#[cfg(feature = "kafka")]
fn default_producer_idempotence() -> bool {
    true
}

#[cfg(feature = "kafka")]
fn default_producer_acks() -> String {
    "all".to_string()
}

// Kafka 消费者默认值
#[cfg(feature = "kafka")]
fn default_consumer_auto_commit() -> bool {
    false
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                addr: "127.0.0.1".to_string(),
                port: 3000,
                workers: None,
                context_path: None,
            },
            log: LogConfig {
                level: "info".to_string(),
                json: false,
                timezone: 8,
                file: None,
            },
            cors: CorsConfig {
                allowed_origins: vec!["*".to_string()],
                allowed_methods: vec![
                    "GET".to_string(),
                    "POST".to_string(),
                    "PUT".to_string(),
                    "DELETE".to_string(),
                    "PATCH".to_string(),
                    "OPTIONS".to_string(),
                ],
                allowed_headers: vec!["*".to_string()],
                // 注意：当 allowed_origins 或 allowed_headers 为 * 时，allow_credentials 会自动设置为 false
                allow_credentials: false,
            },
            #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
            database: None,
            #[cfg(feature = "redis")]
            redis: None,
            #[cfg(feature = "nacos")]
            nacos: None,
            #[cfg(feature = "kafka")]
            kafka: None,
        }
    }
}

impl Config {
    /// 获取本机 IP 地址
    /// 返回第一个非回环的 IPv4 地址，如果获取失败则返回 None
    fn get_local_ip() -> Option<String> {
        match local_ip_address::local_ip() {
            Ok(ip) => {
                // 只返回 IPv4 地址，跳过回环地址
                if ip.is_ipv4() && !ip.is_loopback() {
                    Some(ip.to_string())
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    /// 从 .env 文件和环境变量加载配置
    ///
    /// 配置项命名规则：
    /// - APP__SERVER__ADDR -> server.addr (如果不配置，自动获取本机 IP，获取不到则使用 127.0.0.1)
    /// - APP__SERVER__PORT -> server.port
    /// - APP__SERVER__CONTEXT_PATH -> server.context_path (可选，例如 "/api")
    /// - APP__LOG__LEVEL -> log.level
    /// - APP__LOG__JSON -> log.json
    /// - APP__CORS__ALLOWED_ORIGINS -> cors.allowed_origins (逗号分隔)
    /// - APP__CORS__ALLOWED_METHODS -> cors.allowed_methods (逗号分隔)
    /// - APP__CORS__ALLOWED_HEADERS -> cors.allowed_headers (逗号分隔)
    /// - APP__CORS__ALLOW_CREDENTIALS -> cors.allow_credentials
    /// - APP__DATABASE__URL -> database.url (可选，需要启用 database 特性)
    /// - APP__DATABASE__MAX_CONNECTIONS -> database.max_connections (可选，默认 100)
    /// - APP__DATABASE__MIN_CONNECTIONS -> database.min_connections (可选，默认 10)
    /// - APP__REDIS__URL -> redis.url (可选，需要启用 redis 特性)
    /// - APP__REDIS__PASSWORD -> redis.password (可选，如果 URL 中已包含密码则不需要)
    /// - APP__REDIS__POOL_SIZE -> redis.pool_size (可选，默认 10)
    /// 查找项目根目录（通过查找 Cargo.toml 或 .env 文件）
    ///
    /// 查找策略（按优先级）：
    /// 1. 从可执行文件路径推断项目目录（例如 target/debug/im-server -> im-server/）
    /// 2. 从可执行文件所在目录向上查找 .env 文件
    /// 3. 从当前工作目录向上查找 .env 文件
    fn find_project_root() -> Option<std::path::PathBuf> {
        // 策略 1: 从可执行文件路径推断项目目录
        // 例如：/path/to/hula-server/target/debug/im-server -> /path/to/hula-server/im-server/
        if let Ok(exe_path) = std::env::current_exe() {
            // 获取可执行文件名（例如 "im-server"）
            if let Some(exe_name) = exe_path.file_stem().and_then(|s| s.to_str()) {
                // 从可执行文件路径向上查找，直到找到 workspace 根目录或项目根目录
                if let Some(exe_dir) = exe_path.parent() {
                    let mut path = exe_dir.to_path_buf();
                    loop {
                        // 检查当前目录的父目录是否包含与可执行文件同名的目录
                        if let Some(parent) = path.parent() {
                            let project_dir = parent.join(exe_name);
                            // 如果找到同名目录且包含 .env 文件，这就是项目根目录
                            if project_dir.join(".env").exists() {
                                return Some(project_dir);
                            }
                            // 如果找到同名目录且包含 Cargo.toml（非 workspace），这也是项目根目录
                            let cargo_toml = project_dir.join("Cargo.toml");
                            if cargo_toml.exists() {
                                if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                                    if !content.contains("[workspace]") {
                                        return Some(project_dir);
                                    }
                                }
                            }
                        }
                        // 检查当前目录是否有 .env 文件
                        if path.join(".env").exists() {
                            return Some(path);
                        }
                        // 向上查找
                        match path.parent() {
                            Some(parent) => path = parent.to_path_buf(),
                            None => break,
                        }
                    }
                }
            }
        }

        // 策略 2: 从当前工作目录向上查找 .env 文件
        if let Ok(mut current_dir) = std::env::current_dir() {
            loop {
                if current_dir.join(".env").exists() {
                    // 检查是否是 workspace 根目录
                    let cargo_toml = current_dir.join("Cargo.toml");
                    if cargo_toml.exists() {
                        if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                            if content.contains("[workspace]") {
                                // 这是 workspace 根目录，但找到了 .env，返回当前目录
                                return Some(current_dir);
                            }
                        }
                    }
                    return Some(current_dir);
                }
                match current_dir.parent() {
                    Some(parent) => current_dir = parent.to_path_buf(),
                    None => break,
                }
            }
        }

        None
    }

    pub fn from_env() -> Result<Self, config::ConfigError> {
        // 加载 .env 文件（如果存在）
        // 优先从项目根目录加载，确保当库被其他项目使用时能正确找到 .env 文件

        // 方法 1: 使用 CARGO_MANIFEST_DIR 环境变量（编译时设置，最可靠）
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let env_path = std::path::Path::new(&manifest_dir).join(".env");
            if env_path.exists() {
                if dotenvy::from_path(&env_path).is_ok() {
                    eprintln!(
                        "✓ 从 CARGO_MANIFEST_DIR 加载 .env 文件: {}",
                        env_path.display()
                    );
                    return Self::load_config_from_env();
                }
            }
        }

        // 方法 2: 从项目根目录加载（通过查找逻辑）
        if let Some(project_root) = Self::find_project_root() {
            let env_path = project_root.join(".env");
            if env_path.exists() {
                if dotenvy::from_path(&env_path).is_ok() {
                    eprintln!("✓ 从项目根目录加载 .env 文件: {}", env_path.display());
                    return Self::load_config_from_env();
                }
            }
        }

        // 方法 3: 使用 dotenvy::dotenv() 从当前工作目录向上查找（备用方案）
        match dotenvy::dotenv() {
            Ok(path) => {
                eprintln!("✓ 从当前工作目录向上查找加载 .env 文件: {}", path.display());
            }
            Err(_) => {
                eprintln!("⚠ 未找到 .env 文件，将使用环境变量和默认配置");
            }
        }

        Self::load_config_from_env()
    }

    /// 从环境变量加载配置（内部方法）
    fn load_config_from_env() -> Result<Self, config::ConfigError> {
        // 先手动处理数组类型的配置项，设置默认值
        let mut default_origins = vec!["*".to_string()];
        let mut default_methods = vec![
            "GET".to_string(),
            "POST".to_string(),
            "PUT".to_string(),
            "DELETE".to_string(),
            "PATCH".to_string(),
            "OPTIONS".to_string(),
        ];
        let mut default_headers = vec!["*".to_string()];

        // 从环境变量读取数组配置
        if let Ok(origins_str) = std::env::var("APP__CORS__ALLOWED_ORIGINS") {
            default_origins = origins_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        if let Ok(methods_str) = std::env::var("APP__CORS__ALLOWED_METHODS") {
            default_methods = methods_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        if let Ok(headers_str) = std::env::var("APP__CORS__ALLOWED_HEADERS") {
            default_headers = headers_str
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        // 处理 Nacos server_addrs 数组类型配置（同 CORS 数组处理逻辑）
        #[cfg(feature = "nacos")]
        let nacos_server_addrs_override: Option<Vec<String>> = {
            if let Ok(addrs_str) = std::env::var("APP__NACOS__SERVER_ADDRS") {
                Some(
                    addrs_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect(),
                )
            } else {
                None
            }
        };

        // 临时移除这些环境变量，避免 config crate 尝试解析它们（Vec<String> 类型无法直接反序列化）
        let origins_backup = std::env::var("APP__CORS__ALLOWED_ORIGINS").ok();
        let methods_backup = std::env::var("APP__CORS__ALLOWED_METHODS").ok();
        let headers_backup = std::env::var("APP__CORS__ALLOWED_HEADERS").ok();
        #[cfg(feature = "nacos")]
        let nacos_addrs_backup = std::env::var("APP__NACOS__SERVER_ADDRS").ok();

        if origins_backup.is_some() {
            std::env::remove_var("APP__CORS__ALLOWED_ORIGINS");
        }
        if methods_backup.is_some() {
            std::env::remove_var("APP__CORS__ALLOWED_METHODS");
        }
        if headers_backup.is_some() {
            std::env::remove_var("APP__CORS__ALLOWED_HEADERS");
        }
        #[cfg(feature = "nacos")]
        if nacos_addrs_backup.is_some() {
            std::env::remove_var("APP__NACOS__SERVER_ADDRS");
        }

        // 如果未配置 APP__SERVER__ADDR，则自动获取本机 IP
        // 注意：如果环境变量 APP__SERVER__ADDR 存在，config crate 会优先使用环境变量的值，set_default 的值不会被使用
        let default_server_addr = if std::env::var("APP__SERVER__ADDR").is_ok() {
            // 环境变量已存在，set_default 的值不会被使用，但 API 要求提供一个值
            // 这里返回任意值都可以，因为不会被使用
            "127.0.0.1".to_string()
        } else {
            // 环境变量不存在，尝试获取本机 IP 作为默认值
            match Self::get_local_ip() {
                Some(ip) => {
                    eprintln!("✓ 自动获取本机 IP 地址: {}", ip);
                    ip
                }
                None => {
                    eprintln!("⚠ 无法获取本机 IP 地址，将使用 127.0.0.1");
                    "127.0.0.1".to_string()
                }
            }
        };

        let builder = config::Config::builder()
            .set_default("server.addr", default_server_addr.as_str())?
            .set_default("server.port", 3000)?
            .set_default("log.level", "info")?
            .set_default("log.json", false)?
            .set_default("log.timezone", 8)?
            // 文件日志配置默认值
            .set_default("log.file.directory", "./logs")?
            .set_default("log.file.filename", "app")?
            .set_default("log.file.format", "plain")?
            .set_default("log.file.size_limit_mb", 100u64)?
            .set_default("log.file.count_limit", 10u32)?
            .set_default("log.file.rotation", "daily")?
            .set_default("cors.allowed_origins", default_origins.clone())?
            .set_default("cors.allowed_methods", default_methods.clone())?
            .set_default("cors.allowed_headers", default_headers.clone())?
            .set_default("cors.allow_credentials", false)?;

        // Nacos 配置默认值
        #[cfg(feature = "nacos")]
        let builder = {
            // 如果环境变量提供了 server_addrs，使用手动解析的值（因为已从环境中移除）
            let nacos_addrs = nacos_server_addrs_override
                .clone()
                .unwrap_or_else(default_nacos_server_addrs);
            builder
                .set_default("nacos.server_addrs", nacos_addrs)?
                .set_default("nacos.service_name", String::new())?
                .set_default("nacos.group_name", default_nacos_group())?
                .set_default("nacos.namespace", default_nacos_namespace())?
                .set_default("nacos.username", default_nacos_username())?
                .set_default("nacos.password", default_nacos_password())?
                .set_default("nacos.health_check_path", default_nacos_health_check_path())?
        };

        // Kafka 配置默认值
        #[cfg(feature = "kafka")]
        let builder = builder.set_default("kafka.brokers", "localhost:9092")?;

        #[cfg(feature = "producer")]
        let builder = builder
            .set_default("kafka.producer.retries", default_producer_retries())?
            .set_default(
                "kafka.producer.enable_idempotence",
                default_producer_idempotence(),
            )?
            .set_default("kafka.producer.acks", default_producer_acks())?;

        #[cfg(feature = "consumer")]
        let builder = builder.set_default(
            "kafka.consumer.enable_auto_commit",
            default_consumer_auto_commit(),
        )?;

        let builder = builder.add_source(config::Environment::with_prefix("APP").separator("__"));

        let config = builder.build()?;
        let result: Config = config.try_deserialize()?;

        // 恢复被临时移除的环境变量
        if let Some(v) = origins_backup {
            std::env::set_var("APP__CORS__ALLOWED_ORIGINS", v);
        }
        if let Some(v) = methods_backup {
            std::env::set_var("APP__CORS__ALLOWED_METHODS", v);
        }
        if let Some(v) = headers_backup {
            std::env::set_var("APP__CORS__ALLOWED_HEADERS", v);
        }
        #[cfg(feature = "nacos")]
        if let Some(v) = nacos_addrs_backup {
            std::env::set_var("APP__NACOS__SERVER_ADDRS", v);
        }

        Ok(result)
    }
}
