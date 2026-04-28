use async_trait::async_trait;
use meilisearch_sdk::client::Client;
use tracing::{info, warn};

use super::port::{SearchCriteria, SearchDocument, SearchPort, SearchResult};

/// Meilisearch 索引名称
const INDEX_NAME: &str = "contents";

/// Meilisearch 适配器 — SearchPort 的当前实现
pub struct MeilisearchAdapter {
    client: Client,
}

impl MeilisearchAdapter {
    /// 创建适配器实例并初始化索引配置（通过后台异步任务下发避免阻塞启动）
    pub fn new(url: &str, api_key: &str) -> anyhow::Result<Self> {
        let client = Client::new(url, Some(api_key))?;

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

            // 配置可搜索字段和可筛选字段
            let index = client_clone.index(INDEX_NAME);
            let _ = index
                .set_searchable_attributes(["title", "summary", "body_text"])
                .await;
            let _ = index
                .set_filterable_attributes([
                    "content_type",
                    "author_id",
                    "status",
                    "visibility",
                    "published_at",
                ])
                .await;
            let _ = index
                .set_sortable_attributes(["published_at", "created_at", "like_count", "view_count"])
                .await;
            info!("Meilisearch 索引 '{}' 属性配置下发完成", INDEX_NAME);
        });

        Ok(Self { client })
    }
}

#[async_trait]
impl SearchPort for MeilisearchAdapter {
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

    /// 搜索
    async fn search(&self, criteria: SearchCriteria) -> anyhow::Result<SearchResult> {
        let index = self.client.index(INDEX_NAME);

        let query_str = criteria.keyword.as_deref().unwrap_or("");

        let sort_parts: Vec<&str> = criteria.sort_by.split(':').collect();
        let sort_str = if sort_parts.len() == 2 {
            Some(format!("{}:{}", sort_parts[0], sort_parts[1]))
        } else {
            None
        };

        let mut filters = Vec::new();
        // 只搜已发布的内容
        filters.push("status = 2".to_string());
        filters.push("visibility = 0".to_string());
        if let Some(ref ct) = criteria.content_type {
            filters.push(format!("content_type = \"{}\"", ct));
        }
        if let Some(author_id) = criteria.author_id {
            filters.push(format!("author_id = {}", author_id));
        }
        let filter_str = if !filters.is_empty() {
            Some(filters.join(" AND "))
        } else {
            None
        };

        let mut search = index.search();
        search.with_query(query_str);
        search.with_offset(criteria.offset as usize);
        search.with_limit(criteria.limit as usize);

        if let Some(ref fs) = filter_str {
            search.with_filter(fs);
        }

        let sort_slice: Vec<&str> = if let Some(ref ss) = sort_str {
            vec![ss.as_str()]
        } else {
            vec![]
        };

        if !sort_slice.is_empty() {
            search.with_sort(&sort_slice);
        }

        let results = search.execute::<SearchDocument>().await?;

        let hits: Vec<SearchDocument> = results.hits.into_iter().map(|h| h.result).collect();

        Ok(SearchResult {
            total: results.estimated_total_hits.unwrap_or(0) as u64,
            hits,
        })
    }
}
