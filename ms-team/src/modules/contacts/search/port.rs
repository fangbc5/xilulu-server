use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// 搜索文档（胖文档设计，包含展示所需的全部字段）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    /// 员工 ID（作为 Meilisearch 主键）
    pub id: i64,
    /// 租户 ID
    pub tenant_id: i64,
    /// 组织 ID
    pub org_id: i64,
    /// 员工工号
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_no: Option<String>,
    /// 员工姓名
    pub name: String,
    /// 员工头像
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    /// 工作手机
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<String>,
    /// 工作邮箱
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// 部门内职位（如：资深工程师）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department_title: Option<String>,
    /// 主部门名称
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_department_name: Option<String>,
    /// 状态：0-离职 1-在职 2-试用期 3-停薪留职
    pub status: i16,
}

/// 搜索条件
#[derive(Debug, Clone)]
pub struct SearchCriteria {
    /// 搜索关键词
    pub keyword: String,
    /// 组织 ID
    pub org_id: i64,
    /// 租户 ID
    pub tenant_id: i64,
    /// 偏移量（offset 分页）
    pub offset: u32,
    /// 请求数量（含过采样）
    pub limit: u32,
}

/// 搜索结果
#[derive(Debug, Clone)]
pub struct SearchDocumentResult {
    /// 匹配的员工文档列表
    pub items: Vec<SearchDocument>,
    /// 估算命中总数
    pub estimated_total: u64,
}

/// 员工搜索端口 — 业务层唯一的搜索契约
///
/// 业务层不直接依赖任何搜索引擎 SDK，
/// 仅通过此 trait 进行索引和查询操作。
#[async_trait]
pub trait EmployeeSearchPort: Send + Sync {
    /// 搜索 → 返回匹配的完整文档列表 + 估算总数
    async fn search(&self, criteria: SearchCriteria) -> anyhow::Result<SearchDocumentResult>;

    /// 索引/更新文档（以 id 为主键覆盖写，保证幂等）
    async fn index(&self, doc: SearchDocument) -> anyhow::Result<()>;

    /// 批量索引文档
    async fn batch_index(&self, docs: Vec<SearchDocument>) -> anyhow::Result<()>;

    /// 删除文档
    async fn delete(&self, id: i64) -> anyhow::Result<()>;
}
