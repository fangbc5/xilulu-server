use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, Router},
    Router as AxumRouter,
};
use serde_json::json;
use std::sync::Arc;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tracing::{info, warn};

use crate::http::{create_cors_layer, health_check, request_logging_middleware, root};
// gRPC 专用日志工具函数，预留给需要对 gRPC 请求/响应做细粒度日志记录的场景
// （当前全局 request_logging_middleware 已覆盖基本的 gRPC fallback 日志）
#[cfg(feature = "grpc")]
#[allow(unused_imports)]
use crate::http::{grpc_log_request, grpc_log_response};
use crate::state::AppState;
use crate::{config::Config, AppResult};

/// Web 服务器
pub struct Server {
    config: Config,
    app_state: Arc<AppState>,
    http_router: Option<AxumRouter>,
    #[cfg(feature = "grpc")]
    grpc_router: Option<tonic::service::Routes>,
}

/// 服务器构建器，用于在闭包中配置服务器
pub struct ServerBuilder {
    config: Config,
    app_state: Arc<AppState>,
    http_router: Option<AxumRouter>,
    #[cfg(feature = "grpc")]
    grpc_router: Option<tonic::service::Routes>,
    #[cfg(feature = "consumer")]
    kafka_handlers: Vec<Arc<dyn crate::messaging::KafkaMessageHandler>>,
}

impl ServerBuilder {
    /// 创建新的服务器构建器
    #[warn(dead_code)]
    fn new(config: Config, app_state: Arc<AppState>) -> Self {
        Self {
            config,
            app_state,
            http_router: None,
            #[cfg(feature = "grpc")]
            grpc_router: None,
            #[cfg(feature = "consumer")]
            kafka_handlers: Vec::new(),
        }
    }

    /// 获取应用状态的引用
    pub fn app_state(&self) -> &Arc<AppState> {
        &self.app_state
    }

    /// 注册 Kafka 消息处理器
    ///
    /// # 参数
    /// - `handler`: 实现了 KafkaMessageHandler trait 的处理器
    ///
    /// # 示例
    /// ```ignore
    /// Server::run(|builder| {
    ///     builder
    ///         .with_kafka_handler(Arc::new(MyHandler::new()))
    ///         .http_router(routes)
    /// })
    /// ```
    #[cfg(feature = "consumer")]
    pub fn with_kafka_handler(
        mut self,
        handler: Arc<dyn crate::messaging::KafkaMessageHandler>,
    ) -> Self {
        self.kafka_handlers.push(handler);
        self
    }

    /// 批量注册 Kafka 消息处理器
    ///
    /// # 示例
    ///
    /// ```no_run
    /// use fbc_starter::Server;
    /// use std::sync::Arc;
    ///
    /// Server::run(|builder| {
    ///     let handlers = vec![
    ///         Arc::new(Handler1::new()) as Arc<dyn fbc_starter::KafkaMessageHandler>,
    ///         Arc::new(Handler2::new()) as Arc<dyn fbc_starter::KafkaMessageHandler>,
    ///     ];
    ///     builder
    ///         .with_kafka_handlers(handlers)
    ///         .http_router(routes)
    /// })
    /// ```
    #[cfg(feature = "consumer")]
    pub fn with_kafka_handlers(
        mut self,
        handlers: Vec<Arc<dyn crate::messaging::KafkaMessageHandler>>,
    ) -> Self {
        self.kafka_handlers.extend(handlers);
        self
    }

    /// 设置 HTTP 路由
    pub fn http_router(mut self, router: AxumRouter) -> Self {
        self.http_router = Some(router);
        self
    }

    /// 设置 gRPC 路由（仅在启用 grpc 特性时可用）
    #[cfg(feature = "grpc")]
    pub fn grpc_router(mut self, router: tonic::service::Routes) -> Self {
        self.grpc_router = Some(router);
        self
    }

    /// 获取配置的引用
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// 获取应用状态的引用
    pub fn state(&self) -> &Arc<AppState> {
        &self.app_state
    }

    /// 构建 Server
    fn build(self) -> Server {
        Server {
            config: self.config,
            app_state: self.app_state,
            http_router: self.http_router,
            #[cfg(feature = "grpc")]
            grpc_router: self.grpc_router,
        }
    }
}

