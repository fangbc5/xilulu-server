mod client;
mod service;

use dashmap::DashMap;
use nacos_rust_client::client::naming_client::Instance;
use once_cell::sync::Lazy;
use std::sync::Arc;

/// 服务实例存储：服务名称 -> 实例列表
/// 用于记录订阅的服务和其实例列表，每次服务变化时更新该 map
static SERVICE_INSTANCES: Lazy<DashMap<String, Vec<Arc<Instance>>>> = Lazy::new(|| DashMap::new());

/// 配置存储：配置键（data_id:group:namespace） -> 配置内容
/// 用于记录订阅的配置和其内容，每次配置变化时更新该 map
static NACOS_CONFIGS: Lazy<DashMap<String, String>> = Lazy::new(|| DashMap::new());

/// 获取指定服务的实例列表
pub fn get_service_instances(service_name: &str) -> Option<Vec<Arc<Instance>>> {
    SERVICE_INSTANCES
        .get(service_name)
        .map(|entry| entry.value().clone())
}

/// 更新服务的实例列表（内部使用）
///
/// 防御策略：拒绝用空列表覆盖非空实例列表，
/// 因为 nacos_rust_client 在订阅阶段会产生虚假的空回调。
/// 如果服务真的全部下线，保留过期实例让 gRPC 传输层返回更有意义的连接错误。
pub(crate) fn update_service_instances(service_name: &str, instances: Vec<Arc<Instance>>) {
    if instances.is_empty() {
        if let Some(existing) = SERVICE_INSTANCES.get(service_name) {
            if !existing.value().is_empty() {
                tracing::warn!(
                    "⚠️ 拒绝用空列表覆盖服务 {} 的 {} 个现有实例（防御虚假回调）",
                    service_name,
                    existing.value().len()
                );
                return;
            }
        }
    }
    SERVICE_INSTANCES.insert(service_name.to_string(), instances);
}

/// 获取配置内容
///
/// # 参数
/// - `data_id`: 配置的 Data ID
/// - `group`: 配置的 Group（可选，默认为 "DEFAULT_GROUP"）
/// - `namespace`: 配置的命名空间（可选，默认为 "public"）
///
/// # 返回
/// 如果找到配置，返回 `Some(配置内容)`，否则返回 `None`
pub fn get_config(data_id: &str, group: Option<&str>, namespace: Option<&str>) -> Option<String> {
    let group = group.unwrap_or("DEFAULT_GROUP");
    let namespace = namespace.unwrap_or("public");
    let key = format!("{}:{}:{}", data_id, group, namespace);
    NACOS_CONFIGS.get(&key).map(|entry| entry.value().clone())
}

/// 获取所有已订阅的配置键列表
pub fn get_subscribed_configs() -> Vec<String> {
    NACOS_CONFIGS
        .iter()
        .map(|entry| entry.key().clone())
        .collect()
}

/// 获取所有已订阅的服务名称列表
pub fn get_subscribed_services() -> Vec<String> {
    SERVICE_INSTANCES
        .iter()
        .map(|entry| entry.key().clone())
        .collect()
}

/// 更新配置内容（内部使用）
pub(crate) fn update_config(data_id: &str, group: &str, namespace: &str, value: String) {
    let key = format!("{}:{}:{}", data_id, group, namespace);
    NACOS_CONFIGS.insert(key, value);
}

pub use client::{get_config_client, get_naming_client, init_nacos};
pub use service::{deregister_service, register_service, subscribe_configs, subscribe_services};
