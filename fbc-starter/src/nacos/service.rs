use nacos_rust_client::client::config_client::{listener::ConfigListener, ConfigKey};
use nacos_rust_client::client::naming_client::Instance;

use crate::config::{NacosConfig, ServerConfig};

/// 注册服务到 Nacos
pub async fn register_service(
    config: &NacosConfig,
    server_config: &ServerConfig,
) -> Result<(), anyhow::Error> {
    let naming_client = super::client::get_naming_client()
        .map_err(|e| anyhow::anyhow!("获取 Nacos 命名客户端失败: {}", e))?;

    let service_ip = config
        .service_ip
        .as_ref()
        .unwrap_or(&server_config.addr)
        .clone();

    // 如果 IP 是 0.0.0.0（监听所有接口），则自动获取本机真实 IP
    let service_ip = if service_ip == "0.0.0.0" {
        get_local_ip().unwrap_or_else(|| {
            tracing::warn!("无法自动获取本机 IP，Nacos 注册将使用 127.0.0.1");
            "127.0.0.1".to_string()
        })
    } else {
        service_ip
    };
    let service_port = config.service_port.unwrap_or(server_config.port as u32);

    // 构建元数据
    let mut metadata = std::collections::HashMap::new();
    if let Some(ref meta) = config.metadata {
        metadata.extend(meta.clone());
    }
    // 添加健康检查路径到元数据
    if let Some(ref health_path) = config.health_check_path {
        metadata.insert("health_check_path".to_string(), health_path.clone());
    }

    // 获取服务名称：优先使用配置中的 service_name，如果没有则尝试从环境变量获取
    let service_name = if !config.service_name.is_empty() {
        config.service_name.clone()
    } else {
        // 尝试从环境变量获取服务名称（使用 starter 的服务会在编译时设置 CARGO_PKG_NAME）
        std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "unknown-service".to_string())
    };

    // 创建服务实例
    // 根据 nacos_rust_client 0.3 的 API，Instance::new_simple 创建基础实例
    let mut instance =
        Instance::new_simple(&service_ip, service_port, &service_name, config.effective_naming_group());

    // 设置元数据（metadata 是 Option 类型）
    if !metadata.is_empty() {
        instance.metadata = Some(metadata.clone());
    }

    // 注册服务实例（register 方法只接受 instance 参数，返回 ()）
    // 注意：命名空间在创建 NamingClient 时已经设置，不需要在 Instance 中再次设置
    naming_client.register(instance);

    tracing::info!(
        "服务注册成功: {}:{} (服务名: {}, 组: {}, 命名空间: {:?}, 元数据: {:?})",
        service_ip,
        service_port,
        service_name,
        config.effective_naming_group(),
        config.effective_naming_namespace(),
        metadata
    );

    Ok(())
}

/// 从 Nacos 注销服务
pub async fn deregister_service(
    config: &NacosConfig,
    server_config: &ServerConfig,
) -> Result<(), anyhow::Error> {
    let naming_client = super::client::get_naming_client()
        .map_err(|e| anyhow::anyhow!("获取 Nacos 命名客户端失败: {}", e))?;

    // 获取服务名称：优先使用配置中的 service_name，如果没有则尝试从环境变量获取
    let service_name = if !config.service_name.is_empty() {
        config.service_name.clone()
    } else {
        // 尝试从环境变量获取服务名称（使用 starter 的服务会在编译时设置 CARGO_PKG_NAME）
        std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "unknown-service".to_string())
    };

    let service_ip = config
        .service_ip
        .as_ref()
        .unwrap_or(&server_config.addr)
        .clone();
    let service_port = config.service_port.unwrap_or(server_config.port as u32);

    // 注销服务实例
    // 根据 nacos_rust_client 0.3 的 API，注销服务需要创建相同的实例，然后调用 unregister 方法
    let instance =
        Instance::new_simple(&service_ip, service_port, &service_name, config.effective_naming_group());

    // 使用 unregister 方法注销服务实例
    naming_client.unregister(instance);

    tracing::info!(
        "服务注销成功: {}:{} (服务名: {}, 组: {}, 命名空间: {:?})",
        service_ip,
        service_port,
        service_name,
        config.effective_naming_group(),
        config.effective_naming_namespace()
    );

    Ok(())
}

