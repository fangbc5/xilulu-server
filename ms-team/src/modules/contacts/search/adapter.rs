use async_trait::async_trait;
use meilisearch_sdk::client::Client;
use tracing::{info, warn};

use super::port::{EmployeeSearchPort, SearchCriteria, SearchDocument, SearchDocumentResult};

/// Meilisearch 索引名称
const INDEX_NAME: &str = "employees";

/// Meilisearch 适配器 — EmployeeSearchPort 的实现
pub struct MeilisearchAdapter {
    client: Client,
}

impl MeilisearchAdapter {
    /// 创建适配器实例并初始化索引配置
    pub fn new(url: &str, api_key: &str) -> anyhow::Result<Self> {
        let client = Client::new(url, Some(api_key))?;

        // 通过后台异步任务下发索引配置，避免阻塞启动
        let client_clone = client.clone();
        tokio::spawn(async move {
            // 确保索引存在
            let task = client_clone.create_index(INDEX_NAME, Some("id")).await;
            match task {
                Ok(task_info) => {
                    info!(
                        "Meilisearch 索引 '{}' 创建任务已提交: {:?}",
                        INDEX_NAME, task_info
                    );
                }
                Err(e) => {
                    // 索引可能已存在，忽略错误
                    warn!("Meilisearch 索引创建提示（可能已存在）: {}", e);
                }
            }

            // 配置可搜索字段
            let index = client_clone.index(INDEX_NAME);
            let _ = index
                .set_searchable_attributes([
                    "name",
                    "mobile",
                    "email",
                    "employee_no",
                    "department_title",
                ])
                .await;

            // 配置可过滤字段
            let _ = index
                .set_filterable_attributes(["tenant_id", "org_id", "status"])
                .await;

            // 配置可排序字段
            let _ = index
                .set_sortable_attributes(["name", "employee_no"])
                .await;

            info!("Meilisearch 索引 '{}' 属性配置下发完成", INDEX_NAME);
        });

        Ok(Self { client })
    }
}

#[async_trait]
impl EmployeeSearchPort for MeilisearchAdapter {
    /// 搜索员工
    async fn search(&self, criteria: SearchCriteria) -> anyhow::Result<SearchDocumentResult> {
        let index = self.client.index(INDEX_NAME);

        // 构建过滤条件
        let mut filters = Vec::new();
        filters.push(format!("tenant_id = {}", criteria.tenant_id));
        filters.push(format!("org_id = {}", criteria.org_id));
        // 只搜在职员工
        filters.push("status = 1".to_string());
        let filter_str = filters.join(" AND ");

        let mut search = index.search();
        search.with_query(&criteria.keyword);
        search.with_offset(criteria.offset as usize);
        search.with_limit(criteria.limit as usize);
        search.with_filter(&filter_str);

        let results = search.execute::<SearchDocument>().await?;

        let hits: Vec<SearchDocument> = results.hits.into_iter().map(|h| h.result).collect();

        Ok(SearchDocumentResult {
            items: hits,
            estimated_total: results.estimated_total_hits.unwrap_or(0) as u64,
        })
    }

    /// 索引/更新文档
    async fn index(&self, doc: SearchDocument) -> anyhow::Result<()> {
        let index = self.client.index(INDEX_NAME);
        index.add_documents(&[doc], Some("id")).await?;
        Ok(())
    }

    /// 批量索引文档
    async fn batch_index(&self, docs: Vec<SearchDocument>) -> anyhow::Result<()> {
        if docs.is_empty() {
            return Ok(());
        }
        let index = self.client.index(INDEX_NAME);
        index.add_documents(&docs, Some("id")).await?;
        Ok(())
    }

    /// 删除文档
    async fn delete(&self, id: i64) -> anyhow::Result<()> {
        let index = self.client.index(INDEX_NAME);
        index.delete_document(&id.to_string()).await?;
        Ok(())
    }
}
