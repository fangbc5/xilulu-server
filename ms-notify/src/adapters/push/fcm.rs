use yup_oauth2::{ServiceAccountAuthenticator, ServiceAccountKey};
use reqwest::Client;
use serde_json::json;
use anyhow::{Context, Result};
use tracing::{info, warn};
use std::fs;
use crate::config::FcmConfig;

#[derive(Clone)]
pub struct FcmAdapter {
    project_id: String,
    authenticator: yup_oauth2::authenticator::Authenticator<yup_oauth2::hyper_rustls::HttpsConnector<yup_oauth2::hyper::client::HttpConnector>>,
    client: Client,
}

impl FcmAdapter {
    pub async fn new(config: &FcmConfig) -> Result<Self> {
        let key_data = fs::read_to_string(&config.service_account_json_path)
            .with_context(|| format!("Failed to read FCM service account from {}", config.service_account_json_path))?;
            
        let sa_key: ServiceAccountKey = serde_json::from_str(&key_data)
            .context("Failed to parse FCM service account JSON")?;

        let authenticator = ServiceAccountAuthenticator::builder(sa_key)
            .build()
            .await
            .context("Failed to build FCM Service Account Authenticator")?;

        let client = Client::new();

        Ok(Self {
            project_id: config.project_id.clone(),
            authenticator,
            client,
        })
    }

    pub async fn send_push(&self, device_token: &str, title: &str, body: &str) -> Result<()> {
        let scopes = &["https://www.googleapis.com/auth/firebase.messaging"];
        let token = self.authenticator.token(scopes)
            .await
            .context("Failed to get FCM OAuth2 token")?;

        let url = format!("https://fcm.googleapis.com/v1/projects/{}/messages:send", self.project_id);

        let payload = json!({
            "message": {
                "token": device_token,
                "notification": {
                    "title": title,
                    "body": body
                }
            }
        });

        let res = self.client.post(&url)
            .bearer_auth(token.token().unwrap_or_default())
            .json(&payload)
            .send()
            .await
            .context("Failed to send FCM request")?;

        if res.status().is_success() {
            info!("FCM push sent successfully to {}", device_token);
            Ok(())
        } else {
            let status = res.status();
            let error_text = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            warn!("FCM push failed for {}: [{}] {}", device_token, status, error_text);
            Err(anyhow::anyhow!("FCM push error: {}", error_text))
        }
    }
}
