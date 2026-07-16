use super::core::{AgentEvent, Config};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum KafkaError {
    #[error("transport not available")]
    NotAvailable,
    #[error("serialization failed: {0}")]
    Serialize(#[from] serde_json::Error),
    #[error("not implemented in stub")]
    Stub,
}

/// Abstraction over the message bus so the agent can run without a real broker.
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn send_event(&self, topic: &str, event: &AgentEvent) -> Result<(), KafkaError>;
    async fn receive_command(&mut self) -> Result<Option<CommandEnvelope>, KafkaError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandEnvelope {
    pub id: String,
    pub ts: chrono::DateTime<chrono::Utc>,
    pub service_id: String,
    pub kind: String,
    pub payload: Value,
    #[serde(default)]
    pub signature: Option<String>,
}

pub struct TopicNames {
    pub events: String,
    pub commands: String,
    pub alerts: String,
    pub audit: String,
}

impl TopicNames {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            events: format!("cc.events.{}", cfg.service_id),
            commands: format!("cc.commands.{}", cfg.service_id),
            alerts: "cc.alerts".into(),
            audit: "cc.audit".into(),
        }
    }
}

/// Placeholder transport that prints to stdout.
pub struct StdoutTransport;

#[async_trait::async_trait]
impl Transport for StdoutTransport {
    async fn send_event(&self, topic: &str, event: &AgentEvent) -> Result<(), KafkaError> {
        let json = serde_json::to_string(event)?;
        info!("[{}] {}", topic, json);
        Ok(())
    }

    async fn receive_command(&mut self) -> Result<Option<CommandEnvelope>, KafkaError> {
        // Stub: no real commands in this minimal example.
        Ok(None)
    }
}

/// Secure wrapper: in a real build this would add envelope encryption,
/// replay protection (nonce + timestamp), and signature verification.
pub struct SecureTransport<T: Transport> {
    inner: T,
}

impl<T: Transport> SecureTransport<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub async fn send_event(&self, topic: &str, event: &AgentEvent) -> Result<(), KafkaError> {
        // TODO: encrypt envelope here.
        self.inner.send_event(topic, event).await
    }

    pub async fn receive_command(&mut self) -> Result<Option<CommandEnvelope>, KafkaError> {
        let cmd = self.inner.receive_command().await?;
        if let Some(ref c) = cmd {
            if c.signature.is_none() {
                warn!("command {} has no signature; rejecting", c.id);
                return Ok(None);
            }
            // TODO: verify Ed25519 signature.
        }
        Ok(cmd)
    }
}
