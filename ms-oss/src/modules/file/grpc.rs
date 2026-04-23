//! gRPC 服务实现

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::state::OssState;

pub mod pb {
    tonic::include_proto!("oss");
}

use pb::oss_service_server::OssService;

pub struct OssGrpcService {
    state: Arc<OssState>,
}

impl OssGrpcService {
    pub fn new(state: Arc<OssState>) -> Self {
        Self { state }
    }
}

#[tonic::async_trait]
impl OssService for OssGrpcService {
    async fn presign_upload(
        &self,
        _request: Request<pb::PresignUploadReq>,
    ) -> Result<Response<pb::PresignUploadResp>, Status> {
        Err(Status::unimplemented(
            "该接口已被废弃，请直接请求 OSS RESTful API",
        ))
    }

    async fn presign_download(
        &self,
        _request: Request<pb::PresignDownloadReq>,
    ) -> Result<Response<pb::PresignDownloadResp>, Status> {
        Err(Status::unimplemented(
            "该接口已被废弃，请直接请求 OSS RESTful API",
        ))
    }

    async fn get_file_meta(
        &self,
        _request: Request<pb::GetFileMetaReq>,
    ) -> Result<Response<pb::FileMetaResp>, Status> {
        Err(Status::unimplemented(
            "该接口已被废弃，请直接请求 OSS RESTful API",
        ))
    }
}