impl Server {
    /// 启动服务器（使用闭包配置方式）
    ///
    /// # 参数
    /// - `configure`: 配置闭包，接收 ServerBuilder 用于配置路由
    ///
    /// # 示例
    /// ```rust,no_run
    /// use fbc_starter::{Config, Server};
    /// use axum::{routing::get, Router};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// Server::run(|builder| {
    ///     // 获取配置
    ///     let config = builder.config();
    ///     
    ///     // 配置 HTTP 路由
    ///     let http_router = Router::new()
    ///         .route("/api/users", get(|| async { "users" }));
    ///     
    ///     // 配置 gRPC 路由（如果启用）
    ///     #[cfg(feature = "grpc")]
    ///     let grpc_router = tonic::transport::Server::builder()
    ///         .add_service(MyServiceServer::new(MyServiceImpl));
    ///     
    ///     builder
    ///         .http_router(http_router)
    ///         #[cfg(feature = "grpc")]
    ///         .grpc_router(grpc_router)
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run<F>(configure: F) -> AppResult<()>
    where
        F: FnOnce(ServerBuilder) -> ServerBuilder,
    {
        // 加载配置
        let config = Config::from_env().unwrap_or_else(|e| {
            eprintln!("⚠ 警告: 无法从环境变量加载配置: {}, 使用默认配置", e);
            Config::default()
        });

        Self::run_with_config(config, configure).await
    }

