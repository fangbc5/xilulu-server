use std::sync::Arc;
use tonic::{Request, Response, Status};
use redis::AsyncCommands;

pub mod im_pb {
    tonic::include_proto!("im");
}

use im_pb::im_service_server::{ImService, ImServiceServer};
use im_pb::{BatchGetContactMuteStatusRequest, BatchGetContactMuteStatusResponse, MuteStatus};

use crate::state::ImState;
use crate::modules::contact::repository::ContactRepo;
use crate::cache::ContactMuteCacheKeyBuilder;
use fbc_starter::cache::CacheKeyBuilder;

pub struct ImServiceImpl {
    pub im_state: Arc<ImState>,
}

impl ImServiceImpl {
    pub fn new(im_state: Arc<ImState>) -> Self {
        Self { im_state }
    }

    pub fn into_server(self) -> ImServiceServer<Self> {
        ImServiceServer::new(self)
    }
}

#[tonic::async_trait]
impl ImService for ImServiceImpl {
    async fn batch_get_contact_mute_status(
        &self,
        request: Request<BatchGetContactMuteStatusRequest>,
    ) -> Result<Response<BatchGetContactMuteStatusResponse>, Status> {
        let req = request.into_inner();
        let room_id = req.room_id;
        let uids = req.uids;

        let uids_i64: Vec<i64> = uids.into_iter().collect();
        let mut status_list = Vec::new();
        let mut missed_uids = Vec::new();

        // 1. 优先查缓存 (旁路缓存)
        let mut cache_conn_res = self.im_state.fbc.redis().await;
        if let Ok(ref mut conn) = cache_conn_res {
            if !uids_i64.is_empty() {
                let cache_keys: Vec<String> = uids_i64
                    .iter()
                    .map(|&uid| ContactMuteCacheKeyBuilder.key(&[&uid, &room_id]).key)
                    .collect();

                let mut mget_cmd = redis::cmd("MGET");
                for key in &cache_keys {
                    mget_cmd.arg(key);
                }

                let mget_res: redis::RedisResult<Vec<Option<String>>> =
                    mget_cmd.query_async(conn).await;

                if let Ok(values) = mget_res {
                    for (i, &uid) in uids_i64.iter().enumerate() {
                        match values.get(i) {
                            Some(Some(val)) if val == "1" => {
                                status_list.push(MuteStatus { uid, is_mute: true });
                            }
                            Some(Some(val)) if val == "0" => {
                                status_list.push(MuteStatus { uid, is_mute: false });
                            }
                            _ => {
                                missed_uids.push(uid);
                            }
                        }
                    }
                } else {
                    missed_uids = uids_i64.clone();
                }
            }
        } else {
            // 如果 Redis 连接失败，全部走查库兜底
            missed_uids = uids_i64.clone();
        }

        // 2. 缓存未命中的查询数据库
        if !missed_uids.is_empty() {
            let contacts = ContactRepo::find_by_uids_room(
                self.im_state.db_pool.mysql_pool(),
                &missed_uids,
                room_id,
            )
            .await
            .map_err(|e| Status::internal(format!("DB 错误: {}", e)))?;

            let mut mute_map = std::collections::HashMap::new();
            for contact in contacts {
                if let Some(uid) = contact.uid {
                    mute_map.insert(uid, contact.is_mute.unwrap_or(0) == 1);
                }
            }

            // 3. 回写缓存避免穿透，并合并结果
            for uid in missed_uids {
                let is_mute = mute_map.get(&uid).copied().unwrap_or(false);
                
                if let Ok(ref mut conn) = cache_conn_res {
                    let cache_key = ContactMuteCacheKeyBuilder.key(&[&uid, &room_id]);
                    if let Some(expire) = cache_key.expire {
                        let ttl = expire.as_secs();
                        let val = if is_mute { "1" } else { "0" };
                        let _: Result<(), redis::RedisError> = conn.set_ex(&cache_key.key, val, ttl).await;
                    }
                }

                status_list.push(MuteStatus { uid, is_mute });
            }
        }

        Ok(Response::new(BatchGetContactMuteStatusResponse {
            statuses: status_list,
        }))
    }
}
