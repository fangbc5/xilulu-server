use std::sync::Arc;
use tracing::{info, warn};
use xxljob_sdk_rs::{XxlClientBuilder, client::client::XxlClient};

/// 分布式调度任务配置表
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobConfig {
    pub admin_address: String,
    pub access_token: String,
    pub app_name: String,
    pub executor_port: Option<u16>,
    pub log_path: Option<String>,
}

/// 快速初始化 XXL-JOB 兼容的执行器客户端（如 Ratchjob）
/// 会在后台异步启动 Actix Web 服务器，接收调度中心指令
pub fn init_job_client(config: JobConfig) -> anyhow::Result<Arc<XxlClient>> {
    info!("正在初始化分布式任务调度客户端...");
    info!("API 地址: {}, 执行器名称: {}", config.admin_address, config.app_name);

    if config.access_token.is_empty() {
        warn!("当前 Job 配置的 access_token 为空，请注意安全防护！");
    }

    let mut builder = XxlClientBuilder::new(config.admin_address.clone())
        .set_access_token(config.access_token)
        .set_app_name(config.app_name.clone());

    if let Some(port) = config.executor_port {
        builder = builder.set_port(port);
    }
    
    if let Some(log_path) = config.log_path {
        builder = builder.set_log_path(log_path);
    } else {
        builder = builder.set_log_path("logs/xxljob".to_string());
    }

    // build() 方法内会自动寻找可用端口并在独立环境启动监听服务器
    let client = builder.build().map_err(|e| anyhow::anyhow!("Job 客户端构建失败: {:?}", e))?;
    info!("分布式调度器 {} 启动成功，准备接收命令", config.app_name);
    Ok(client)
}
