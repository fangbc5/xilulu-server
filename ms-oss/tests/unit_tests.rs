//! ms-oss 单元测试

use ms_oss::config::{OssConfig, SceneRule};
use ms_oss::modules::file::model::dto::*;
use ms_oss::modules::file::model::entity::FileMeta;
use ms_oss::provider::{ObjectMeta, OssProvider, PresignedUrl};
use async_trait::async_trait;
use std::sync::atomic::{AtomicBool, Ordering};

// ============================================
// Mock Provider
// ============================================

struct MockOssProvider {
    fail_presign: AtomicBool,
    fail_delete: AtomicBool,
}

impl MockOssProvider {
    fn new() -> Self {
        Self {
            fail_presign: AtomicBool::new(false),
            fail_delete: AtomicBool::new(false),
        }
    }

    fn with_fail_presign(self) -> Self {
        self.fail_presign.store(true, Ordering::Relaxed);
        self
    }

    fn with_fail_delete(self) -> Self {
        self.fail_delete.store(true, Ordering::Relaxed);
        self
    }
}

#[async_trait]
impl OssProvider for MockOssProvider {
    async fn presign_put(&self, bucket: &str, key: &str, _content_type: Option<&str>, expires_secs: u64) -> anyhow::Result<PresignedUrl> {
        if self.fail_presign.load(Ordering::Relaxed) {
            return Err(anyhow::anyhow!("presign_put mock error"));
        }
        Ok(PresignedUrl { url: format!("https://{}.s3.example.com/{}?signed=true", bucket, key), expires_in: expires_secs })
    }

    async fn presign_get(&self, bucket: &str, key: &str, expires_secs: u64) -> anyhow::Result<PresignedUrl> {
        Ok(PresignedUrl { url: format!("https://{}.s3.example.com/{}?download=true", bucket, key), expires_in: expires_secs })
    }

    async fn delete_object(&self, _bucket: &str, _key: &str) -> anyhow::Result<()> {
        if self.fail_delete.load(Ordering::Relaxed) { return Err(anyhow::anyhow!("delete_object mock error")); }
        Ok(())
    }

    async fn head_object(&self, bucket: &str, key: &str) -> anyhow::Result<ObjectMeta> {
        Ok(ObjectMeta { bucket: bucket.to_string(), key: key.to_string(), size: Some(1024), content_type: Some("image/png".to_string()) })
    }
}

// ============================================
// OssConfig 测试
// ============================================

#[test]
fn test_oss_config_default() {
    let config = OssConfig::default();
    assert_eq!(config.provider, "rustfs");
    assert_eq!(config.endpoint, "http://127.0.0.1:9000");
    assert_eq!(config.presign_expires_secs, 3600);
    assert_eq!(config.default_bucket, "public");
    assert!(!config.watermark_enabled);
    assert!(!config.thumbnail_enabled);
}

#[test]
fn test_oss_config_from_env() {
    std::env::set_var("OSS__PROVIDER", "aliyun");
    std::env::set_var("OSS__ENDPOINT", "https://oss-cn-hangzhou.aliyuncs.com");
    std::env::set_var("OSS__REGION", "cn-hangzhou");
    std::env::set_var("OSS__ACCESS_KEY", "test_ak");
    std::env::set_var("OSS__SECRET_KEY", "test_sk");
    std::env::set_var("OSS__DEFAULT_BUCKET", "test-bucket");

    let config = OssConfig::from_env();
    assert_eq!(config.provider, "aliyun");
    assert_eq!(config.endpoint, "https://oss-cn-hangzhou.aliyuncs.com");
    assert_eq!(config.access_key, "test_ak");
    assert_eq!(config.default_bucket, "test-bucket");
    assert!(config.presign_expires_secs > 0);

    std::env::remove_var("OSS__PROVIDER");
    std::env::remove_var("OSS__ENDPOINT");
    std::env::remove_var("OSS__REGION");
    std::env::remove_var("OSS__ACCESS_KEY");
    std::env::remove_var("OSS__SECRET_KEY");
    std::env::remove_var("OSS__DEFAULT_BUCKET");
}

