// 临时 Token 服务
// 用于多租户场景下的租户选择

use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::state::AppState;
use fbc_starter::cache::TokenService;

/// 临时 Token 数据
#[derive(Debug, Serialize, Deserialize)]
pub struct TempTokenData {
    /// 用户ID
    pub user_id: i64,
    /// 租户列表
    pub tenant_list: Vec<TenantInfo>,
}

/// 租户信息（用于临时 token）
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TenantInfo {
    pub tenant_id: i64,
    pub is_owner: bool,
}

/// 临时 Token 服务
pub struct TempTokenService;

impl TempTokenService {
    const TEMP_TOKEN_PREFIX: &'static str = "temp_token:";
    const TEMP_TOKEN_TTL: u64 = 300; // 5分钟过期

    /// 生成临时 token 并存储
    pub async fn create_temp_token(
        app_state: &AppState,
        user_id: i64,
        tenant_list: Vec<TenantInfo>,
    ) -> Result<String> {
        let token = Uuid::new_v4().to_string();
        let key = format!("{}{}", Self::TEMP_TOKEN_PREFIX, token);

        let data = TempTokenData {
            user_id,
            tenant_list,
        };

        let json_data = serde_json::to_string(&data)?;

        // 存储到 Redis
        let mut redis_conn = app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| anyhow::anyhow!("Redis连接失败: {}", e))?;

        TokenService::set_token(&mut redis_conn, &key, &json_data, Self::TEMP_TOKEN_TTL)
            .await
            .map_err(|e| anyhow::anyhow!("存储临时token失败: {}", e))?;

        Ok(token)
    }

    /// 验证并获取临时 token 数据
    pub async fn verify_and_get(app_state: &AppState, token: &str) -> Result<TempTokenData> {
        let key = format!("{}{}", Self::TEMP_TOKEN_PREFIX, token);

        let mut redis_conn = app_state
            .fbc_app_state
            .redis()
            .await
            .map_err(|e| anyhow::anyhow!("Redis连接失败: {}", e))?;

        let json_data: Option<String> = TokenService::get_token(&mut redis_conn, &key)
            .await
            .map_err(|e| anyhow::anyhow!("获取临时token失败: {}", e))?;

        let json_data = json_data.ok_or_else(|| anyhow::anyhow!("临时token不存在或已过期"))?;

        let data: TempTokenData = serde_json::from_str(&json_data)?;

        // 删除临时 token（一次性使用）
        TokenService::delete_token(&mut redis_conn, &key)
            .await
            .map_err(|e| anyhow::anyhow!("删除临时token失败: {}", e))?;

        Ok(data)
    }
}
