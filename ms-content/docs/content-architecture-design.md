# Content Core 内容管理内核 — 架构设计文档

> **服务名**：`ms-content`  
> **定位**：全平台统一内容管理内核，一套模型覆盖博客 / 论坛 / 朋友圈 / 小红书 / 商品  
> **版本**：v2.0（生产级终稿）  
> **数据库**：MySQL 8.0+  
> **搜索引擎**：Meilisearch（已部署 v1.42.1，端口 7700）  
> **消息队列**：Kafka（已部署，复用 fbc-starter 基础设施）

---

# 一、设计原则

1. **DB 只存事实，Search 只存视图** — CQRS 读写分离，互不侵入。
2. **主表轻、详情重** — 控制字段与大文本物理隔离，杜绝 MVCC 膨胀。
3. **动态但受控** — JSONB 提供灵活性，Schema 注册提供约束性，二者缺一不可。
4. **正文即协议** — Block DSL 统一所有内容形态的正文表达，前端一套渲染器通吃。
5. **万物皆可关联** — 图模型的 Relation 表取代一切硬编码的 `parent_id` / `reply_to`。
6. **内核不做分发** — 不含 Feed 流推拉、不含推荐算法、不含社交关系链。

---

# 二、架构总览

## 2.1 系统分层图

```text
┌──────────────────────────────────────────────────────┐
│                   Client / API Gateway               │
└────────────┬────────────────────────┬────────────────┘
             │                        │
        (Write)                  (Read)
             │                        │
             ▼                        ▼
┌────────────────────┐   ┌─────────────────────────┐
│  ms-content (Core) │   │  ms-content (Query)     │
│  Handler → Service │   │  SearchService          │
│  → Repository      │   │  → SearchPort (trait)   │
│  → DB Transaction  │   │    → MeilisearchAdapter │
└─────────┬──────────┘   └─────────────────────────┘
          │                          ▲
          │ (事务内写 Outbox)         │ (消费事件 → 更新索引)
          ▼                          │
┌──────────────┐            ┌────────┴───────┐
│  event_outbox│────CDC────▶│  Sync Worker   │
│  (DB 表)     │  或轮询    │  (Kafka消费者) │
└──────────────┘            └────────────────┘
```

## 2.2 数据流

### 写链路
```
Client → Handler(入参校验) → Service(Schema校验 + 组装领域对象)
  → DB Transaction {
      INSERT content_main
      INSERT content_detail
      INSERT event_outbox   ← 同一事务，原子性保证
    }
  → 返回 content_id
```

### 读链路
```
详情页：GET /contents/{id}    → 查 DB（content_main JOIN content_detail）
列表页：GET /contents/search  → 查 Meilisearch（SearchPort.search()）
```

### 同步链路
```
event_outbox → Kafka → SyncWorker → 从 DB 读完整数据 → SearchPort.index()
```

---

# 三、数据模型（共 8 张表）

## 3.1 `content_schema` — 类型约束注册表

**职责**：为每种 `content_type` 的 `ext_data` 定义合法结构，Service 层写入前强制校验。

```sql
CREATE TABLE IF NOT EXISTS `content_schema` (
    `content_type`      VARCHAR(32)  NOT NULL             COMMENT '内容类型唯一标识，如 blog / product / moment',
    `display_name`      VARCHAR(64)  NOT NULL             COMMENT '类型中文名，如 博客 / 商品',
    `schema_definition` JSON         NOT NULL             COMMENT 'ext_data 的 JSON Schema 定义',
    `status`            TINYINT      NOT NULL DEFAULT 1   COMMENT '0=禁用 1=启用',
    `created_at`        BIGINT       NOT NULL DEFAULT 0   COMMENT '创建时间',
    `updated_at`        BIGINT       NOT NULL DEFAULT 0   COMMENT '更新时间',
    PRIMARY KEY (`content_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容类型 Schema 注册表';
