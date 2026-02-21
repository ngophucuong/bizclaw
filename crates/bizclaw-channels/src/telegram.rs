//! Telegram Bot channel — uses Telegram Bot API for messaging.

use async_trait::async_trait;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::Channel;
use bizclaw_core::types::{IncomingMessage, OutgoingMessage, ThreadType};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};

/// Telegram channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramConfig {
    pub bot_token: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_poll_interval")]
    pub poll_interval: u64,
}

fn default_true() -> bool { true }
fn default_poll_interval() -> u64 { 1 }

/// Telegram Bot channel.
pub struct TelegramChannel {
    config: TelegramConfig,
    client: reqwest::Client,
    last_update_id: i64,
    connected: bool,
}

impl TelegramChannel {
    pub fn new(config: TelegramConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            last_update_id: 0,
            connected: false,
        }
    }

    fn api_url(&self, method: &str) -> String {
        format!("https://api.telegram.org/bot{}/{}", self.config.bot_token, method)
    }

    /// Get updates from Telegram using long polling.
    pub async fn get_updates(&mut self) -> Result<Vec<TelegramUpdate>> {
        let response = self.client
            .get(&self.api_url("getUpdates"))
            .query(&[
                ("offset", (self.last_update_id + 1).to_string()),
                ("timeout", "30".into()),
            ])
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Telegram getUpdates failed: {e}")))?;

        let body: TelegramResponse<Vec<TelegramUpdate>> = response.json().await
            .map_err(|e| BizClawError::Channel(format!("Invalid Telegram response: {e}")))?;

        if !body.ok {
            return Err(BizClawError::Channel(format!(
                "Telegram API error: {}", body.description.unwrap_or_default()
            )));
        }

        let updates = body.result.unwrap_or_default();
        if let Some(last) = updates.last() {
            self.last_update_id = last.update_id;
        }
        Ok(updates)
    }

    /// Send a text message to a chat.
    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<()> {
        let body = serde_json::json!({
            "chat_id": chat_id,
            "text": text,
            "parse_mode": "Markdown",
        });

        let response = self.client
            .post(&self.api_url("sendMessage"))
            .json(&body)
            .send()
            .await
            .map_err(|e| BizClawError::Channel(format!("Telegram sendMessage failed: {e}")))?;

        let result: TelegramResponse<serde_json::Value> = response.json().await
            .map_err(|e| BizClawError::Channel(format!("Invalid send response: {e}")))?;

        if !result.ok {
            return Err(BizClawError::Channel(format!(
                "Send failed: {}", result.description.unwrap_or_default()
            )));
        }
        Ok(())
    }

    /// Get bot info.
    pub async fn get_me(&self) -> Result<TelegramUser> {
        let response = self.client.get(&self.api_url("getMe")).send().await
            .map_err(|e| BizClawError::Channel(format!("getMe failed: {e}")))?;
        let body: TelegramResponse<TelegramUser> = response.json().await
            .map_err(|e| BizClawError::Channel(format!("Invalid getMe response: {e}")))?;
        body.result.ok_or_else(|| BizClawError::Channel("No bot info".into()))
    }
}

#[async_trait]
impl Channel for TelegramChannel {
    fn name(&self) -> &str { "telegram" }

    async fn connect(&mut self) -> Result<()> {
        let me = self.get_me().await?;
        tracing::info!("Telegram bot: @{} ({})",
            me.username.as_deref().unwrap_or("unknown"), me.first_name);
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool { self.connected }

    async fn send(&self, message: OutgoingMessage) -> Result<()> {
        let chat_id: i64 = message.thread_id.parse()
            .map_err(|_| BizClawError::Channel("Invalid chat_id".into()))?;
        self.send_message(chat_id, &message.content).await
    }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = IncomingMessage> + Send + Unpin>> {
        Ok(Box::new(stream::pending()))
    }
}

// --- API Types ---

#[derive(Debug, Deserialize)]
pub struct TelegramResponse<T> {
    pub ok: bool,
    pub result: Option<T>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    pub text: Option<String>,
    pub date: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub is_bot: bool,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    #[serde(rename = "type")]
    pub chat_type: String,
    pub title: Option<String>,
}

impl TelegramUpdate {
    /// Convert to BizClaw IncomingMessage.
    pub fn to_incoming(&self) -> Option<IncomingMessage> {
        let msg = self.message.as_ref()?;
        let text = msg.text.as_ref()?;
        let from = msg.from.as_ref()?;

        Some(IncomingMessage {
            channel: "telegram".into(),
            thread_id: msg.chat.id.to_string(),
            sender_id: from.id.to_string(),
            sender_name: Some(from.first_name.clone()),
            content: text.clone(),
            thread_type: if msg.chat.chat_type == "private" {
                ThreadType::Direct
            } else {
                ThreadType::Group
            },
            timestamp: chrono::Utc::now(),
            reply_to: None,
        })
    }
}
