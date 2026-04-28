use sqlxplus::{Crud, DbPool, QueryBuilder};
use std::sync::Arc;

use super::model::entity::*;

/// 内容 Repository — 仅实现 CRUD trait 不提供的方法
pub struct ContentRepo;

impl ContentRepo {
    /// 根据 content_type 查询 Schema（需启用状态）
    pub async fn find_schema(
        pool: &Arc<DbPool>,
        content_type: &str,
    ) -> anyhow::Result<Option<ContentSchema>> {
        let builder = QueryBuilder::new("SELECT * FROM `content_schema`")
            .and_eq("content_type", content_type)
            .and_eq("status", 1);
        let result = ContentSchema::find_one(pool.mysql_pool(), builder).await?;
        Ok(result)
    }

    /// 按 ID 查询内容主表
    pub async fn find_main_by_id(
        pool: &Arc<DbPool>,
        id: i64,
    ) -> anyhow::Result<Option<ContentMain>> {
        let builder = QueryBuilder::new("SELECT * FROM `content_main`").and_eq("id", id);
        let result = ContentMain::find_one(pool.mysql_pool(), builder).await?;
        Ok(result)
    }

    /// 按 content_id 查询内容详情
    pub async fn find_detail_by_content_id(
        pool: &Arc<DbPool>,
        content_id: i64,
    ) -> anyhow::Result<Option<ContentDetail>> {
        let builder =
            QueryBuilder::new("SELECT * FROM `content_detail`").and_eq("content_id", content_id);
        let result = ContentDetail::find_one(pool.mysql_pool(), builder).await?;
        Ok(result)
    }

    /// 按 content_id 查询统计数据
    pub async fn find_stats_by_content_id(
        pool: &Arc<DbPool>,
        content_id: i64,
    ) -> anyhow::Result<Option<ContentStats>> {
        let builder =
            QueryBuilder::new("SELECT * FROM `content_stats`").and_eq("content_id", content_id);
        let result = ContentStats::find_one(pool.mysql_pool(), builder).await?;
        Ok(result)
    }

    /// 按 target_id 查询关系列表（游标分页，倒序）
    pub async fn find_relations_by_target(
        pool: &Arc<DbPool>,
        target_id: i64,
        relation_type: &str,
        cursor: Option<u32>,
        page_size: u32,
    ) -> anyhow::Result<(Vec<ContentRelation>, Option<u32>, bool)> {
        let mut builder = QueryBuilder::new("SELECT * FROM `content_relation`")
            .and_eq("target_id", target_id)
            .and_eq("relation_type", relation_type)
            .order_by("created_at", false);

        // 游标分页：用 offset 模拟（cursor = offset）
        let offset = cursor.unwrap_or(0);
        let limit = page_size;

        let result =
            ContentRelation::paginate(pool.mysql_pool(), builder, (offset / limit) + 1, limit)
                .await?;

        let total = result.total;
        let next_offset = offset + result.items.len() as u32;
        let has_next = (next_offset as i64) < total;
        let next_cursor = if has_next { Some(next_offset) } else { None };

        Ok((result.items, next_cursor, has_next))
    }

    /// 计算关系深度（逐级追溯，用于限制评论深度）
    pub async fn count_relation_depth(
        pool: &Arc<DbPool>,
        source_id: i64,
        relation_type: &str,
    ) -> anyhow::Result<u8> {
        let mut current_id = source_id;
        let mut depth: u8 = 0;

        loop {
            let builder = QueryBuilder::new("SELECT * FROM `content_relation`")
                .and_eq("source_id", current_id)
                .and_eq("relation_type", relation_type);
            let parent = ContentRelation::find_one(pool.mysql_pool(), builder).await?;

            match parent {
                Some(rel) => {
                    depth += 1;
                    if depth >= 10 {
                        // 安全阀，防止无限循环
                        break;
                    }
                    current_id = rel.target_id.unwrap_or_default();
                }
                None => break,
            }
        }

        Ok(depth)
    }
}