/// 订阅服务列表（用于服务发现）
pub async fn subscribe_services(config: &NacosConfig) -> Result<(), anyhow::Error> {
    if config.subscribe_services.is_empty() {
        return Ok(());
    }

    let naming_client = super::client::get_naming_client()
        .map_err(|e| anyhow::anyhow!("获取 Nacos 命名客户端失败: {}", e))?;

    // 订阅服务变更
    for service_name in &config.subscribe_services {
        let service_name_clone = service_name.clone();

        // 创建服务变更监听器
        // 根据 nacos_rust_client 0.3 的 API，subscribe 方法接受一个实现了 InstanceListener 的对象
        use nacos_rust_client::client::naming_client::{InstanceListener, ServiceInstanceKey};

        let group_name = config.effective_naming_group().to_string();

        struct ServiceChangeListener {
            service_name: String,
            group_name: String,
        }

        impl InstanceListener for ServiceChangeListener {
            fn get_key(&self) -> ServiceInstanceKey {
                ServiceInstanceKey::new(&self.service_name, &self.group_name)
            }

            fn change(
                &self,
                _key: &ServiceInstanceKey,
                value: &Vec<std::sync::Arc<nacos_rust_client::client::naming_client::Instance>>,
                add_list: &Vec<std::sync::Arc<nacos_rust_client::client::naming_client::Instance>>,
                remove_list: &Vec<
                    std::sync::Arc<nacos_rust_client::client::naming_client::Instance>,
                >,
            ) {
                tracing::info!(
                    "服务 {} 实例列表发生变化 (全量:{} 新增:{} 移除:{})",
                    self.service_name,
                    value.len(),
                    add_list.len(),
                    remove_list.len()
                );

                // 更新实例列表（存储层会拒绝用空列表覆盖非空实例）
                super::update_service_instances(&self.service_name, value.clone());

                // 打印所有实例信息
                for instance in value {
                    tracing::info!(
                        "  - 服务实例: {}:{} (健康状态: {})",
                        instance.ip,
                        instance.port,
                        instance.healthy
                    );
                }
            }
        }

        // ===== 先 query_instances 填充实例列表，再 subscribe 注册变更监听 =====
        // 避免 subscribe 和 query_instances 之间的竞争导致虚假移除回调
        use nacos_rust_client::client::naming_client::QueryInstanceListParams;

        let params = QueryInstanceListParams::new_simple(&service_name_clone, config.effective_naming_group());

        match naming_client.query_instances(params).await {
            Ok(instances) => {
                let instance_count = instances.len();
                tracing::info!(
                    "获取服务 {} 的当前实例列表成功，实例数量: {}",
                    service_name_clone,
                    instance_count
                );

                // 填充 SERVICE_INSTANCES
                super::update_service_instances(&service_name_clone, instances.clone());

                // 打印所有实例信息
                for instance in &instances {
                    tracing::info!(
                        "  - 服务实例: {}:{} (健康状态: {})",
                        instance.ip,
                        instance.port,
                        instance.healthy
                    );
                }
            }
            Err(e) => {
                tracing::warn!("获取服务 {} 的当前实例列表失败: {}", service_name_clone, e);
            }
        }

        // 注册变更监听（后续增量更新）
        let listener = Box::new(ServiceChangeListener {
            service_name: service_name_clone.clone(),
            group_name,
        });

        naming_client
            .subscribe(listener)
            .await
            .map_err(|e| anyhow::anyhow!("订阅服务 {} 失败: {}", service_name, e))?;

        tracing::info!(
            "订阅服务成功: {} (命名空间: {:?}, 组: {})",
            service_name,
            config.effective_naming_namespace(),
            config.effective_naming_group()
        );
    }

    Ok(())
}

/// 订阅配置列表（用于配置管理）
pub async fn subscribe_configs(config: &NacosConfig) -> Result<(), anyhow::Error> {
    if config.subscribe_configs.is_empty() {
        return Ok(());
    }

    let config_client = super::client::get_config_client()
        .map_err(|e| anyhow::anyhow!("获取 Nacos 配置客户端失败: {}", e))?;

    // 订阅配置变更
    for config_item in &config.subscribe_configs {
        let data_id = config_item.data_id.clone();
        let group = config_item.group.clone();
        let namespace = config_item.namespace.clone();

        // 创建配置变更监听器
        struct ConfigChangeListener {
            data_id: String,
            group: String,
            namespace: String,
        }

        impl ConfigListener for ConfigChangeListener {
            fn get_key(&self) -> ConfigKey {
                ConfigKey::new(&self.data_id, &self.group, &self.namespace)
            }

            fn change(&self, key: &ConfigKey, value: &str) {
                let namespace = if key.tenant.is_empty() {
                    "public"
                } else {
                    &key.tenant
                };

                tracing::info!(
                    "配置变更: {} (组: {}, 命名空间: {}, 值长度: {} 字节)",
                    key.data_id,
                    key.group,
                    namespace,
                    value.len()
                );
                tracing::debug!("配置内容: {}", value);

                // 更新 DashMap 中的配置内容
                super::update_config(&key.data_id, &key.group, namespace, value.to_string());
            }
        }

        let listener = Box::new(ConfigChangeListener {
            data_id: data_id.clone(),
            group: group.clone(),
            namespace: namespace.clone(),
        });

        // 订阅配置变更
        config_client
            .subscribe(listener)
            .await
            .map_err(|e| anyhow::anyhow!("订阅配置 {} 失败: {}", data_id, e))?;

        tracing::info!(
            "订阅配置成功: {} (组: {}, 命名空间: {})",
            data_id,
            group,
            if namespace.is_empty() {
                "public"
            } else {
                &namespace
            }
        );
    }

    Ok(())
}

/// 获取本机真实 IP 地址（通过 UDP socket 探测）
/// 与 ms-gateway 的 local_ip() 实现一致
fn get_local_ip() -> Option<String> {
    use std::net::UdpSocket;
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket.connect("8.8.8.8:80").ok()?;
    socket.local_addr().ok().map(|a| a.ip().to_string())
}
