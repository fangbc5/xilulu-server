use a2::{Client, Endpoint};
use a2::request::notification::NotificationBuilder;
use anyhow::{Context, Result};
use tracing::{info, warn};
use std::fs::File;
use crate::config::ApnsConfig;

#[derive(Clone)]
pub struct ApnsAdapter {
    client: Client,
    topic: String,
}

impl ApnsAdapter {
    pub fn new(config: &ApnsConfig) -> Result<Self> {
        let mut file = File::open(&config.p8_cert_path)
            .with_context(|| format!("Failed to open p8 cert file at {}", config.p8_cert_path))?;
        
        let client = Client::token(&mut file, &config.key_id, &config.team_id, Endpoint::Production)
            .map_err(|e| anyhow::anyhow!("Failed to initialize APNs client: {}", e))?;

        Ok(Self { 
            client, 
            topic: config.topic.clone() 
        })
    }

    pub async fn send_push(&self, device_token: &str, title: &str, body: &str) -> Result<()> {
        let mut request = a2::request::notification::DefaultNotificationBuilder::new()
            .set_title(title)
            .set_body(body)
            .build(device_token, Default::default());
        request.options.apns_topic = Some(self.topic.as_str());

        match self.client.send(request).await {
            Ok(res) => {
                info!("APNs push sent successfully to {}: {:?}", device_token, res);
                Ok(())
            }
            Err(e) => {
                warn!("APNs push failed for {}: {:?}", device_token, e);
                Err(anyhow::anyhow!("APNs push error: {:?}", e))
            }
        }
    }
}
