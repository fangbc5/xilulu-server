pub mod device_pb {
    tonic::include_proto!("device");
}

use device_pb::device_service_client::DeviceServiceClient;
use device_pb::{GetUserDevicesRequest, GetUserDevicesResponse};
use tonic::transport::Channel;
use anyhow::Result;

pub struct DeviceClient;

impl DeviceClient {
    pub async fn get_user_devices(user_ids: Vec<u64>) -> Result<GetUserDevicesResponse> {
        let lb = fbc_starter::get_load_balancer("ms-identity");
        use fbc_starter::LoadBalancer;
        let endpoint = lb
            .next_endpoint()
            .ok_or_else(|| anyhow::anyhow!("未找到 ms-identity 服务"))?;

        let channel = Channel::from_shared(format!("http://{}", endpoint.endpoint.uri()))
            .map_err(|e| anyhow::anyhow!("创建 Channel 失败: {}", e))?
            .connect()
            .await?;

        let mut client = DeviceServiceClient::new(channel);

        let user_ids_i64 = user_ids.into_iter().map(|id| id as i64).collect();
        let request = tonic::Request::new(GetUserDevicesRequest { user_ids: user_ids_i64 });
        let response = client.get_user_devices(request).await?;

        Ok(response.into_inner())
    }
}
