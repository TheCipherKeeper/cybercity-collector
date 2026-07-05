use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

pub mod lifecycle;
pub mod policy;

pub use lifecycle::{Lifecycle, State};
pub use policy::{HostPermission, Policy};

#[derive(Debug, Clone, Error)]
#[error("configuration error: {0}")]
pub struct ConfigError(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub node_id: String,
    pub service_id: String,
    pub segment: String,

    #[serde(default = "default_kafka_broker")]
    pub kafka_broker: String,

    #[serde(default = "default_spool_path")]
    pub spool_path: String,

    #[serde(default)]
    pub telemetry: TelemetryConfig,

    #[serde(default)]
    pub policy: Policy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct TelemetryConfig {
    #[serde(default)]
    pub log_paths: Vec<String>,
    #[serde(default = "default_poll_interval_secs")]
    pub poll_interval_secs: u64,
    #[serde(default = "default_buffer_size")]
    pub buffer_size: usize,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let text = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError(format!("failed to read {:?}: {e}", path.as_ref())))?;
        let mut cfg: Config =
            toml::from_str(&text).map_err(|e| ConfigError(format!("failed to parse TOML: {e}")))?;
        cfg.merge_env();
        cfg.validate()?;
        Ok(cfg)
    }

    fn merge_env(&mut self) {
        if let Ok(v) = std::env::var("CCC_NODE_ID") {
            self.node_id = v;
        }
        if let Ok(v) = std::env::var("CCC_SERVICE_ID") {
            self.service_id = v;
        }
        if let Ok(v) = std::env::var("CCC_KAFKA_BROKER") {
            self.kafka_broker = v;
        }
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if self.node_id.is_empty() {
            return Err(ConfigError("node_id is required".into()));
        }
        if self.service_id.is_empty() {
            return Err(ConfigError("service_id is required".into()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub node_id: String,
    pub service_id: String,
    pub segment: String,
    pub boot_time: DateTime<Utc>,
}

impl NodeIdentity {
    pub fn from_config(cfg: &Config) -> Self {
        Self {
            node_id: cfg.node_id.clone(),
            service_id: cfg.service_id.clone(),
            segment: cfg.segment.clone(),
            boot_time: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentEvent {
    pub ts: DateTime<Utc>,
    pub node_id: String,
    pub service_id: String,
    pub kind: String,
    pub payload: serde_json::Value,
}

impl AgentEvent {
    pub fn new(
        node_id: String,
        service_id: String,
        kind: impl Into<String>,
        payload: impl Serialize,
    ) -> Self {
        Self {
            ts: Utc::now(),
            node_id,
            service_id,
            kind: kind.into(),
            payload: serde_json::to_value(payload).unwrap_or_default(),
        }
    }
}

fn default_kafka_broker() -> String {
    "localhost:9092".into()
}

fn default_spool_path() -> String {
    "/var/lib/cybercity-agent/spool".into()
}

fn default_poll_interval_secs() -> u64 {
    5
}

fn default_buffer_size() -> usize {
    1024
}
