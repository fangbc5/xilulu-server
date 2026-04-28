use sqlxplus::{Crud, DbPool};
use std::sync::Arc;
use tracing::{info, warn};
use uuid::Uuid;

use super::model::domain::{self, count_words, extract_body_text, MAX_RELATION_DEPTH};
use super::model::dto::*;
use super::model::entity::*;
use super::repository::ContentRepo;
use super::search::port::{SearchDocument, SearchPort};
use crate::error::ContentError;

/// 内容服务 — 核心业务逻辑
pub struct ContentService {
    /// 数据库连接池
    db_pool: Arc<DbPool>,
    /// 搜索端口（防腐层）
    search_port: Arc<dyn SearchPort>,
}

impl ContentService {
    /// 创建服务实例
    pub fn new(db_pool: Arc<DbPool>, search_port: Arc<dyn SearchPort>) -> Self {
        Self {
            db_pool,
            search_port,
        }
    }

    /// 创建内容
    #[tracing::instrument(skip(self, req), fields(author_id = %author_id))]
    pub async fn create_content(
        &self,
        author_id: i64,
        req: CreateContentReq,
    ) -> Result<i64, ContentError> {
        // 1. 校验 content_type 是否已注册
        let _schema = ContentRepo::find_schema(&self.db_pool, &req.content_type)
            .await
            .map_err(|e| ContentError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                ContentError::SchemaNotFound(format!(
                    "内容类型 '{}' 未注册或已禁用",
                    req.content_type
                ))
            })?;

        // 2. 校验 ext_data（Phase 1 简易校验：仅检查是否为合法 JSON Object）
        if let Some(ref ext) = req.ext_data {
            if !ext.is_object() {
                return Err(ContentError::ExtDataInvalid(
                    "ext_data 必须是 JSON Object".to_string(),
                ));
            }
        }

        // 3. 提取纯文本和字数
        let body_text = extract_body_text(&req.body);
        let word_count = count_words(&req.body);

        // 4. 准备字段
        let now = chrono::Utc::now().timestamp_millis();
        let content_uuid = Uuid::new_v4().to_string();

        // 5. 事务写入（id 由数据库自增生成）
        let content_id: i64 = sqlxplus::with_transaction(&self.db_pool, |tx| {
            let content_uuid = content_uuid.clone();
            let req_content_type = req.content_type.clone();
            let req_title = req.title.clone();
            let req_summary = req.summary.clone();
            let req_cover = req.cover_image.clone();
            let req_body = serde_json::to_value(&req.body).unwrap_or_default();
            let req_ext = req.ext_data.clone();
            let body_text = body_text.clone();
            let visibility = req.visibility.unwrap_or(0);

            Box::pin(async move {
                // 写入 content_main（id=None 由 DB 自增）
                let main = ContentMain {
                    id: None,
                    content_id: Some(content_uuid),
                    content_type: Some(req_content_type),
                    author_id: Some(author_id),
                    status: Some(domain::content_status::DRAFT),
                    visibility: Some(visibility),
                    pinned: Some(0),
                    published_at: Some(0),
                    created_at: Some(now),
                    updated_at: Some(now),
                    version: Some(1),
                };
                let insert_id = main.insert(tx.as_mysql_executor()).await? as i64;

                // 写入 content_detail
                let detail = ContentDetail {
                    id: None,
                    content_id: Some(insert_id),
                    title: req_title,
                    summary: req_summary,
                    cover_image: req_cover,
                    body: Some(req_body),
                    body_text: if body_text.is_empty() {
                        None
                    } else {
                        Some(body_text)
                    },
                    ext_data: req_ext,
                    word_count: Some(word_count),
                };
                detail.insert(tx.as_mysql_executor()).await?;

                // 写入 content_stats（初始化为 0）
                let stats = ContentStats {
                    id: None,
                    content_id: Some(insert_id),
                    view_count: Some(0),
                    like_count: Some(0),
                    comment_count: Some(0),
                    share_count: Some(0),
                    collect_count: Some(0),
                };
                stats.insert(tx.as_mysql_executor()).await?;

                Ok(insert_id)
            })
        })
        .await
        .map_err(|e| ContentError::InternalError(format!("事务写入失败: {}", e)))?;

