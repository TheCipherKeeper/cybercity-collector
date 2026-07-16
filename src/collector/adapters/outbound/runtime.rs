use super::command::CommandExecutor;
use super::core::{Config, Lifecycle, NodeIdentity, State};
use super::host::HostBridge;
use super::kafka::{SecureTransport, StdoutTransport, TopicNames, Transport};
use super::telemetry::TelemetryCollector;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info};

pub fn run() {
    if let Err(error) = run_inner() {
        error!("collector failed: {}", error);
        std::process::exit(1);
    }
}

#[tokio::main]
async fn run_inner() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "config/example.toml".into());

    info!("loading config from {}", config_path);
    let config = Config::load(&config_path)?;
    let identity = NodeIdentity::from_config(&config);
    info!(
        "collector starting: {} / {}",
        identity.node_id, identity.service_id
    );

    let lifecycle = Arc::new(Lifecycle::new(State::Initializing));
    let policy = Arc::new(config.policy.clone());
    let bridge = Arc::new(HostBridge::new(config.policy.clone()));
    let topics = TopicNames::from_config(&config);

    lifecycle.set(State::Active);

    let (event_tx, mut event_rx) = mpsc::channel(config.telemetry.buffer_size);

    // Telemetry collector task.
    let telemetry = TelemetryCollector::new(
        config.telemetry.clone(),
        HostBridge::new(config.policy.clone()),
        event_tx,
    );
    let node_id = identity.node_id.clone();
    let service_id = identity.service_id.clone();
    let telemetry_handle = tokio::spawn(async move {
        telemetry.run(node_id, service_id).await;
    });

    // Transport task: consume events and forward them to Kafka (stub stdout here).
    let transport = Arc::new(SecureTransport::new(StdoutTransport));
    let transport_clone = transport.clone();
    let events_topic = topics.events.clone();
    let transport_handle = tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            if let Err(e) = transport_clone.send_event(&events_topic, &event).await {
                error!("failed to send event: {}", e);
            }
        }
    });

    // Command listener task: poll stub transport for signed commands.
    let executor = Arc::new(CommandExecutor::new(policy, bridge, lifecycle.clone()));
    let command_handle = tokio::spawn(async move {
        let mut transport = StdoutTransport;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
            match transport.receive_command().await {
                Ok(Some(cmd)) => {
                    if let Err(e) = executor.execute(cmd).await {
                        error!("command execution failed: {}", e);
                    }
                }
                Ok(None) => {}
                Err(e) => error!("command receive error: {}", e),
            }
        }
    });

    // Graceful shutdown on SIGINT/SIGTERM.
    match tokio::signal::ctrl_c().await {
        Ok(()) => info!("shutdown signal received"),
        Err(e) => error!("failed to listen for ctrl-c: {}", e),
    }

    lifecycle.set(State::Initializing); // transitional on shutdown
    telemetry_handle.abort();
    transport_handle.abort();
    command_handle.abort();
    info!("collector stopped");
    Ok(())
}
