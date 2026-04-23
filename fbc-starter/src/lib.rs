pub mod auth;
pub mod base;
pub mod config;
pub mod constants;
pub mod entity;
pub mod error;
pub mod http;
pub mod logging;
pub mod server;
pub mod state;
pub mod state_machine;
pub mod utils;

#[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
pub mod database;

pub mod cache;

#[cfg(feature = "balance")]
pub mod balance;

#[cfg(feature = "nacos")]
pub mod nacos;

#[cfg(feature = "kafka")]
pub mod messaging;

pub use base::{CursorPageBaseResp, R};
pub use config::Config;
#[cfg(feature = "kafka")]
pub use config::{
    KafkaConfig, KafkaConsumerConfig as ConfigKafkaConsumerConfig,
    KafkaProducerConfig as ConfigKafkaProducerConfig,
};
pub use constants::{
    CREATE_BY, CREATE_BY_FIELD, CREATE_ORG_ID_FIELD, CREATE_TIME, CREATE_TIME_FIELD, DELETE_FIELD,
    ID_FIELD, LABEL, PARENT_ID, PARENT_ID_FIELD, SORT_VALUE, SORT_VALUE_FIELD, TENANT_ID,
    UPDATE_BY, UPDATE_BY_FIELD, UPDATE_TIME, UPDATE_TIME_FIELD,
};

pub use auth::{RequestContext, user_context_middleware, Claims, JwtService};
pub use entity::*;
pub use error::{AppError, AppResult};
pub use http::{
    create_cors_layer, request_logging_middleware, grpc_log_request, grpc_log_response,
    health_check, root,
};
#[cfg(feature = "kafka")]
pub use messaging::{
    kafka::{KafkaConsumer, KafkaProducer},
    Message, MessageConsumer, MessageConsumerType, MessageProducer, MessageProducerType,
};
#[cfg(feature = "consumer")]
pub use messaging::{KafkaMessageHandler, KafkaMessageRouter};
#[cfg(feature = "nacos")]
pub use nacos::{
    deregister_service, get_config, get_config_client, get_naming_client, get_service_instances,
    get_subscribed_configs, get_subscribed_services, init_nacos, register_service,
    subscribe_configs, subscribe_services,
};
pub use server::{Server, ServerBuilder};
pub use state::AppState;
pub use utils::get_uid_from_headers;

/// 初始化日志系统（内部函数）
///
/// 设置时区（可配置，默认为东八区 UTC+8）
/// 支持两种日志文件滚动策略：
/// - daily: 按天自动滚动（使用 tracing_appender::rolling::daily）
/// - size: 按文件大小手动滚动（需配置 size_limit_mb）
pub(crate) fn init_logging(config: &Config) -> Result<(), anyhow::Error> {
    use time::UtcOffset;
    use tracing_subscriber::{
        fmt::time::OffsetTime, layer::SubscriberExt, util::SubscriberInitExt,
    };

    // 从配置读取时区偏移（小时），默认东八区 +8
    let tz_hour = config.log.timezone;
    let offset = UtcOffset::from_hms(tz_hour as i8, 0, 0)
        .map_err(|e| {
            if tz_hour >= 0 {
                anyhow::anyhow!("无效的时区偏移 UTC+{}: {}", tz_hour, e)
            } else {
                anyhow::anyhow!("无效的时区偏移 UTC{}: {}", tz_hour, e)
            }
        })?;
    
    eprintln!(
        "ℹ 日志时区设置: UTC{}",
        if tz_hour >= 0 {
            format!("+{}", tz_hour)
        } else {
            tz_hour.to_string()
        }
    );

    let timer = OffsetTime::new(
        offset,
        time::format_description::parse(
            "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]",
        )
        .map_err(|e| anyhow::anyhow!("时间格式解析失败: {}", e))?,
    );

    // 初始化日志环境变量过滤器
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        config.log.level.as_str().into()
    });

    // 构建 stdout 层：JSON 或普通格式（使用 Option 层避免分支重复）
    let stdout_json = if config.log.json {
        Some(
            tracing_subscriber::fmt::layer()
                .json()
                .with_timer(timer.clone())
                .with_line_number(true),
        )
    } else {
        None
    };
    let stdout_plain = if !config.log.json {
        Some(
            tracing_subscriber::fmt::layer()
                .with_timer(timer.clone())
                .with_line_number(true),
        )
    } else {
        None
    };

    // 构建文件日志层（可选）
    let (file_json, file_plain) = if let Some(ref file_config) = config.log.file {
        // 创建日志文件目录
        std::fs::create_dir_all(&file_config.directory)
            .map_err(|e| anyhow::anyhow!("无法创建日志目录 {}: {}", file_config.directory, e))?;

        // 清理旧日志文件（如果配置了限制）
        if file_config.count_limit > 0 {
            crate::logging::cleanup_old_logs(
                &file_config.directory,
                &file_config.filename,
                file_config.count_limit,
            ).ok();
        }

        // 创建文件 appender（按大小 or 按天，目前都使用按天滚动）
        let file_appender = match file_config.rotation.as_str() {
            "size" => {
                eprintln!(
                    "ℹ 日志配置: 按大小滚动 (限制: {}MB, 保留: {} 个文件)",
                    if file_config.size_limit_mb == 0 {
                        "无限制".to_string()
                    } else {
                        file_config.size_limit_mb.to_string()
                    },
                    if file_config.count_limit == 0 {
                        "无限制".to_string()
                    } else {
                        file_config.count_limit.to_string()
                    }
                );
                tracing_appender::rolling::daily(
                    &file_config.directory,
                    &file_config.filename,
                )
            }
            _ => {
                tracing_appender::rolling::daily(
                    &file_config.directory,
                    &file_config.filename,
                )
            }
        };

        let is_file_json = file_config.format.as_str() == "json";
        let fj = if is_file_json {
            Some(
                tracing_subscriber::fmt::layer()
                    .json()
                    .with_writer(file_appender)
                    .with_timer(timer.clone())
                    .with_line_number(true)
                    .with_ansi(false),
            )
        } else {
            None
        };
        // 需要重新创建 file_appender（已被 move），如果不是 json 模式
        let fp = if !is_file_json {
            let file_appender2 = tracing_appender::rolling::daily(
                &file_config.directory,
                &file_config.filename,
            );
            Some(
                tracing_subscriber::fmt::layer()
                    .with_writer(file_appender2)
                    .with_timer(timer)
                    .with_line_number(true)
                    .with_ansi(false),
            )
        } else {
            None
        };
        (fj, fp)
    } else {
        (None, None)
    };

    // 组合所有层并初始化（Option<Layer> 自动实现 Layer）
    tracing_subscriber::registry()
        .with(env_filter)
        .with(stdout_json)
        .with(stdout_plain)
        .with(file_json)
        .with(file_plain)
        .try_init()
        .map_err(|e| anyhow::anyhow!("日志系统初始化失败: {}", e))?;

    Ok(())
}


#[cfg(feature = "balance")]
pub use balance::{
    create_grpc_channel, create_grpc_client, get_load_balancer, get_service_endpoints,
    grpc_call, get_instance_circuit_breaker, get_existing_instance_circuit_breaker,
    CircuitBreaker, CircuitBreakerConfig, CircuitState, GrpcClientBuilder,
    HealthCheckConfig, HealthStatus, LoadBalancer, ResilientGrpcClient, RetryConfig,
    RoundRobinLoadBalancer, ServiceEndpoint, start_health_checker,
};

