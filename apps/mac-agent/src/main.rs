use dotenvy::dotenv;
use tokio::sync::mpsc;
use tracing::info;

use mac_agent::config::AgentConfig;
use mac_agent::{init_console_logging, run_agent};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(target_os = "macos")]
    if mac_agent::is_app_bundle() {
        return mac_agent::tray::run_app();
    }

    run_cli().await
}

async fn run_cli() -> anyhow::Result<()> {
    dotenv().ok();
    init_console_logging();

    let config = AgentConfig::from_env()?;

    let (_config_tx, config_rx) = tokio::sync::watch::channel(config.clone());
    let (status_tx, mut status_rx) = mpsc::channel(32);
    let cancel = tokio_util::sync::CancellationToken::new();

    info!(
        api = %config.api_base_url,
        poll_interval_secs = config.poll_interval_secs,
        "mac-agent started (CLI mode)"
    );

    let agent_cancel = cancel.clone();
    let agent_handle = tokio::spawn(async move {
        if let Err(err) = run_agent(config_rx, agent_cancel, status_tx).await {
            tracing::error!(error = %err, "agent task exited with error");
        }
    });

    let status_handle = tokio::spawn(async move {
        while let Some(status) = status_rx.recv().await {
            tracing::debug!(status = %status.menu_label(), "agent status");
        }
    });

    tokio::signal::ctrl_c().await?;
    cancel.cancel();
    agent_handle.await?;
    status_handle.abort();

    Ok(())
}
