use crate::nacos::get_service_instances;
use std::str::FromStr;
use tonic::transport::Endpoint;

/// 服务端点信息
#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub address: String,
    pub endpoint: Endpoint,
    pub instance_id: String,
}

/// 从 Nacos 获取服务实例并转换为 Endpoint 列表
pub fn get_service_endpoints(service_name: &str) -> Vec<ServiceEndpoint> {
    let instances = match get_service_instances(service_name) {
        Some(instances) => instances,
        None => {
            tracing::warn!("服务 {} 没有可用的实例", service_name);
            return vec![];
        }
    };

    let mut endpoints = Vec::new();
    for instance in instances {
        let address = format!("http://{}:{}", instance.ip, instance.port);

        match Endpoint::from_str(&address) {
            Ok(endpoint) => {
                let instance_id = format!("{}:{}", instance.ip, instance.port);
                endpoints.push(ServiceEndpoint {
                    address,
                    endpoint,
                    instance_id,
                });
            }
            Err(e) => {
                tracing::warn!("无法创建服务端点 {}:{}: {}", instance.ip, instance.port, e);
            }
        }
    }

    tracing::debug!("服务 {} 发现 {} 个可用端点", service_name, endpoints.len());
    endpoints
}