    /// 使用指定配置启动服务器
    ///
    /// # 参数
    /// - `config`: 服务器配置
    /// - `configure`: 配置闭包，接收 ServerBuilder 用于配置路由
    pub async fn run_with_config<F>(config: Config, configure: F) -> AppResult<()>
    where
        F: FnOnce(ServerBuilder) -> ServerBuilder,
    {
        // 初始化日志
        crate::init_logging(&config).map_err(|e| anyhow::anyhow!("日志初始化失败: {}", e))?;

        tracing::info!(
            "启动 {} v{}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        // 初始化 Nacos（如果配置了且启用了 nacos 特性）
        #[cfg(feature = "nacos")]
        if let Some(ref nacos_config) = config.nacos {
            crate::nacos::init_nacos(nacos_config)
                .await
                .map_err(|e| anyhow::anyhow!("Nacos 初始化失败: {}", e))?;

            // 注册服务
            crate::nacos::register_service(nacos_config, &config.server)
                .await
                .map_err(|e| anyhow::anyhow!("Nacos 服务注册失败: {}", e))?;

            // 订阅服务
            crate::nacos::subscribe_services(nacos_config)
                .await
                .map_err(|e| anyhow::anyhow!("Nacos 服务订阅失败: {}", e))?;

            // 订阅配置
            crate::nacos::subscribe_configs(nacos_config)
                .await
                .map_err(|e| anyhow::anyhow!("Nacos 配置订阅失败: {}", e))?;
        }
        // 初始化应用状态（不包括消费者组，因为需要在闭包中注册 handlers 后才能初始化）
        let app_state = Self::init_app_state(&config).await?;

        // 创建初始的 builder（不初始化 app_state）
        let mut builder = ServerBuilder::new(config.clone(), Arc::new(app_state));

        // 使用闭包配置服务器（此时可以注册 Kafka handlers）
        builder = configure(builder);

        // 初始化 Kafka 消费者组（需要在 handlers 注册后执行）
        #[cfg(feature = "consumer")]
        {
            builder.app_state = Arc::new(
                Self::init_kafka_consumers(
                    &config,
                    &builder.kafka_handlers,
                    builder.app_state.clone(),
                )
                .await?,
            );
        }

        let server = builder.build();

        // 启动服务器
        server.start_internal().await
    }

    /// 初始化应用状态（不包括 Kafka 消费者组）
    ///
    /// Kafka 消费者组需要在 handlers 注册后才能初始化，因此单独处理
    #[allow(unused_variables)] // config 在条件编译中可能看起来未使用，但实际上被使用了
    async fn init_app_state(config: &Config) -> AppResult<AppState> {
        let app_state = AppState::new();

        // 初始化数据库（如果配置了且启用了 mysql/postgres/sqlite 任一特性）
        #[cfg(any(feature = "mysql", feature = "postgres", feature = "sqlite"))]
        let app_state = if let Some(ref db_config) = config.database {
            let pools = crate::database::init_database(db_config).await?;
            app_state.with_database_pools(pools)
        } else {
            app_state
        };

        // 初始化 Redis（如果配置了且启用了 redis 特性）
        #[cfg(feature = "redis")]
        let app_state = if let Some(ref redis_config) = config.redis {
            let pool = crate::cache::redis::init_redis(
                &redis_config.url,
                redis_config.password.as_deref(),
                redis_config.pool_size,
            )
            .await?;
            app_state.with_redis(pool)
        } else {
            app_state
        };

        // Kafka 消息代理初始化（按需初始化 producer 和 consumer）
        #[cfg(feature = "kafka")]
        let app_state = if let Some(ref kafka_config) = config.kafka {
            // 按需初始化 Producer（仅当启用 producer 特性且配置了 producer 时）
            #[cfg(feature = "producer")]
            let state = if let Some(ref producer_config) = kafka_config.producer {
                use crate::messaging::kafka::KafkaProducer;

                let producer = Arc::new(
                    KafkaProducer::new(&kafka_config.brokers, producer_config)
                        .map_err(|e| anyhow::anyhow!("Kafka 生产者初始化失败: {}", e))?,
                )
                    as Arc<dyn crate::messaging::MessageProducer + Send + Sync>;
                app_state.with_message_producer(producer)
            } else {
                app_state
            };
            #[cfg(not(feature = "producer"))]
            let state = app_state;

            state
        } else {
            app_state
        };

        Ok(app_state)
    }

    /// 初始化 Kafka 消费者组
    ///
    /// 此方法需要在 handlers 注册后才能调用，因为需要根据 handlers 来创建消费者
    #[cfg(feature = "consumer")]
    async fn init_kafka_consumers(
        config: &Config,
        kafka_handlers: &[Arc<dyn crate::messaging::KafkaMessageHandler>],
        mut app_state: Arc<AppState>,
    ) -> AppResult<AppState> {
        use crate::messaging::{kafka, KafkaMessageRouter};
        use std::sync::Arc;

        // 检查是否配置了 Kafka 和 Consumer
        if let Some(ref kafka_config) = config.kafka {
            if let Some(ref consumer_config) = kafka_config.consumer {
                // 如果有注册的 handlers，自动订阅并设置路由
                if !kafka_handlers.is_empty() {
                    let router = Arc::new(KafkaMessageRouter::new(kafka_handlers.to_vec()));
                    let subscribe_topics = router.get_subscribe_topics();

                    if !subscribe_topics.is_empty() {
                        // 根据 handlers 创建多个按 group_id 分组的 consumer
                        let consumers_by_group = kafka::create_consumers_from_handlers(
                            &kafka_config.brokers,
                            consumer_config,
                            kafka_handlers,
                        )?;

                        let router_clone = router.clone();
                        let handler = Arc::new(move |message: crate::messaging::Message| {
                            let router = router_clone.clone();
                            tokio::spawn(async move {
                                router.dispatch(message).await;
                            });
                        });

                        // 为每个 group_id 的 consumer 订阅对应的 topics
                        let mut first_consumer: Option<
                            Arc<dyn crate::messaging::MessageConsumer + Send + Sync>,
                        > = None;
                        for (group_id, (consumer, topics)) in consumers_by_group {
                            // 过滤出该 consumer 需要订阅的 topics（与 handlers 注册的 topics 的交集）
                            let topics_to_subscribe: Vec<String> = topics
                                .into_iter()
                                .filter(|t| subscribe_topics.contains(t))
                                .collect();

                            if !topics_to_subscribe.is_empty() {
                                let consumer_arc = Arc::new(consumer)
                                    as Arc<dyn crate::messaging::MessageConsumer + Send + Sync>;

                                // 保存第一个 consumer 用于 state（向后兼容）
                                if first_consumer.is_none() {
                                    first_consumer = Some(consumer_arc.clone());
                                }

                                consumer_arc
                                    .subscribe_topics(topics_to_subscribe.clone(), handler.clone())
                                    .await
                                    .map_err(|e| {
                                        anyhow::anyhow!(
                                            "Kafka 订阅失败 (group: {}): {}",
                                            group_id,
                                            e
                                        )
                                    })?;

                                tracing::info!(
                                    "✅ Kafka Consumer 初始化成功 (group: {}, 订阅 {} 个 topics: {:?})",
                                    group_id,
                                    topics_to_subscribe.len(),
                                    topics_to_subscribe
                                );
                            }
                        }

                        // 向后兼容：将第一个 consumer 设置到 state 中
                        if let Some(consumer) = first_consumer {
                            app_state =
                                Arc::new((*app_state).clone().with_message_consumer(consumer));
                        }
                    } else {
                        tracing::warn!(
                            "⚠️ Kafka Consumer 已初始化，但没有 handler 注册任何 topics"
                        );
                    }
                } else {
                    tracing::info!("ℹ️ 未注册 handlers，跳过 Kafka Consumer 初始化");
                }
            }
        }

        Ok((*app_state).clone())
    }

    /// 创建基础路由（包含健康检查等系统路由）
    fn create_base_router(&self) -> AxumRouter {
        let state = self.app_state.clone();

        Router::new()
            .route("/", get(root))
            .route("/health", get(health_check))
            .fallback(|| async {
                (
                    StatusCode::NOT_FOUND,
                    Json(json!({
                        "error": "Not found",
                        "status": 404
                    })),
                )
            })
            .layer(
                ServiceBuilder::new()
                    .layer(CompressionLayer::new())
                    .into_inner(),
            )
            .layer(create_cors_layer(&self.config))
            .with_state(state)
    }

    /// 创建完整路由（合并基础路由、用户自定义路由和 gRPC 服务）
    async fn create_router(&mut self) -> AxumRouter {
        let base_router = self.create_base_router();

        // 合并用户提供的 HTTP 路由
        let router = if let Some(custom) = self.http_router.take() {
            base_router.merge(custom)
        } else {
            base_router
        };

        // 如果启用了 gRPC 特性，添加 gRPC 服务
        #[cfg(feature = "grpc")]
        let router = self.add_grpc_services(router).await;

        // 全局中间件：合并后添加，确保覆盖所有路由（用户路由 + 基础路由）
        // 注意：axum 的 .merge() 不会将原 Router 的 layer 应用到被合并的路由，
        //       所以必须在合并后统一添加。
        let router = router
            .layer(axum::middleware::from_fn(request_logging_middleware))
            .layer(axum::middleware::from_fn(
                crate::auth::user_context_middleware,
            ));

        // 如果配置了上下文路径，则嵌套到该路径下
        if let Some(ref context_path) = self.config.server.context_path {
            use std::borrow::Cow;
            let path: Cow<'_, str> = if context_path.starts_with('/') {
                Cow::Borrowed(context_path.as_str())
            } else {
                tracing::warn!("上下文路径 '{}' 应该以 '/' 开头，已自动修正", context_path);
                Cow::Owned(format!("/{}", context_path))
            };
            Router::new().nest(path.as_ref(), router)
        } else {
            router
        }
    }

