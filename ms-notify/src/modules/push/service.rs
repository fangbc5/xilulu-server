use crate::adapters::push::apns::ApnsAdapter;
use crate::adapters::push::fcm::FcmAdapter;
use crate::config::NotifyServiceConfig;
use anyhow::Result;
use tracing::info;

#[derive(Clone)]
pub struct PushService {
    apns: Option<ApnsAdapter>,
    fcm: Option<FcmAdapter>,
}

impl PushService {
    pub async fn new(config: &NotifyServiceConfig) -> Result<Self> {
        let apns = if let Some(apns_cfg) = &config.apns {
            Some(ApnsAdapter::new(apns_cfg)?)
        } else {
            None
        };

        let fcm = if let Some(fcm_cfg) = &config.fcm {
            Some(FcmAdapter::new(fcm_cfg).await?)
        } else {
            None
        };

        Ok(Self { apns, fcm })
    }

    pub async fn dispatch_push(&self, platform: &str, device_token: &str, title: &str, body: &str) -> Result<()> {
        match platform {
            "ios" => {
                if let Some(apns) = &self.apns {
                    let _ = apns.send_push(device_token, title, body).await?;
                } else {
                    info!("APNs is not configured, skipping push for ios: {}", device_token);
                }
            }
            "android" => {
                if let Some(fcm) = &self.fcm {
                    let _ = fcm.send_push(device_token, title, body).await?;
                } else {
                    info!("FCM is not configured, skipping push for android: {}", device_token);
                }
            }
            _ => {
                info!("Unknown platform {} for token {}", platform, device_token);
            }
        }
        Ok(())
    }
}
