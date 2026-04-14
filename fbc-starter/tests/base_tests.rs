//! 统一响应 R<T> 测试
//!
//! 验证空字段条件跳过序列化 + 基本功能

use fbc_starter::base::R;
use serde_json;

/// 测试空字段不出现在 JSON 中
#[test]
fn test_empty_fields_skipped_in_json() {
    let r: R<String> = R::ok_with_data("hello".to_string());
    let json = serde_json::to_string(&r).unwrap();
    
    // 空字段不应该出现
    assert!(!json.contains("\"path\""), "空的 path 不应该被序列化: {}", json);
    assert!(!json.contains("\"version\""), "空的 version 不应该被序列化: {}", json);
    assert!(!json.contains("\"base_version\""), "空的 base_version 不应该被序列化: {}", json);
}

/// 测试必填字段都存在
#[test]
fn test_required_fields_present() {
    let r: R<String> = R::ok_with_data("data".to_string());
    let json = serde_json::to_string(&r).unwrap();
    
    assert!(json.contains("\"success\""), "应该包含 success 字段");
    assert!(json.contains("\"code\""), "应该包含 code 字段");
    assert!(json.contains("\"timestamp\""), "应该包含 timestamp 字段");
    assert!(json.contains("\"data\""), "应该包含 data 字段");
}

/// 测试 R::ok 默认值
#[test]
fn test_r_ok_defaults() {
    let r: R<i32> = R::ok();
    assert!(r.success);
    assert_eq!(r.code, 200);
    assert!(r.data.is_none());
    assert!(r.path.is_empty());
    assert!(r.version.is_empty());
    assert!(r.base_version.is_empty());
}

/// 测试 R::ok_with_data
#[test]
fn test_r_ok_with_data() {
    let r: R<i32> = R::ok_with_data(42);
    assert!(r.success);
    assert_eq!(r.code, 200);
    assert_eq!(r.data, Some(42));
}

/// 测试 R::fail 默认值
#[test]
fn test_r_fail() {
    let r: R<()> = R::fail();
    assert!(!r.success);
}

/// 测试 R::fail_with_message
#[test]
fn test_r_fail_with_message() {
    let r: R<()> = R::fail_with_message("出错了".to_string());
    assert!(!r.success);
    assert_eq!(r.msg, Some("出错了".to_string()));
}

/// 测试反序列化中空字段缺失不影响
#[test]
fn test_deserialize_without_optional_fields() {
    let json = r#"{"success":true,"code":0,"msg":null,"data":"test","timestamp":123456}"#;
    let r: R<String> = serde_json::from_str(json).unwrap();
    assert!(r.success);
    assert_eq!(r.data, Some("test".to_string()));
    assert!(r.path.is_empty());
}

/// 测试非空 path 正常序列化
#[test]
fn test_non_empty_path_serialized() {
    let mut r: R<()> = R::ok();
    r.path = "/api/test".to_string();
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains("path"), "非空的 path 应该被序列化: {}", json);
    assert!(json.contains("/api/test"));
}
