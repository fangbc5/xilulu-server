//! AppState 扩展能力测试
//!
//! 验证类型安全的 extensions 存取、线程安全性

use fbc_starter::state::AppState;
use std::sync::Arc;

/// 用户自定义扩展类型
#[derive(Debug, Clone, PartialEq)]
struct MyConfig {
    api_key: String,
    max_retries: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct SessionStore {
    name: String,
}

/// 测试基本的 set/get extension
#[test]
fn test_set_get_extension() {
    let state = AppState::new();
    
    let config = MyConfig {
        api_key: "secret-123".to_string(),
        max_retries: 3,
    };
    
    state.set_extension(config.clone());
    
    let retrieved = state.get_extension::<MyConfig>();
    assert!(retrieved.is_some());
    assert_eq!(*retrieved.unwrap(), config);
}

/// 测试获取不存在的扩展返回 None
#[test]
fn test_get_nonexistent_extension() {
    let state = AppState::new();
    let result = state.get_extension::<MyConfig>();
    assert!(result.is_none());
}

/// 测试 has_extension
#[test]
fn test_has_extension() {
    let state = AppState::new();
    assert!(!state.has_extension::<MyConfig>());
    
    state.set_extension(MyConfig {
        api_key: "key".to_string(),
        max_retries: 1,
    });
    
    assert!(state.has_extension::<MyConfig>());
    assert!(!state.has_extension::<SessionStore>());
}

/// 测试多个不同类型的扩展共存
#[test]
fn test_multiple_extensions() {
    let state = AppState::new();
    
    state.set_extension(MyConfig {
        api_key: "key-1".to_string(),
        max_retries: 5,
    });
    state.set_extension(SessionStore {
        name: "redis-session".to_string(),
    });
    
    let config = state.get_extension::<MyConfig>().unwrap();
    let session = state.get_extension::<SessionStore>().unwrap();
    
    assert_eq!(config.api_key, "key-1");
    assert_eq!(session.name, "redis-session");
}

/// 测试覆盖写入
#[test]
fn test_override_extension() {
    let state = AppState::new();
    
    state.set_extension(MyConfig {
        api_key: "old-key".to_string(),
        max_retries: 1,
    });
    
    state.set_extension(MyConfig {
        api_key: "new-key".to_string(),
        max_retries: 10,
    });
    
    let config = state.get_extension::<MyConfig>().unwrap();
    assert_eq!(config.api_key, "new-key");
    assert_eq!(config.max_retries, 10);
}

/// 测试 with_extension 链式构建
#[test]
fn test_with_extension_builder() {
    let state = AppState::new()
        .with_extension(MyConfig {
            api_key: "builder-key".to_string(),
            max_retries: 3,
        })
        .with_extension(SessionStore {
            name: "mem-session".to_string(),
        });
    
    assert!(state.has_extension::<MyConfig>());
    assert!(state.has_extension::<SessionStore>());
}

/// 测试克隆后的 state 共享 extensions
#[test]
fn test_clone_shares_extensions() {
    let state = AppState::new();
    state.set_extension(MyConfig {
        api_key: "shared".to_string(),
        max_retries: 1,
    });
    
    let cloned = state.clone();
    
    // 克隆后的 state 应该能看到原始 state 的扩展
    let config = cloned.get_extension::<MyConfig>().unwrap();
    assert_eq!(config.api_key, "shared");
    
    // 在克隆上设置新扩展，原始也能看到（因为共享 Arc<DashMap>）
    cloned.set_extension(SessionStore {
        name: "from-clone".to_string(),
    });
    assert!(state.has_extension::<SessionStore>());
}

/// 测试线程安全
#[test]
fn test_extension_thread_safety() {
    let state = Arc::new(AppState::new());
    
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let s = state.clone();
            std::thread::spawn(move || {
                // 每个线程注入不同类型的数据（通过不同的 i 值）
                s.set_extension(MyConfig {
                    api_key: format!("thread-{}", i),
                    max_retries: i as u32,
                });
                // 读取
                let _ = s.get_extension::<MyConfig>();
            })
        })
        .collect();
    
    for h in handles {
        h.join().unwrap();
    }
    
    // 最终应该有一个 MyConfig（最后写入的线程的值）
    assert!(state.has_extension::<MyConfig>());
}