        info!(content_id = %content_id, action = "create", "内容创建成功");
        Ok(content_id)
    }

    /// 获取内容详情（从 DB 查询，保证绝对实时）
    #[tracing::instrument(skip(self), fields(content_id = %id))]
    pub async fn get_content_detail(&self, id: i64) -> Result<ContentDetailResp, ContentError> {
        // 查询主表
        let main = ContentRepo::find_main_by_id(&self.db_pool, id)
            .await
            .map_err(|e| ContentError::InternalError(e.to_string()))?
            .ok_or_else(|| ContentError::ContentNotFound(format!("内容 {} 不存在", id)))?;

        // 查询详情
        let detail = ContentRepo::find_detail_by_content_id(&self.db_pool, id)
            .await
            .map_err(|e| ContentError::InternalError(e.to_string()))?
            .ok_or_else(|| ContentError::ContentNotFound(format!("内容详情 {} 不存在", id)))?;

        // 查询统计
        let stats = ContentRepo::find_stats_by_content_id(&self.db_pool, id)
            .await
            .map_err(|e| ContentError::InternalError(e.to_string()))?
            .unwrap_or_default();

        Ok(ContentDetailResp {
            id: main.id.unwrap_or_default(),
            content_id: main.content_id.unwrap_or_default(),
            content_type: main.content_type.unwrap_or_default(),
            author_id: main.author_id.unwrap_or_default(),
            status: main.status.unwrap_or_default(),
            visibility: main.visibility.unwrap_or_default(),
            pinned: main.pinned.unwrap_or_default(),
            title: detail.title,
            summary: detail.summary,
            cover_image: detail.cover_image,
            body: detail.body.unwrap_or(serde_json::json!([])),
            ext_data: detail.ext_data,
            word_count: detail.word_count.unwrap_or_default(),
            stats: ContentStatsResp {
                view_count: stats.view_count.unwrap_or_default(),
                like_count: stats.like_count.unwrap_or_default(),
                comment_count: stats.comment_count.unwrap_or_default(),
                share_count: stats.share_count.unwrap_or_default(),
                collect_count: stats.collect_count.unwrap_or_default(),
            },
            published_at: main.published_at.unwrap_or_default(),
            created_at: main.created_at.unwrap_or_default(),
            updated_at: main.updated_at.unwrap_or_default(),
            version: main.version.unwrap_or_default(),
        })
    }

    /// 变更内容状态（发布/下架/删除）
    #[tracing::instrument(skip(self, req), fields(content_id = %id))]
    pub async fn change_status(&self, id: i64, req: ChangeStatusReq) -> Result<(), ContentError> {
        let now = chrono::Utc::now().timestamp_millis();

        let mut main = ContentRepo::find_main_by_id(&self.db_pool, id)
            .await
            .map_err(|e| ContentError::InternalError(e.to_string()))?
            .ok_or_else(|| ContentError::ContentNotFound(format!("内容 {} 不存在", id)))?;

        // 乐观锁校验
        if main.version.unwrap_or(0) != req.version {
            return Err(ContentError::VersionConflict(
                "版本号不匹配，内容可能已被其他人修改".to_string(),
            ));
        }

        main.status = Some(req.status);
        main.updated_at = Some(now);
        main.version = Some(req.version + 1);

        // 如果是发布操作且 published_at 为 0，设置发布时间
        if req.status == domain::content_status::PUBLISHED && main.published_at.unwrap_or(0) == 0 {
            main.published_at = Some(now);
        }

        main.update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| ContentError::InternalError(format!("更新状态失败: {}", e)))?;

        // 同步双写：更新搜索索引
        if req.status == domain::content_status::PUBLISHED {
            if let Err(e) = self.sync_to_search(id).await {
                warn!(content_id = %id, error = ?e, "同步搜索索引失败（非阻塞）");
            }
        } else if req.status == domain::content_status::DELETED {
            if let Err(e) = self.search_port.delete(id).await {
                warn!(content_id = %id, error = ?e, "删除搜索索引失败（非阻塞）");
            }
        }

        info!(content_id = %id, status = req.status, action = "change_status", "内容状态变更");
        Ok(())
    }

    /// 逻辑删除内容
    #[tracing::instrument(skip(self), fields(content_id = %id))]
    pub async fn delete_content(&self, id: i64) -> Result<(), ContentError> {
        let mut main = ContentRepo::find_main_by_id(&self.db_pool, id)
            .await
            .map_err(|e| ContentError::InternalError(e.to_string()))?
            .ok_or_else(|| ContentError::ContentNotFound(format!("内容 {} 不存在", id)))?;

        let now = chrono::Utc::now().timestamp_millis();
        main.status = Some(domain::content_status::DELETED);
        main.updated_at = Some(now);
        main.update(self.db_pool.mysql_pool())
            .await
            .map_err(|e| ContentError::InternalError(format!("删除失败: {}", e)))?;

        // 从搜索索引中移除
        if let Err(e) = self.search_port.delete(id).await {
            warn!(content_id = %id, error = ?e, "删除搜索索引失败（非阻塞）");
        }

        info!(content_id = %id, action = "delete", "内容逻辑删除");
        Ok(())
    }

    /// 搜索内容（查 Meilisearch，游标分页）
    pub async fn search_contents(
        &self,
        req: SearchContentReq,
    ) -> Result<(Vec<SearchDocument>, Option<u32>, bool, u64), ContentError> {
        use super::search::port::SearchCriteria;

        let page_size = req.page.page_size.min(50); // 限制单次最多 50 条
        let cursor = req.page.cursor;

        let criteria = SearchCriteria {
            keyword: req.keyword,
            content_type: req.content_type,
            author_id: req.author_id,
            sort_by: req.sort_by,
            offset: cursor.unwrap_or(0),
            limit: page_size,
        };

        let result = self
            .search_port
            .search(criteria)
            .await
            .map_err(|e| ContentError::InternalError(format!("搜索失败: {}", e)))?;

        let next_offset = cursor.unwrap_or(0) + result.hits.len() as u32;
        let has_next = (next_offset as u64) < result.total;
        let next_cursor = if has_next { Some(next_offset) } else { None };

        Ok((result.hits, next_cursor, has_next, result.total))
    }

    /// 创建关系
    #[tracing::instrument(skip(self, req), fields(source_id = %source_id))]
    pub async fn create_relation(
        &self,
        source_id: i64,
        req: CreateRelationReq,
    ) -> Result<i64, ContentError> {
        // 校验 source 和 target 是否存在
        ContentRepo::find_main_by_id(&self.db_pool, source_id)
            .await
            .map_err(|e| ContentError::InternalError(e.to_string()))?
            .ok_or_else(|| ContentError::ContentNotFound(format!("源内容 {} 不存在", source_id)))?;

        ContentRepo::find_main_by_id(&self.db_pool, req.target_id)
            .await
            .map_err(|e| ContentError::InternalError(e.to_string()))?
            .ok_or_else(|| {
                ContentError::ContentNotFound(format!("目标内容 {} 不存在", req.target_id))
            })?;

        // 深度限制校验（仅对 reply 类型）
        if req.relation_type == "reply" {
            let depth = ContentRepo::count_relation_depth(&self.db_pool, req.target_id, "reply")
                .await
                .map_err(|e| ContentError::InternalError(e.to_string()))?;

            if depth >= MAX_RELATION_DEPTH {
                return Err(ContentError::RelationDepthExceeded(format!(
                    "回复深度已达上限 {} 层",
                    MAX_RELATION_DEPTH
                )));
            }
        }

        let relation_uuid = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp_millis();

        let relation = ContentRelation {
            id: None, // DB 自增
            relation_id: Some(relation_uuid),
            source_id: Some(source_id),
            target_id: Some(req.target_id),
            relation_type: Some(req.relation_type),
            direction: Some(req.direction),
            metadata: req.metadata,
            created_at: Some(now),
        };

        let relation_id = relation
            .insert(self.db_pool.mysql_pool())
            .await
            .map_err(|e| ContentError::InternalError(format!("创建关系失败: {}", e)))?
            as i64;

        info!(relation_id = %relation_id, action = "create_relation", "内容关系创建成功");
        Ok(relation_id)
    }

    /// 查询关系列表（游标分页）
    pub async fn get_relations(
        &self,
        target_id: i64,
        relation_type: &str,
        cursor: Option<u32>,
        page_size: u32,
    ) -> Result<(Vec<ContentRelationResp>, Option<u32>, bool), ContentError> {
        let page_size = page_size.min(20); // 关系查询单次 ≤ 20 条

        let (relations, next_cursor, has_next) = ContentRepo::find_relations_by_target(
            &self.db_pool,
            target_id,
            relation_type,
            cursor,
            page_size,
        )
        .await
        .map_err(|e| ContentError::InternalError(e.to_string()))?;

        let list = relations
            .into_iter()
            .map(|r| ContentRelationResp {
                id: r.id.unwrap_or_default(),
                relation_id: r.relation_id.unwrap_or_default(),
                source_id: r.source_id.unwrap_or_default(),
                target_id: r.target_id.unwrap_or_default(),
                relation_type: r.relation_type.unwrap_or_default(),
                direction: r.direction.unwrap_or_default(),
                metadata: r.metadata,
                created_at: r.created_at.unwrap_or_default(),
            })
            .collect();

        Ok((list, next_cursor, has_next))
    }

    /// 同步内容到搜索索引（同步双写）
    async fn sync_to_search(&self, id: i64) -> anyhow::Result<()> {
        let main = ContentRepo::find_main_by_id(&self.db_pool, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("内容不存在"))?;
        let detail = ContentRepo::find_detail_by_content_id(&self.db_pool, id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("内容详情不存在"))?;
        let stats = ContentRepo::find_stats_by_content_id(&self.db_pool, id)
            .await?
            .unwrap_or_default();

        let doc = SearchDocument {
            id: main.id.unwrap_or_default(),
            content_type: main.content_type.unwrap_or_default(),
            author_id: main.author_id.unwrap_or_default(),
            title: detail.title,
            summary: detail.summary,
            body_text: detail.body_text,
            cover_image: detail.cover_image,
            status: main.status.unwrap_or_default(),
            visibility: main.visibility.unwrap_or_default(),
            published_at: main.published_at.unwrap_or_default(),
            created_at: main.created_at.unwrap_or_default(),
            view_count: stats.view_count.unwrap_or_default(),
            like_count: stats.like_count.unwrap_or_default(),
        };

        self.search_port.index(doc).await?;
        info!(content_id = %id, action = "sync_search", "内容同步到搜索索引");
        Ok(())
    }
}