    /// 添加 gRPC 服务到路由（仅在启用 grpc 特性时可用）
    ///
    /// 使用 axum 的 fallback_service 来托管 gRPC 服务
    /// HTTP 路由优先匹配，未匹配的请求由 gRPC 服务处理
    #[cfg(feature = "grpc")]
    async fn add_grpc_services(&mut self, router: AxumRouter) -> AxumRouter {
        if let Some(grpc_router) = self.grpc_router.take() {
            tracing::info!("集成 gRPC 服务到 axum");

            // tonic 0.13 原生支持 axum 0.8，直接转换为 axum Router
            let grpc_axum_router = grpc_router.into_axum_router();

            // 使用 fallback_service 托管 gRPC 服务
            // HTTP 路由优先匹配，未匹配的请求由 gRPC 服务处理
            router.fallback_service(grpc_axum_router)
        } else {
            router
        }
    }

    /// 内部启动方法（被 start_internal 调用）
    async fn start_internal(mut self) -> AppResult<()> {
        let addr = self.config.server.socket_addr()?;
        let app = self.create_router().await;

        let base_url = if let Some(ref context_path) = self.config.server.context_path {
            format!("http://{}{}", addr, context_path)
        } else {
            format!("http://{}", addr)
        };

        info!("服务器启动在 {}", base_url);
        info!("健康检查: {}/health", base_url);

        #[cfg(feature = "grpc")]
        if self.grpc_router.is_some() {
            info!("gRPC 服务已启用");
        }

        let listener = tokio::net::TcpListener::bind(&addr).await?;

        // 保存配置副本用于优雅关闭时 Nacos 注销
        #[cfg(feature = "nacos")]
        let shutdown_config = self.config.clone();

        // 使用 into_make_service_with_connect_info 以支持 ConnectInfo extractor
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await?;

        // 优雅关闭后：执行 Nacos 服务注销
        #[cfg(feature = "nacos")]
        if let Some(ref nacos_config) = shutdown_config.nacos {
            info!("正在从 Nacos 注销服务...");
            match crate::nacos::deregister_service(nacos_config, &shutdown_config.server).await {
                Ok(_) => info!("Nacos 服务注销成功"),
                Err(e) => warn!("Nacos 服务注销失败（将由心跳超时自动下线）: {}", e),
            }
        }

        info!("服务器已关闭");
        Ok(())
    }
}

/// 优雅关闭信号处理
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("无法安装 Ctrl+C 信号处理器");
        info!("收到 Ctrl+C 信号，开始优雅关闭...");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("无法安装 SIGTERM 信号处理器")
            .recv()
            .await;
        warn!("收到 SIGTERM 信号，开始优雅关闭...");
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("服务器正在关闭...");
}