```

**示例数据**：
```sql
INSERT INTO content_schema (content_type, display_name, schema_definition) VALUES
('blog', '博客', '{"required":["tags"],"properties":{"tags":{"type":"array"},"reading_time":{"type":"integer"},"is_original":{"type":"boolean"}}}'),
('moment', '动态', '{"properties":{"location":{"type":"object","properties":{"name":{"type":"string"},"lat":{"type":"number"},"lng":{"type":"number"}}}}}'),
('product', '商品', '{"required":["price","currency"],"properties":{"price":{"type":"integer"},"currency":{"type":"string"},"stock":{"type":"integer"},"brand":{"type":"string"},"specs":{"type":"object"}}}');
```

---

## 3.2 `content_main` — 内容主表（轻量路由层）

**职责**：仅存储路由、控制、排序所需的标量字段。绝不放大文本。

```sql
CREATE TABLE IF NOT EXISTS `content_main` (
    `id`            BIGINT       NOT NULL                 COMMENT '雪花算法主键',
    `content_type`  VARCHAR(32)  NOT NULL                 COMMENT '内容类型 → content_schema.content_type',
    `author_id`     BIGINT       NOT NULL                 COMMENT '作者/所有者 ID',
    `status`        TINYINT      NOT NULL DEFAULT 0       COMMENT '0=草稿 1=待审核 2=已发布 3=已下架 4=已删除',
    `visibility`    TINYINT      NOT NULL DEFAULT 0       COMMENT '0=公开 1=私密 2=仅关注者可见',
    `pinned`        TINYINT      NOT NULL DEFAULT 0       COMMENT '0=普通 1=置顶',
    `published_at`  BIGINT       NOT NULL DEFAULT 0       COMMENT '发布时间（用于排序）',
    `created_at`    BIGINT       NOT NULL DEFAULT 0       COMMENT '创建时间',
    `updated_at`    BIGINT       NOT NULL DEFAULT 0       COMMENT '更新时间',
    `version`       INT          NOT NULL DEFAULT 1       COMMENT '乐观锁版本号',
    PRIMARY KEY (`id`),
    KEY `idx_author_status_pub` (`author_id`, `status`, `published_at`),
    KEY `idx_type_status_pub` (`content_type`, `status`, `published_at`),
    KEY `idx_created_at` (`created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容主表（路由与控制）';
```

> **设计意图**：对 `status` 的高频更新（审核通过、下架等）只涉及这张小行表，不会触发详情大字段的行拷贝开销。

---

## 3.3 `content_detail` — 内容详情表（大文本 + Block DSL + 扩展）

**职责**：与 `content_main` 1:1，存储标题、摘要、Block DSL 正文和类型专属扩展数据。

```sql
CREATE TABLE IF NOT EXISTS `content_detail` (
    `content_id`    BIGINT       NOT NULL                 COMMENT '→ content_main.id',
    `title`         VARCHAR(255) DEFAULT NULL             COMMENT '标题（Moment 可为空）',
    `summary`       VARCHAR(500) DEFAULT NULL             COMMENT '摘要/简介',
    `cover_image`   VARCHAR(512) DEFAULT NULL             COMMENT '封面图 OSS Key',
    `body`          JSON         NOT NULL                 COMMENT '正文 Block DSL（结构化数组）',
    `body_text`     MEDIUMTEXT   DEFAULT NULL             COMMENT '正文纯文本（用于全文检索同步，由 Service 层自动提取）',
    `ext_data`      JSON         DEFAULT NULL             COMMENT '类型专属扩展字段（写入前必须通过 Schema 校验）',
    `word_count`    INT          DEFAULT 0                COMMENT '字数统计',
    PRIMARY KEY (`content_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容详情表（Block DSL + 扩展）';
```

---

## 3.4 Block DSL 正文协议

`content_detail.body` 存储的是一个 JSON 数组，每个元素是一个 Block。所有内容形态共用这套协议，前端只需一套渲染器。

### 基础 Block 类型

| Block Type | 用途 | 结构示例 |
|:---|:---|:---|
| `text` | 纯文本/段落 | `{"type":"text","value":"今天天气不错"}` |
| `heading` | 标题 | `{"type":"heading","level":2,"value":"章节一"}` |
| `image` | 图片 | `{"type":"image","key":"oss/xxx.jpg","caption":"风景"}` |
| `gallery` | 九宫格图片组 | `{"type":"gallery","keys":["k1","k2","k3"]}` |
| `video` | 视频 | `{"type":"video","key":"oss/xxx.mp4","cover":"oss/cover.jpg"}` |
| `code` | 代码块 | `{"type":"code","lang":"rust","value":"fn main(){}"}` |
| `quote` | 引用块 | `{"type":"quote","value":"名人名言..."}` |
| `divider` | 分隔线 | `{"type":"divider"}` |

### 业务扩展 Block 类型

| Block Type | 用途 | 结构示例 |
|:---|:---|:---|
| `product_card` | 商品卡片 | `{"type":"product_card","product_id":888}` |
| `location_card` | 位置卡片 | `{"type":"location_card","name":"西湖","lat":30.2,"lng":120.1}` |
| `poll` | 投票 | `{"type":"poll","question":"你觉得？","options":["A","B"]}` |
| `link_card` | 链接预览 | `{"type":"link_card","url":"https://...","title":"..."}` |

### 完整示例：小红书图文笔记

```json
[
  { "type": "gallery", "keys": ["oss/note/img1.jpg", "oss/note/img2.jpg", "oss/note/img3.jpg"] },
  { "type": "text", "value": "第一家店氛围感直接拉满，咖啡也好喝 ☕️" },
  { "type": "location_card", "name": "魔都某探店", "lat": 31.2, "lng": 121.4 },
  { "type": "product_card", "product_id": 12345 },
  { "type": "text", "value": "强烈推荐！#周末去哪儿" }
]
```

---

## 3.5 `content_relation` — 内容关系图表

**职责**：取代一切硬编码的 `parent_id`，用有向图语义统一表达评论、回复、引用、挂载等关系。

```sql
CREATE TABLE IF NOT EXISTS `content_relation` (
    `id`            BIGINT       NOT NULL                 COMMENT '主键',
    `source_id`     BIGINT       NOT NULL                 COMMENT '发起方内容 ID',
    `target_id`     BIGINT       NOT NULL                 COMMENT '目标方内容 ID',
    `relation_type` VARCHAR(32)  NOT NULL                 COMMENT '关系类型：comment / reply / attach / quote / collection',
    `direction`     TINYINT      NOT NULL DEFAULT 1       COMMENT '0=双向 1=单向（source→target）',
    `metadata`      JSON         DEFAULT NULL             COMMENT '边属性（置顶参数、排序权重等）',
    `created_at`    BIGINT       NOT NULL DEFAULT 0       COMMENT '创建时间',
    PRIMARY KEY (`id`),
    KEY `idx_source` (`source_id`, `relation_type`),
    KEY `idx_target` (`target_id`, `relation_type`),
    UNIQUE KEY `uk_relation` (`source_id`, `target_id`, `relation_type`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容关系图表';
```

**典型用法**：
- 评论帖子：`source=评论ID, target=帖子ID, type=comment`
- 回复评论：`source=回复ID, target=评论ID, type=reply`
- 笔记挂商品：`source=笔记ID, target=商品ID, type=attach`
- 文章引用：`source=文章A, target=文章B, type=quote`
- 专栏收录：`source=专栏ID, target=文章ID, type=collection`

---

## 3.6 `content_version` — 内容版本快照

**职责**：关键更新时记录 `content_detail` 快照，支持回滚与审计。

```sql
CREATE TABLE IF NOT EXISTS `content_version` (
    `id`                BIGINT   NOT NULL                 COMMENT '主键',
    `content_id`        BIGINT   NOT NULL                 COMMENT '→ content_main.id',
    `version`           INT      NOT NULL                 COMMENT '版本号（自增）',
    `detail_snapshot`   JSON     NOT NULL                 COMMENT 'content_detail 的完整快照',
    `operator_id`       BIGINT   DEFAULT NULL             COMMENT '操作人 ID',
    `remark`            VARCHAR(255) DEFAULT NULL         COMMENT '版本备注',
    `created_at`        BIGINT   NOT NULL DEFAULT 0       COMMENT '创建时间',
    PRIMARY KEY (`id`),
    KEY `idx_content_version` (`content_id`, `version`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容版本快照';
```

---

## 3.7 `content_stats` — 统计计数表

**职责**：高频更新的计数器与低频的内容本体物理隔离，消除行锁竞争。

```sql
CREATE TABLE IF NOT EXISTS `content_stats` (
    `content_id`    BIGINT   NOT NULL                     COMMENT '→ content_main.id',
    `view_count`    BIGINT   NOT NULL DEFAULT 0           COMMENT '浏览量',
    `like_count`    INT      NOT NULL DEFAULT 0           COMMENT '点赞数',
    `comment_count` INT      NOT NULL DEFAULT 0           COMMENT '评论数',
    `share_count`   INT      NOT NULL DEFAULT 0           COMMENT '分享数',
    `collect_count` INT      NOT NULL DEFAULT 0           COMMENT '收藏数',
    PRIMARY KEY (`content_id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci COMMENT='内容统计计数表';
```

> **热点更新策略**：高并发场景下，`view_count` 先累积到 Redis（`INCRBY`），再由定时任务（Ratchjob）批量回写 DB。`like_count` 等由业务事件驱动更新。

---

## 3.8 `event_outbox` — 事务发件箱

**职责**：与业务写入在同一事务中记录事件，由独立进程投递到 Kafka，保证 DB↔MQ 原子性。

```sql
CREATE TABLE IF NOT EXISTS `event_outbox` (
    `id`            BIGINT       NOT NULL AUTO_INCREMENT  COMMENT '主键',
    `aggregate_type` VARCHAR(32) NOT NULL                 COMMENT '聚合类型，如 content',
    `aggregate_id`  BIGINT       NOT NULL                 COMMENT '聚合根 ID，如 content_main.id',
    `event_type`    VARCHAR(64)  NOT NULL                 COMMENT '事件类型：ContentCreated / ContentUpdated / ContentDeleted',
    `payload`       JSON         NOT NULL                 COMMENT '事件载荷（序列化后的完整数据）',
    `status`        TINYINT      NOT NULL DEFAULT 0       COMMENT '0=待发送 1=已发送 2=发送失败',
    `retry_count`   INT          NOT NULL DEFAULT 0       COMMENT '重试次数',
    `created_at`    BIGINT       NOT NULL DEFAULT 0       COMMENT '创建时间',
    `sent_at`       BIGINT       DEFAULT NULL             COMMENT '发送成功时间',
    PRIMARY KEY (`id`),
    KEY `idx_status_created` (`status`, `created_at`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='事务发件箱（Outbox Pattern）';
```

---

# 四、多形态映射实战

所有内容形态共享 `content_main` + `content_detail`，仅 `content_type`、`body` Block DSL 和 `ext_data` 不同。

### 📝 博客 (type = `blog`)

```text
content_main:  type=blog, status=2(已发布)
content_detail:
  title: "Rust Tokio 异步原理深入解析"
  summary: "本文从 Runtime 底层出发..."
  body: [
    {"type":"heading","level":1,"value":"章一：异步模型"},
    {"type":"text","value":"Tokio 采用了多线程 work-stealing 调度器..."},
    {"type":"code","lang":"rust","value":"#[tokio::main]\nasync fn main() {}"},
    {"type":"image","key":"oss/blog/arch.png","caption":"架构图"}
  ]
  ext_data: {"tags":["rust","async"],"reading_time":15,"is_original":true}
```

### 💬 朋友圈 (type = `moment`)

```text
content_main:  type=moment, status=2
content_detail:
  title: null
  summary: null
  body: [
    {"type":"text","value":"今天阳光真好，在西湖喝咖啡 ☕️"},
    {"type":"gallery","keys":["oss/moment/img1.jpg","oss/moment/img2.jpg"]}
  ]
  ext_data: {"location":{"name":"西湖景区","lat":30.2,"lng":120.1}}
```

### 📕 小红书笔记 (type = `note`)

```text
content_main:  type=note, status=2
content_detail:
  title: "魔都周末探店攻略 📸"
  cover_image: "oss/note/cover.jpg"
  body: [
    {"type":"video","key":"oss/note/vlog.mp4","cover":"oss/note/cover.jpg"},
    {"type":"text","value":"第一家店氛围感直接拉满..."},
    {"type":"location_card","name":"某咖啡厅","lat":31.2,"lng":121.4},
    {"type":"product_card","product_id":888}
  ]
  ext_data: {"bgm":{"id":"1001","name":"Chill Vibes"},"topic":"周末去哪儿"}
```

### 🛍 商品 (type = `product`)

```text
content_main:  type=product, status=2
content_detail:
  title: "iPhone 15 Pro 钛金属"
  cover_image: "oss/product/main.jpg"
  body: [
    {"type":"gallery","keys":["oss/p/img1.jpg","oss/p/img2.jpg","oss/p/img3.jpg"]},
    {"type":"text","value":"全新钛金属设计，A17 Pro 芯片..."},
    {"type":"heading","level":2,"value":"规格参数"},
    {"type":"text","value":"屏幕：6.1 英寸 OLED..."}
  ]
  ext_data: {"price":799900,"currency":"CNY","stock":100,"brand":"Apple","specs":{"color":"钛金属","storage":"256GB"}}
```

---

# 五、搜索架构

## 5.1 Search Port 抽象（防腐层）

业务层不直接依赖 Meilisearch SDK，而是通过 trait 抽象：

```rust
/// 搜索端口 — 业务层唯一的搜索契约
#[async_trait]
pub trait SearchPort: Send + Sync {
    /// 索引/更新文档
    async fn index(&self, doc: SearchDocument) -> anyhow::Result<()>;
    /// 删除文档
    async fn delete(&self, id: &str) -> anyhow::Result<()>;
    /// 搜索
    async fn search(&self, criteria: SearchCriteria) -> anyhow::Result<SearchResult>;
}

/// 搜索文档（写入模型 — 由 SyncWorker 从 DB 组装）
pub struct SearchDocument {
    pub id: String,
    pub content_type: String,
    pub author_id: i64,
    pub title: Option<String>,
    pub summary: Option<String>,
    pub body_text: Option<String>,       // 纯文本，用于全文检索
    pub cover_image: Option<String>,
    pub status: i8,
    pub tags: Vec<String>,
    pub stats: SearchDocStats,
    pub ext_attributes: serde_json::Value, // 扁平化后的 ext_data
    pub published_at: i64,
    pub created_at: i64,
}

/// 搜索条件（查询模型 — 业务层构建）
pub struct SearchCriteria {
    pub keyword: Option<String>,
    pub content_type: Option<String>,
    pub author_id: Option<i64>,
    pub status: Option<i8>,
    pub sort_by: String,      // "published_at:desc" / "like_count:desc"
    pub page: u32,
    pub size: u32,
}
```

## 5.2 Meilisearch Adapter（当前实现）

```rust
pub struct MeilisearchAdapter {
    client: meilisearch_sdk::Client,
    index_name: String, // "contents"
}

#[async_trait]
impl SearchPort for MeilisearchAdapter {
    async fn index(&self, doc: SearchDocument) -> anyhow::Result<()> { /* ... */ }
    async fn delete(&self, id: &str) -> anyhow::Result<()> { /* ... */ }
    async fn search(&self, criteria: SearchCriteria) -> anyhow::Result<SearchResult> { /* ... */ }
}
```

## 5.3 未来引擎迁移

若需要从 Meilisearch 切换到 Elasticsearch，只需：
1. 编写 `ElasticsearchAdapter` 实现 `SearchPort`
2. 通过配置切换注入的实现
3. **Service 层零改动**

---

# 六、数据同步机制

## 6.1 Outbox 投递流程

```text
┌─ DB Transaction ────────────────────────┐
│  INSERT content_main                     │
│  INSERT content_detail                   │
│  INSERT event_outbox (status=0)          │
└──────────────────────────────────────────┘
         │
         ▼ (Ratchjob 定时任务每 3 秒轮询 status=0)
         │
┌────────┴──────────────────────────────┐
│  SELECT * FROM event_outbox            │
│    WHERE status=0 ORDER BY id LIMIT 100│
│                                        │
│  → 发送到 Kafka topic: content-events  │
│  → UPDATE status=1, sent_at=NOW()      │
└───────────────────────────────────────┘
```

## 6.2 SyncWorker 消费流程

```text
Kafka(content-events) → SyncWorker:
  1. 解析 event_type
  2. 若 Created/Updated → 从 DB 读完整数据 → 组装 SearchDocument → searchPort.index()
  3. 若 Deleted → searchPort.delete(id)
  4. 异常 → 进入重试队列（指数退避）
  5. 重试 5 次仍失败 → 进入 DLQ，告警人工介入
```

## 6.3 全量重建

提供 API `POST /admin/search/rebuild`（需鉴权），遍历 `content_main` 全表，逐批索引到 Meilisearch。用于灾难恢复或引擎迁移。可作为 Ratchjob 定时任务按需触发。

---

# 七、API 接口设计

## 7.1 写操作

| 方法 | 路径 | 说明 |
|:---|:---|:---|
| `POST` | `/contents` | 创建内容 |
| `PUT` | `/contents/{id}` | 更新内容（乐观锁） |
| `DELETE` | `/contents/{id}` | 逻辑删除 |
| `PUT` | `/contents/{id}/status` | 状态变更（发布/下架/审核） |

## 7.2 读操作

| 方法 | 路径 | 数据源 | 说明 |
|:---|:---|:---|:---|
| `GET` | `/contents/{id}` | **DB** | 详情页（绝对实时 + 权限校验） |
| `GET` | `/contents/search` | **Meilisearch** | 列表/搜索 |
| `GET` | `/users/{uid}/contents` | **Meilisearch** | 作者内容列表 |

## 7.3 关系操作

| 方法 | 路径 | 说明 |
|:---|:---|:---|
| `POST` | `/contents/{id}/relations` | 建立关系（评论、挂载等） |
| `GET` | `/contents/{id}/relations?type=comment` | 查询关系链 |
| `DELETE` | `/contents/{id}/relations/{rel_id}` | 删除关系 |

## 7.4 管理接口

| 方法 | 路径 | 说明 |
|:---|:---|:---|
| `GET` | `/admin/schemas` | 获取所有已注册的 Schema |
| `POST` | `/admin/search/rebuild` | 触发搜索索引全量重建 |
| `GET` | `/contents/{id}/versions` | 获取版本历史 |
| `POST` | `/contents/{id}/versions/{ver}/rollback` | 回滚到指定版本 |

---

# 八、Rust 代码层设计

## 8.1 领域模型

```rust
/// 内容类型枚举 — 与 content_schema.content_type 对应
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContentType {
    Blog,
    Note,
    Moment,
    Post,
    Product,
}

/// ext_data 强类型封装 — 杜绝裸操作 JSON Map
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentExtension {
    Blog(BlogExt),
    Note(NoteExt),
    Moment(MomentExt),
    Product(ProductExt),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlogExt {
    pub tags: Vec<String>,
    pub reading_time: Option<i32>,
    pub is_original: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductExt {
    pub price: i64,          // 分为单位
    pub currency: String,
    pub stock: Option<i32>,
    pub brand: Option<String>,
    pub specs: Option<serde_json::Value>,
}
```

## 8.2 分层架构

```text
ms-content/src/
├── main.rs                 # fbc-starter Server::run 启动入口
├── config.rs               # 环境配置
├── error.rs                # 业务错误定义
├── job/                    # Ratchjob 定时任务
│   ├── mod.rs
│   ├── outbox_relay.rs     # Outbox 轮询投递任务
│   └── stats_flush.rs      # Redis 计数器回写 DB 任务
├── modules/
│   └── content/
│       ├── mod.rs
│       ├── handler.rs      # HTTP Handler（入参校验 → 调 Service）
│       ├── service.rs      # 业务逻辑（Schema 校验、事务、事件发射）
│       ├── repository.rs   # DB 操作（content_main / content_detail）
│       ├── model/
│       │   ├── entity.rs   # sqlxplus 实体
│       │   ├── dto.rs      # 请求/响应 DTO
│       │   └── domain.rs   # ContentExtension / Block DSL 领域模型
│       └── search/
│           ├── port.rs     # SearchPort trait 定义
│           ├── adapter.rs  # MeilisearchAdapter 实现
│           └── model.rs    # SearchDocument / SearchCriteria
├── kafka/
│   └── sync_worker.rs      # Kafka 消费者 → 索引同步
└── router.rs               # 路由注册
```

---

# 九、基础设施依赖

| 组件 | 技术 | 状态 | 用途 |
|:---|:---|:---|:---|
| DB | MySQL 8.0+ | ✅ 已有 | 事实存储 |
| Search | Meilisearch v1.42.1 | ✅ 已部署 (7700) | 列表检索 |
| MQ | Kafka | ✅ 已有 | 事件驱动 |
| 定时任务 | Ratchjob | ✅ 已部署 (8825) | Outbox 轮询 / 计数器回写 |
| 对象存储 | ms-oss (RustFS) | ✅ 已有 | 媒体文件 |
| 缓存 | Redis | ✅ 已有 | 热点计数器 / 详情缓存 |

---

# 十、风险与对策

| 风险 | 影响 | 对策 |
|:---|:---|:---|
| **写后查不到** | 用户刚发布，列表页搜不到（同步延迟 ~500ms） | 写成功后跳转详情页（查 DB）；前端乐观更新列表 |
| **Outbox 堆积** | 轮询任务异常，事件堆积延迟扩大 | Ratchjob 监控告警；DLQ 兜底 |
| **Schema 变更** | 新增字段后旧数据不符合新 Schema | Schema 变更只允许「向后兼容」（新增可选字段），不可删除/重命名必填字段 |
| **Block DSL 版本** | Block 类型升级后旧数据渲染异常 | Block 解析器向前兼容，未知 type 降级为纯文本展示 |
| **搜索深分页** | 超过 1000 条后性能劣化 | 禁止 offset > 1000，强制使用游标分页 |
| **热点计数器** | 爆款内容 `view_count` 写入密集 | Redis INCRBY 缓冲 → Ratchjob 定时批量回写 |

---

# 十一、实施路线

```text
Sprint 1（基础骨架）：
  ├── 建表：content_main / content_detail / content_stats / content_schema / event_outbox
  ├── 代码：CRUD Handler → Service → Repository
  ├── 代码：Schema 校验逻辑
  └── 代码：Block DSL 序列化/反序列化

Sprint 2（搜索集成）：
  ├── 代码：SearchPort trait + MeilisearchAdapter
  ├── 代码：SyncWorker (Kafka 消费者)
  ├── 代码：Outbox 轮询投递 (Ratchjob 任务)
  └── 联调：写入 → Outbox → Kafka → Meilisearch 全链路

Sprint 3（关系与版本）：
  ├── 建表：content_relation / content_version
  ├── 代码：关系 CRUD
  ├── 代码：版本快照 + 回滚
  └── 代码：stats_flush 定时任务

Sprint 4（打磨上线）：
  ├── 全量索引重建接口
  ├── 搜索降级兜底（Meilisearch 宕机 → DB LIKE 查询）
  ├── 压测 + 监控告警
  └── API 文档（OpenAPI / Swagger）
```