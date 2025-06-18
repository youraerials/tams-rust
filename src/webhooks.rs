use crate::{error::TamsResult, models::*};
use reqwest::Client;
use serde_json::json;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct WebhookInfo {
    pub webhook: Webhook,
    pub api_key_value: String,
}

pub struct WebhookManager {
    client: Client,
    webhooks: Arc<RwLock<HashMap<String, WebhookInfo>>>,
}

impl WebhookManager {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            webhooks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_webhook(&self, webhook: Webhook, api_key_value: String) {
        let mut webhooks = self.webhooks.write().await;
        webhooks.insert(
            webhook.url.clone(),
            WebhookInfo {
                webhook,
                api_key_value,
            },
        );
        info!("Added webhook: {}", webhooks.len());
    }

    pub async fn remove_webhook(&self, url: &str) {
        let mut webhooks = self.webhooks.write().await;
        if webhooks.remove(url).is_some() {
            info!("Removed webhook: {}", url);
        }
    }

    pub async fn send_notification<T>(&self, notification: EventNotification<T>)
    where
        T: serde::Serialize + Send + Sync,
    {
        let webhooks = self.webhooks.read().await;
        
        for webhook_info in webhooks.values() {
            if webhook_info.webhook.events.contains(&notification.event_type)
                || webhook_info.webhook.events.contains(&"*".to_string())
            {
                let webhook_info = webhook_info.clone();
                let notification_json = match serde_json::to_value(&notification) {
                    Ok(json) => json,
                    Err(e) => {
                        error!("Failed to serialize notification: {}", e);
                        continue;
                    }
                };
                
                let client = self.client.clone();
                tokio::spawn(async move {
                    if let Err(e) = Self::send_webhook_request(
                        &client,
                        &webhook_info,
                        notification_json,
                    ).await {
                        error!("Failed to send webhook notification to {}: {}", 
                               webhook_info.webhook.url, e);
                    }
                });
            }
        }
    }

    async fn send_webhook_request(
        client: &Client,
        webhook_info: &WebhookInfo,
        payload: serde_json::Value,
    ) -> TamsResult<()> {
        let mut request_builder = client
            .post(&webhook_info.webhook.url)
            .json(&payload)
            .header("Content-Type", "application/json")
            .header("User-Agent", "TAMS-Rust/6.0");

        // Add API key header if specified
        if let Some(api_key_name) = &webhook_info.webhook.api_key_name {
            request_builder = request_builder.header(api_key_name, &webhook_info.api_key_value);
        }

        let response = request_builder.send().await?;

        if response.status().is_success() {
            info!("Successfully sent webhook notification to {}", webhook_info.webhook.url);
        } else {
            warn!(
                "Webhook returned non-success status {}: {}",
                response.status(),
                webhook_info.webhook.url
            );
        }

        Ok(())
    }

    pub async fn get_webhook_count(&self) -> usize {
        let webhooks = self.webhooks.read().await;
        webhooks.len()
    }

    pub async fn load_webhooks_from_database(&self, webhooks: Vec<(Webhook, String)>) {
        let mut webhook_map = self.webhooks.write().await;
        webhook_map.clear();
        
        for (webhook, api_key_value) in webhooks {
            webhook_map.insert(
                webhook.url.clone(),
                WebhookInfo {
                    webhook,
                    api_key_value,
                },
            );
        }
        
        info!("Loaded {} webhooks from database", webhook_map.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_webhook_manager_creation() {
        let manager = WebhookManager::new();
        assert_eq!(manager.get_webhook_count().await, 0);
    }

    #[tokio::test]
    async fn test_add_remove_webhook() {
        let manager = WebhookManager::new();
        
        let webhook = Webhook {
            url: "https://example.com/webhook".to_string(),
            api_key_name: Some("X-API-Key".to_string()),
            api_key_value: None,
            events: vec!["flow.created".to_string()],
        };
        
        manager.add_webhook(webhook.clone(), "secret-key".to_string()).await;
        assert_eq!(manager.get_webhook_count().await, 1);
        
        manager.remove_webhook(&webhook.url).await;
        assert_eq!(manager.get_webhook_count().await, 0);
    }

    #[tokio::test]
    async fn test_load_webhooks_from_database() {
        let manager = WebhookManager::new();
        
        let webhook1 = Webhook {
            url: "https://example.com/webhook1".to_string(),
            api_key_name: None,
            api_key_value: None,
            events: vec!["*".to_string()],
        };
        
        let webhook2 = Webhook {
            url: "https://example.com/webhook2".to_string(),
            api_key_name: Some("Authorization".to_string()),
            api_key_value: None,
            events: vec!["flow.created".to_string(), "flow.updated".to_string()],
        };
        
        let webhooks = vec![
            (webhook1, "".to_string()),
            (webhook2, "Bearer token".to_string()),
        ];
        
        manager.load_webhooks_from_database(webhooks).await;
        assert_eq!(manager.get_webhook_count().await, 2);
    }
} 