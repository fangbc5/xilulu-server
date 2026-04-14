use crate::modules::device::DeviceService;
// 导入自动生成的 proto
pub mod device_pb {
    tonic::include_proto!("device");
}

use device_pb::device_service_server::{DeviceService as GrpcDeviceService, DeviceServiceServer};
use device_pb::{DeviceTokenInfo, GetUserDevicesRequest, GetUserDevicesResponse};

use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::error;

/// 设备 gRPC 服务实现
pub struct DeviceServiceImpl {
    device_service: Arc<DeviceService>,
}

impl DeviceServiceImpl {
    pub fn new(device_service: Arc<DeviceService>) -> Self {
        Self { device_service }
    }

    pub fn server(device_service: Arc<DeviceService>) -> DeviceServiceServer<Self> {
        DeviceServiceServer::new(Self::new(device_service))
    }
}

#[tonic::async_trait]
impl GrpcDeviceService for DeviceServiceImpl {
    async fn get_user_devices(
        &self,
        request: Request<GetUserDevicesRequest>,
    ) -> Result<Response<GetUserDevicesResponse>, Status> {
        let req = request.into_inner();
        let uids = req.user_ids;

        if uids.is_empty() {
            return Ok(Response::new(GetUserDevicesResponse {
                success: true,
                message: "success".to_string(),
                devices: vec![],
            }));
        }

        // 查找这些用户的有效设备
        match self.device_service.get_active_devices_by_uids(&uids).await {
            Ok(db_devices) => {
                let devices = db_devices
                    .into_iter()
                    .map(|d| DeviceTokenInfo {
                        user_id: d.uid.unwrap_or(0),
                        platform: d.platform.unwrap_or_default(),
                        push_token: d.device_token.unwrap_or_default(),
                    })
                    .collect();

                Ok(Response::new(GetUserDevicesResponse {
                    success: true,
                    message: "success".to_string(),
                    devices,
                }))
            }
            Err(e) => {
                error!("查找用户设备失败: {:?}", e);
                Err(Status::internal("查找用户设备失败"))
            }
        }
    }
}
