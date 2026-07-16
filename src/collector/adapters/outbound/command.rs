use super::core::{AgentEvent, Lifecycle, Policy};
use super::host::{HostBridge, HostError};
use super::kafka::CommandEnvelope;
use serde_json::json;
use std::sync::Arc;
use thiserror::Error;
use tracing::{info, warn};

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("command kind not allowed: {0}")]
    NotAllowed(String),
    #[error("missing field: {0}")]
    MissingField(String),
    #[error("host error: {0}")]
    Host(#[from] HostError),
    #[error("invalid signature")]
    InvalidSignature,
}

pub struct CommandExecutor {
    policy: Arc<Policy>,
    bridge: Arc<HostBridge>,
    lifecycle: Arc<Lifecycle>,
}

impl CommandExecutor {
    pub fn new(policy: Arc<Policy>, bridge: Arc<HostBridge>, lifecycle: Arc<Lifecycle>) -> Self {
        Self {
            policy,
            bridge,
            lifecycle,
        }
    }

    /// Execute a command from the engine if it is allowed and (placeholder) signed.
    pub async fn execute(&self, cmd: CommandEnvelope) -> Result<AgentEvent, CommandError> {
        if !self.policy.can_run_command(&cmd.kind) {
            warn!("rejecting unauthorized command kind: {}", cmd.kind);
            self.lifecycle.record_tamper();
            return Err(CommandError::NotAllowed(cmd.kind));
        }

        info!("executing command {} (kind={})", cmd.id, cmd.kind);

        match cmd.kind.as_str() {
            "status" => self.handle_status(cmd),
            "read_file" => self.handle_read_file(cmd).await,
            other => {
                warn!("unknown command kind: {}", other);
                Err(CommandError::NotAllowed(other.into()))
            }
        }
    }

    fn handle_status(&self, cmd: CommandEnvelope) -> Result<AgentEvent, CommandError> {
        Ok(AgentEvent::new(
            "*".into(),
            cmd.service_id,
            "command_status",
            json!({
                "command_id": cmd.id,
                "state": self.lifecycle.current().to_string(),
                "tamper_count": self.lifecycle.tamper_count(),
            }),
        ))
    }

    async fn handle_read_file(&self, cmd: CommandEnvelope) -> Result<AgentEvent, CommandError> {
        let path = cmd
            .payload
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| CommandError::MissingField("path".into()))?;
        let data = self.bridge.read_file(std::path::Path::new(path)).await?;
        let text = String::from_utf8_lossy(&data);
        Ok(AgentEvent::new(
            "*".into(),
            cmd.service_id,
            "command_read_file",
            json!({
                "command_id": cmd.id,
                "path": path,
                "size": data.len(),
                "preview": text.chars().take(256).collect::<String>(),
            }),
        ))
    }
}
