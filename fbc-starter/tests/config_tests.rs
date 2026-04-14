//! 配置加载测试

use fbc_starter::config::Config;

/// 测试默认配置创建
#[test]
fn test_default_config() {
    let config = Config::default();
    assert_eq!(config.server.port, 3000);
    assert_eq!(config.log.level, "info");
    assert!(!config.log.json);
    assert_eq!(config.log.timezone, 8);
    assert!(!config.cors.allow_credentials);
}

/// 测试默认 CORS 配置
#[test]
fn test_default_cors_config() {
    let config = Config::default();
    assert!(config.cors.allowed_origins.contains(&"*".to_string()));
    assert!(config.cors.allowed_methods.contains(&"GET".to_string()));
    assert!(config.cors.allowed_methods.contains(&"POST".to_string()));
    assert!(config.cors.allowed_headers.contains(&"*".to_string()));
}

/// 测试配置默认值一致性
#[test]
fn test_config_default_values_consistency() {
    let config = Config::default();
    // 确保日志时区默认东八区
    assert_eq!(config.log.timezone, 8);
    // 确保默认端口是 3000
    assert_eq!(config.server.port, 3000);
}
