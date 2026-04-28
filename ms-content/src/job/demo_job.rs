use async_trait::async_trait;
use tracing::info;
use xxljob_sdk_rs::common::model::handler::{AsyncJobHandler, JobContext};

pub struct DemoJobTask;

#[async_trait]
impl AsyncJobHandler for DemoJobTask {
    async fn process(&self, mut context: JobContext) -> anyhow::Result<JobContext> {
        info!("【分布式调度执行】收到执行触发指令！");
        info!(
            "---- 任务参数 ----\n任务 ID: {}\n参数: {:?}\nLog ID: {}",
            context.job_id, context.job_param, context.log_id
        );

        // 模拟延时耗时计算
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        info!("【分布式调度执行】执行完毕并反馈成功！");
        context.handle_msg = Some("成功完成了模拟计算".to_string());
        Ok(context)
    }
}