#[test]
fn test_oss_config_from_env_with_invalid_expires() {
    std::env::set_var("OSS__PRESIGN_EXPIRES_SECS", "not_a_number");
    let config = OssConfig::from_env();
    assert_eq!(config.presign_expires_secs, 3600);
    std::env::remove_var("OSS__PRESIGN_EXPIRES_SECS");
}

// ============================================
// SceneRule 测试
// ============================================

#[test]
fn test_scene_rule_avatar() {
    let config = OssConfig::default();
    let rule = config.get_scene_rule("avatar");
    assert!(rule.allowed_extensions.contains(&"jpg".to_string()));
    assert!(rule.allowed_extensions.contains(&"png".to_string()));
    assert_eq!(rule.max_size_bytes, 5 * 1024 * 1024);
}

#[test]
fn test_scene_rule_chat_image() {
    let config = OssConfig::default();
    let rule = config.get_scene_rule("chat_image");
    assert!(rule.allowed_extensions.contains(&"gif".to_string()));
    assert_eq!(rule.max_size_bytes, 10 * 1024 * 1024);
}

#[test]
fn test_scene_rule_logo() {
    let config = OssConfig::default();
    let rule = config.get_scene_rule("logo");
    assert!(rule.allowed_extensions.contains(&"svg".to_string()));
    assert_eq!(rule.max_size_bytes, 2 * 1024 * 1024);
}

#[test]
fn test_scene_rule_unknown_fallback() {
    let config = OssConfig::default();
    let rule = config.get_scene_rule("unknown_scene");
    assert!(rule.allowed_extensions.contains(&"*".to_string()));
    assert_eq!(rule.max_size_bytes, 50 * 1024 * 1024);
}

// ============================================
// Provider 测试
// ============================================

#[tokio::test]
async fn test_provider_presign_put_success() {
    let provider = MockOssProvider::new();
    let result = provider.presign_put("hula", "avatar/test.jpg", Some("image/jpeg"), 3600).await;
    assert!(result.is_ok());
    let presigned = result.unwrap();
    assert!(presigned.url.contains("hula"));
    assert_eq!(presigned.expires_in, 3600);
}

#[tokio::test]
async fn test_provider_presign_put_without_content_type() {
    let provider = MockOssProvider::new();
    let result = provider.presign_put("hula", "data/file.bin", None, 1800).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().expires_in, 1800);
}

#[tokio::test]
async fn test_provider_presign_put_failure() {
    let provider = MockOssProvider::new().with_fail_presign();
    let result = provider.presign_put("hula", "avatar/test.jpg", None, 3600).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_provider_presign_get() {
    let provider = MockOssProvider::new();
    let result = provider.presign_get("hula", "avatar/test.jpg", 3600).await;
    assert!(result.is_ok());
    assert!(result.unwrap().url.contains("download=true"));
}

#[tokio::test]
async fn test_provider_delete_object_success() {
    let provider = MockOssProvider::new();
    assert!(provider.delete_object("hula", "test.jpg").await.is_ok());
}

#[tokio::test]
async fn test_provider_delete_object_failure() {
    let provider = MockOssProvider::new().with_fail_delete();
    assert!(provider.delete_object("hula", "test.jpg").await.is_err());
}

#[tokio::test]
async fn test_provider_head_object() {
    let provider = MockOssProvider::new();
    let meta = provider.head_object("hula", "avatar/test.jpg").await.unwrap();
    assert_eq!(meta.size, Some(1024));
    assert_eq!(meta.content_type, Some("image/png".to_string()));
}

// ============================================
// DTO 测试
// ============================================

#[test]
fn test_presign_upload_request_full() {
    let json = r#"{"bucket":"custom","filename":"photo.jpg","content_type":"image/jpeg","scene":"avatar"}"#;
    let req: PresignUploadRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.bucket, Some("custom".to_string()));
    assert_eq!(req.scene, "avatar");
}

#[test]
fn test_presign_upload_request_minimal() {
    let json = r#"{"filename":"photo.jpg","scene":"avatar"}"#;
    let req: PresignUploadRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.filename, "photo.jpg");
    assert!(req.bucket.is_none());
    assert!(req.content_type.is_none());
}

