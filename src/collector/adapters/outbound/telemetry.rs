use super::core::{AgentEvent, TelemetryConfig};
use super::host::{HostBridge, HostError};
use chrono::Utc;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio::time;
use tracing::{debug, error, info, warn};

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("host bridge error: {0}")]
    Host(#[from] HostError),
    #[error("channel closed")]
    ChannelClosed,
}

#[derive(Debug, Clone)]
pub struct LogLine {
    pub source: PathBuf,
    pub line_no: usize,
    pub text: String,
    pub ts: chrono::DateTime<Utc>,
}

pub struct TelemetryCollector {
    config: TelemetryConfig,
    bridge: HostBridge,
    sender: mpsc::Sender<AgentEvent>,
}

impl TelemetryCollector {
    pub fn new(
        config: TelemetryConfig,
        bridge: HostBridge,
        sender: mpsc::Sender<AgentEvent>,
    ) -> Self {
        Self {
            config,
            bridge,
            sender,
        }
    }

    pub async fn run(&self, node_id: String, service_id: String) {
        let mut interval = time::interval(Duration::from_secs(self.config.poll_interval_secs));
        interval.set_missed_tick_behavior(time::MissedTickBehavior::Skip);

        info!(
            "telemetry collector started for {} paths",
            self.config.log_paths.len()
        );

        loop {
            interval.tick().await;
            for pattern in &self.config.log_paths {
                if let Err(e) = self.collect_pattern(pattern, &node_id, &service_id).await {
                    warn!("telemetry collection failed for {}: {}", pattern, e);
                }
            }
        }
    }

    async fn collect_pattern(
        &self,
        pattern: &str,
        node_id: &str,
        service_id: &str,
    ) -> Result<(), TelemetryError> {
        let path = Path::new(pattern);
        if path.is_file() {
            self.tail_file(path, node_id, service_id).await?;
        } else if path.is_dir() {
            let entries = self.bridge.list_dir(path).await?;
            for entry in entries {
                if entry.is_file()
                    && !entry
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .starts_with('.')
                {
                    self.tail_file(&entry, node_id, service_id).await?;
                }
            }
        } else {
            debug!("telemetry path does not exist yet: {}", pattern);
        }
        Ok(())
    }

    async fn tail_file(
        &self,
        path: &Path,
        node_id: &str,
        service_id: &str,
    ) -> Result<(), TelemetryError> {
        let bytes = self.bridge.read_file(path).await?;
        let text = String::from_utf8_lossy(&bytes);
        for (line_no, text) in text.lines().enumerate() {
            let line = LogLine {
                source: path.to_path_buf(),
                line_no: line_no + 1,
                text: text.to_string(),
                ts: Utc::now(),
            };
            let event = AgentEvent::new(
                node_id.into(),
                service_id.into(),
                "log_line",
                serde_json::json!({
                    "source": line.source,
                    "line_no": line.line_no,
                    "text": line.text,
                    "ts": line.ts,
                }),
            );
            if self.sender.send(event).await.is_err() {
                error!("telemetry channel closed");
                return Err(TelemetryError::ChannelClosed);
            }
        }
        Ok(())
    }
}
