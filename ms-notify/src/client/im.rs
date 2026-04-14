pub mod im_pb {
    tonic::include_proto!("im");
}

use im_pb::im_service_client::ImServiceClient;
use im_pb::{BatchGetContactMuteStatusRequest, BatchGetContactMuteStatusResponse};
use tonic::transport::Channel;
use anyhow::Result;

pub struct ImClient;

impl ImClient {
    pub async fn batch_get_contact_mute_status(
        room_id: i64,
        uids: Vec<u64>,
    ) -> Result<BatchGetContactMuteStatusResponse> {
        let lb = fbc_starter::get_load_balancer("ms-im");
        use fbc_starter::LoadBalancer;
        let endpoint = lb
            .next_endpoint()
            .ok_or_else(|| anyhow::anyhow!("未找到 ms-im 服务"))?;

        let channel = Channel::from_shared(format!("http://{}", endpoint.endpoint.uri()))
            .map_err(|e| anyhow::anyhow!("创建 Channel 失败: {}", e))?
            .connect()
            .await?;

        let mut client = ImServiceClient::new(channel);

        let uids_i64: Vec<i64> = uids.into_iter().map(|id| id as i64).collect();
        let request = tonic::Request::new(BatchGetContactMuteStatusRequest {
            room_id,
            uids: uids_i64,
        });

        let response = client.batch_get_contact_mute_status(request).await?;

        Ok(response.into_inner())
    }
}