#[test]
fn test_upload_callback_with_size() {
    let json = r#"{"file_id":42,"size":1024}"#;
    let req: UploadCallbackRequest = serde_json::from_str(json).unwrap();
    assert_eq!(req.file_id, 42);
    assert_eq!(req.size, Some(1024));
}

#[test]
fn test_upload_callback_without_size() {
    let json = r#"{"file_id":1}"#;
    let req: UploadCallbackRequest = serde_json::from_str(json).unwrap();
    assert!(req.size.is_none());
}

#[test]
fn test_presign_upload_response_serialization() {
    let resp = PresignUploadResponse {
        upload_url: "https://example.com/signed".to_string(),
        object_key: "avatar/2026/03/test.jpg".to_string(),
        file_id: 1,
        expires_in: 3600,
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["file_id"], 1);
    assert_eq!(json["expires_in"], 3600);
}

#[test]
fn test_file_meta_response_serialization() {
    let resp = FileMetaResponse {
        id: 1,
        file_key: "avatar/test.jpg".to_string(),
        bucket: "public".to_string(),
        original_name: Some("photo.jpg".to_string()),
        content_type: Some("image/jpeg".to_string()),
        size: Some(2048),
        scene: "avatar".to_string(),
        status: 1,
        created_at: "2026-03-12 16:00:00".to_string(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert_eq!(json["scene"], "avatar");
    assert_eq!(json["status"], 1);
}

// ============================================
// Entity 测试
// ============================================

#[test]
fn test_file_meta_entity_creation() {
    let meta = FileMeta {
        id: None, file_key: Some("avatar/2026/03/abc.jpg".to_string()),
        bucket: Some("public".to_string()), original_name: Some("photo.jpg".to_string()),
        content_type: Some("image/jpeg".to_string()), size: None,
        scene: Some("avatar".to_string()), uploader_id: Some(100),
        provider: Some("rustfs".to_string()), status: Some(0),
        watermark: Some(0), thumbnail_key: None,
        created_at: None, updated_at: None,
    };
    assert!(meta.id.is_none());
    assert_eq!(meta.scene.as_deref(), Some("avatar"));
}

#[test]
fn test_file_meta_serialization_roundtrip() {
    let meta = FileMeta {
        id: Some(1), file_key: Some("test/key".to_string()),
        bucket: Some("public".to_string()), original_name: Some("test.txt".to_string()),
        content_type: Some("text/plain".to_string()), size: Some(512),
        scene: Some("document".to_string()), uploader_id: Some(1),
        provider: Some("rustfs".to_string()), status: Some(1),
        watermark: Some(0), thumbnail_key: None,
        created_at: None, updated_at: None,
    };
    let json = serde_json::to_string(&meta).unwrap();
    let deserialized: FileMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, Some(1));
    assert_eq!(deserialized.size, Some(512));
}

// ============================================
// Error 测试
// ============================================

#[test]
fn test_oss_error_display() {
    use ms_oss::error::OssError;
    assert_eq!(OssError::FileNotFound("not found".into()).to_string(), "not found");
    assert_eq!(OssError::FileTypeNotAllowed(".exe".into()).to_string(), ".exe");
    assert_eq!(OssError::FileTooLarge("too big".into()).to_string(), "too big");
}

#[test]
fn test_oss_error_from_anyhow() {
    use ms_oss::error::OssError;
    let err: OssError = anyhow::anyhow!("internal").into();
    assert!(err.to_string().contains("internal"));
}

#[test]
fn test_oss_error_code_values() {
    use ms_oss::error::code;
    assert_eq!(code::FILE_NOT_FOUND, 4501);
    assert_eq!(code::FILE_SIZE_MISMATCH, 4502);
    assert_eq!(code::PRESIGN_FAILED, 4503);
    assert_eq!(code::CALLBACK_FAILED, 4504);
    assert_eq!(code::BAD_REQUEST, 4507);
    assert_eq!(code::FILE_TYPE_NOT_ALLOWED, 4508);
    assert_eq!(code::FILE_TOO_LARGE, 4509);
    assert_eq!(code::INTERNAL_ERROR, 5001);
}
