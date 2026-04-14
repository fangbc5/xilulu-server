//! gRPC 服务实现

use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::modules::file::model::dto::{
    PresignDownloadRequest, PresignUploadRequest,
};
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
        request: Request<pb::PresignUploadReq>,
    ) -> Result<Response<pb::PresignUploadResp>, Status> {
        let req = request.into_inner();

        let dto = PresignUploadRequest {
            bucket: if req.bucket.is_empty() { None } else { Some(req.bucket) },
            filename: req.filename,
            content_type: if req.content_type.is_empty() { None } else { Some(req.content_type) },
            scene: req.scene,
            size: if req.size > 0 { Some(req.size) } else { None },
        };

        let uploader_id = if req.uploader_id > 0 { Some(req.uploader_id) } else { None };

        match self.state.file_service.presign_upload(dto, uploader_id).await {
            Ok(resp) => Ok(Response::new(pb::PresignUploadResp {
                success: true,
                message: "ok".to_string(),
                upload_url: resp.upload_url,
                object_key: resp.object_key,
                file_id: resp.file_id,
                expires_in: resp.expires_in,
            })),
            Err(e) => Ok(Response::new(pb::PresignUploadResp {
                success: false,
                message: e.to_string(),
                ..Default::default()
            })),
        }
    }

    async fn presign_download(
        &self,
        request: Request<pb::PresignDownloadReq>,
    ) -> Result<Response<pb::PresignDownloadResp>, Status> {
        let req = request.into_inner();

        let dto = PresignDownloadRequest {
            bucket: if req.bucket.is_empty() { None } else { Some(req.bucket) },
            object_key: req.object_key,
        };

        match self.state.file_service.presign_download(dto).await {
            Ok(resp) => Ok(Response::new(pb::PresignDownloadResp {
                success: true,
                message: "ok".to_string(),
                download_url: resp.download_url,
                expires_in: resp.expires_in,
            })),
            Err(e) => Ok(Response::new(pb::PresignDownloadResp {
                success: false,
                message: e.to_string(),
                ..Default::default()
            })),
        }
    }

    async fn get_file_meta(
        &self,
        request: Request<pb::GetFileMetaReq>,
    ) -> Result<Response<pb::FileMetaResp>, Status> {
        let req = request.into_inner();

        match self.state.file_service.get_file_meta(req.file_id).await {
            Ok(Some(meta)) => Ok(Response::new(pb::FileMetaResp {
                success: true,
                message: "ok".to_string(),
                id: meta.id,
                file_key: meta.file_key,
                bucket: meta.bucket,
                original_name: meta.original_name.unwrap_or_default(),
                content_type: meta.content_type.unwrap_or_default(),
                size: meta.size.unwrap_or(0),
                scene: meta.scene,
                status: meta.status as i32,
                created_at: meta.created_at,
            })),
            Ok(None) => Ok(Response::new(pb::FileMetaResp {
                success: false,
                message: "文件不存在".to_string(),
                ..Default::default()
            })),
            Err(e) => Ok(Response::new(pb::FileMetaResp {
                success: false,
                message: e.to_string(),
                ..Default::default()
            })),
        }
    }
}
