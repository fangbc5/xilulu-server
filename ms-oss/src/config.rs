use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 场景规则：定义特定 scene 的校验规则和 Bucket 路由
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneRule {
    /// 专用 Bucket（None 则使用 default_bucket）
    pub bucket: Option<String>,
    /// 允许的文件扩展名（空或含 "*" 表示不限制）
    pub allowed_extensions: Vec<String>,
    /// 文件大小上限（字节）
    pub max_size_bytes: i64,
}

impl Default for SceneRule {
    fn default() -> Self {
        Self {
            bucket: None,
            allowed_extensions: vec!["*".to_string()],
            max_size_bytes: 50 * 1024 * 1024, // 50MB
        }
    }
}

/// OSS 配置（从环境变量加载）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OssConfig {
    /// 当前激活的 Provider（rustfs / aliyun / tencent / aws）
    pub provider: String,
    /// S3 兼容端点（例如：http://127.0.0.1:9000）
    pub endpoint: String,
    /// 区域
    pub region: String,
    /// 访问密钥
    pub access_key: String,
    /// 密钥
    pub secret_key: String,
    /// 默认 Bucket（默认值：public）
    pub default_bucket: String,
    /// 客户端可访问的公网端点（用于预签名 URL，为空时等于 endpoint）
    pub public_endpoint: String,
    /// 预签名 URL 过期时间（秒，默认 3600）
    pub presign_expires_secs: u64,
    /// 是否启用水印处理
    pub watermark_enabled: bool,
    /// 是否启用缩略图生成
    pub thumbnail_enabled: bool,
    /// 场景规则（key = scene 名称）
    #[serde(default)]
    pub scene_rules: HashMap<String, SceneRule>,
}

impl OssConfig {
    /// 从环境变量加载 OSS 配置
    pub fn from_env() -> Self {
        Self {
            provider: std::env::var("OSS__PROVIDER")
                .unwrap_or_else(|_| "rustfs".to_string()),
            endpoint: std::env::var("OSS__ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string()),
            public_endpoint: std::env::var("OSS__PUBLIC_ENDPOINT")
                .unwrap_or_else(|_| std::env::var("OSS__ENDPOINT")
                    .unwrap_or_else(|_| "http://127.0.0.1:9000".to_string())),
            region: std::env::var("OSS__REGION")
                .unwrap_or_else(|_| "us-east-1".to_string()),
            access_key: std::env::var("OSS__ACCESS_KEY")
                .unwrap_or_else(|_| "rustfsadmin".to_string()),
            secret_key: std::env::var("OSS__SECRET_KEY")
                .unwrap_or_else(|_| "rustfsadmin".to_string()),
            default_bucket: std::env::var("OSS__DEFAULT_BUCKET")
                .unwrap_or_else(|_| "public".to_string()),
            presign_expires_secs: std::env::var("OSS__PRESIGN_EXPIRES_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(3600),
            watermark_enabled: std::env::var("OSS__WATERMARK_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            thumbnail_enabled: std::env::var("OSS__THUMBNAIL_ENABLED")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false),
            scene_rules: Self::default_scene_rules(),
        }
    }

    /// 内置默认场景规则
    fn default_scene_rules() -> HashMap<String, SceneRule> {
        let mut rules = HashMap::new();

        rules.insert("avatar".to_string(), SceneRule {
            bucket: None,
            allowed_extensions: vec!["jpg".into(), "jpeg".into(), "png".into(), "webp".into()],
            max_size_bytes: 5 * 1024 * 1024,
        });

        rules.insert("chat_image".to_string(), SceneRule {
            bucket: Some("chat-media".into()),
            allowed_extensions: vec!["jpg".into(), "jpeg".into(), "png".into(), "gif".into(), "webp".into(), "heic".into()],
            max_size_bytes: 10 * 1024 * 1024,
        });

        rules.insert("chat_voice".to_string(), SceneRule {
            bucket: Some("chat-media".into()),
            allowed_extensions: vec!["aac".into(), "mp3".into(), "m4a".into(), "ogg".into(), "wav".into(), "webm".into()],
            max_size_bytes: 5 * 1024 * 1024, // ~60s
        });

        rules.insert("chat_video".to_string(), SceneRule {
            bucket: Some("chat-media".into()),
            allowed_extensions: vec!["mp4".into(), "mov".into(), "webm".into()],
            max_size_bytes: 50 * 1024 * 1024, // ~5min
        });

        rules.insert("chat_file".to_string(), SceneRule {
            bucket: Some("chat-files".into()),
            allowed_extensions: vec!["*".into()],
            max_size_bytes: 100 * 1024 * 1024,
        });

        rules.insert("logo".to_string(), SceneRule {
            bucket: None,
            allowed_extensions: vec!["jpg".into(), "jpeg".into(), "png".into(), "webp".into(), "svg".into()],
            max_size_bytes: 2 * 1024 * 1024,
        });

        rules.insert("document".to_string(), SceneRule {
            bucket: None,
            allowed_extensions: vec!["pdf".into(), "doc".into(), "docx".into(), "xls".into(), "xlsx".into(), "ppt".into(), "pptx".into()],
            max_size_bytes: 20 * 1024 * 1024,
        });

        rules
    }

    /// 按 scene 获取规则，找不到则用默认规则
    pub fn get_scene_rule(&self, scene: &str) -> SceneRule {
        self.scene_rules.get(scene).cloned().unwrap_or_default()
    }
}

impl Default for OssConfig {
    fn default() -> Self {
        Self {
            provider: "rustfs".to_string(),
            endpoint: "http://127.0.0.1:9000".to_string(),
            public_endpoint: "http://127.0.0.1:9000".to_string(),
            region: "us-east-1".to_string(),
            access_key: "rustfsadmin".to_string(),
            secret_key: "rustfsadmin".to_string(),
            default_bucket: "public".to_string(),
            presign_expires_secs: 3600,
            watermark_enabled: false,
            thumbnail_enabled: false,
            scene_rules: Self::default_scene_rules(),
        }
    }
}
