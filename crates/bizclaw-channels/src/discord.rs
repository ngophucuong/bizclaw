//! Discord Bot channel — uses Discord REST API + Gateway.

use async_trait::async_trait;
use bizclaw_core::error::{BizClawError, Result};
use bizclaw_core::traits::Channel;
use bizclaw_core::types::{IncomingMessage, OutgoingMessage, ThreadType};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};

/// Discord channel configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    pub bot_token: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool { true }

/// Discord Bot channel.
pub struct DiscordChannel {
    config: DiscordConfig,
    client: reqwest::Client,
    connected: bool,
}

impl DiscordChannel {
    pub fn new(config: DiscordConfig) -> Self {
        let client = reqwest::Client::builder()
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert("Authorization", format!("Bot {}", config.bot_token).parse().unwrap());
                h.insert("User-Agent", "BizClaw/0.1".parse().unwrap());
                h
            })
            .build()
            .unwrap_or_default();

        Self { config, client, connected: false }
    }

    /// Send a message to a channel.
    pub async fn send_message(&self, channel_id: &str, content: &str) -> Result<()> {
        let url = format!("https://discord.com/api/v10/channels/{channel_id}/messages");
        let body = serde_json::json!({ "content": content });

        let response = self.client.post(&url).json(&body).send().await
            .map_err(|e| BizClawError::Channel(format!("Discord send failed: {e}")))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(BizClawError::Channel(format!("Discord error: {text}")));
        }
        Ok(())
    }

    /// Get current bot info.
    pub async fn get_me(&self) -> Result<DiscordUser> {
        let response = self.client
            .get("https://discord.com/api/v10/users/@me")
            .send().await
            .map_err(|e| BizClawError::Channel(format!("Discord getMe failed: {e}")))?;

        response.json().await
            .map_err(|e| BizClawError::Channel(format!("Invalid Discord response: {e}")))
    }
}

#[async_trait]
impl Channel for DiscordChannel {
    fn name(&self) -> &str { "discord" }

    async fn connect(&mut self) -> Result<()> {
        let me = self.get_me().await?;
        tracing::info!("Discord bot: {} ({})", me.username, me.id);
        self.connected = true;
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool { self.connected }

    async fn send(&self, message: OutgoingMessage) -> Result<()> {
        self.send_message(&message.thread_id, &message.content).await
    }

    async fn listen(&self) -> Result<Box<dyn Stream<Item = IncomingMessage> + Send + Unpin>> {
        Ok(Box::new(stream::pending()))
    }
}

// --- API Types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordUser {
    pub id: String,
    pub username: String,
    pub discriminator: String,
    pub bot: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordMessage {
    pub id: String,
    pub channel_id: String,
    pub author: DiscordUser,
    pub content: String,
    pub guild_id: Option<String>,
}

impl DiscordMessage {
    /// Convert to BizClaw IncomingMessage.
    pub fn to_incoming(&self) -> IncomingMessage {
        IncomingMessage {
            channel: "discord".into(),
            thread_id: self.channel_id.clone(),
            sender_id: self.author.id.clone(),
            sender_name: Some(self.author.username.clone()),
            content: self.content.clone(),
            thread_type: if self.guild_id.is_some() {
                ThreadType::Group
            } else {
                ThreadType::Direct
            },
            timestamp: chrono::Utc::now(),
            reply_to: None,
        }
    }
}
