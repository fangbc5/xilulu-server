use sa_token_core::{config::{SaTokenConfig, TokenStyle}, refresh::RefreshTokenManager, SaTokenManager};
use sa_token_plugin_axum::{RedisStorage, SaStorage};
use serde_json::json;
use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

const TEN_YEARS_SECONDS: i64 = 10 * 365 * 24 * 60 * 60;

fn load_env_file() -> HashMap<String, String> {
    let env_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    let content = fs::read_to_string(&env_path)
        .unwrap_or_else(|e| panic!("failed to read {}: {}", env_path.display(), e));

    content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                return None;
            }

            let (key, value) = line.split_once('=')?;
            Some((
                key.trim().to_string(),
                value.trim().trim_matches('"').trim_matches('\'').to_string(),
            ))
        })
        .collect()
}

fn build_redis_url(envs: &HashMap<String, String>) -> String {
    let base_url = envs
        .get("APP__REDIS__URL")
        .cloned()
        .unwrap_or_else(|| "redis://127.0.0.1:6379".to_string());

    let password = envs
        .get("APP__REDIS__PASSWORD")
        .filter(|value| !value.is_empty())
        .cloned();

    if let Some(password) = password {
        if base_url.contains('@') {
            base_url
        } else if let Some(prefix_end) = base_url.find("://") {
            let prefix = &base_url[..prefix_end + 3];
            let rest = &base_url[prefix_end + 3..];
            format!("{}:{}@{}", prefix, password, rest)
        } else {
            base_url
        }
    } else {
        base_url
    }
}

#[tokio::test]
async fn generate_permanent_debug_token_for_user_1_tenant_1() -> Result<(), Box<dyn std::error::Error>> {
    let envs = load_env_file();
    let jwt_secret = envs
        .get("APP__AUTH__JWT_SECRET")
        .cloned()
        .expect("APP__AUTH__JWT_SECRET is missing in .env");
    let redis_url = build_redis_url(&envs);

    let storage: Arc<dyn SaStorage> = Arc::new(RedisStorage::new(&redis_url, "sa_token:").await?);
    let config = SaTokenConfig {
        token_name: "Authorization".to_string(),
        timeout: TEN_YEARS_SECONDS,
        token_style: TokenStyle::Jwt,
        jwt_secret_key: Some(jwt_secret),
        enable_refresh_token: true,
        refresh_token_timeout: TEN_YEARS_SECONDS,
        ..Default::default()
    };

    let manager = SaTokenManager::new(storage.clone(), config.clone());
    let extra = json!({
        "tenant_id": "1",
        "username": "debug-user-1",
        "token_type": "access"
    });

    let access_token = manager
        .login_with_options("1", None, None, Some(extra.clone()), None, None)
        .await?;

    let refresh_token_mgr = RefreshTokenManager::new(storage, Arc::new(config));
    let refresh_token = refresh_token_mgr.generate("1");
    refresh_token_mgr
        .store_with_extra(&refresh_token, access_token.as_str(), "1", Some(&extra))
        .await?;

    let token_info = manager.get_token_info(&access_token).await?;
    assert_eq!(token_info.login_id, "1");
    assert!(token_info.expire_time.is_some(), "token should have a long expiration");
    assert_eq!(refresh_token_mgr.validate(&refresh_token).await?, "1");

    println!("ACCESS_TOKEN={}", access_token.as_str());
    println!("REFRESH_TOKEN={}", refresh_token);

    Ok(())
}
