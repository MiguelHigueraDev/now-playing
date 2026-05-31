pub mod agent;
pub mod api_client;
pub mod config;
pub mod config_store;

#[cfg(target_os = "macos")]
pub mod gl_window;
#[cfg(target_os = "macos")]
pub mod login_item;
#[cfg(target_os = "macos")]
pub mod preferences;
#[cfg(target_os = "macos")]
pub mod tray;

use std::path::Path;

use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use music_provider::{AppleMusicProvider, MusicProvider};
use shared_types::{NowPlaying, UpdateNowPlayingRequest};
use tokio::sync::{mpsc, watch};
use tokio::time::{self, MissedTickBehavior};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::agent::PlaybackSnapshot;
use crate::api_client::ApiClient;
use crate::config::AgentConfig;

#[derive(Debug, Clone)]
pub enum AgentStatus {
    Idle,
    Syncing,
    LastTrack(String),
    Error(String),
}

impl AgentStatus {
    pub fn menu_label(&self) -> String {
        match self {
            Self::Idle => "Status: Idle".to_string(),
            Self::Syncing => "Status: Syncing…".to_string(),
            Self::LastTrack(track) => format!("Status: {track}"),
            Self::Error(message) => format!("Status: Error — {message}"),
        }
    }
}

pub fn is_app_bundle() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|path| {
            path.parent()
                .and_then(|macos| macos.parent())
                .and_then(|contents| contents.parent())
                .map(|app| app.extension().and_then(|ext| ext.to_str()) == Some("app"))
        })
        .unwrap_or(false)
}

pub fn init_console_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mac_agent=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}

#[cfg(target_os = "macos")]
pub fn init_file_logging(log_dir: &Path) -> anyhow::Result<()> {
    use tracing_appender::non_blocking;
    use tracing_subscriber::fmt;

    std::fs::create_dir_all(log_dir).context("failed to create log directory")?;

    let file_appender = tracing_appender::rolling::never(log_dir, "agent.log");
    let (non_blocking_appender, guard) = non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,mac_agent=debug".into()),
        )
        .with(fmt::layer().with_writer(non_blocking_appender))
        .init();

    std::mem::forget(guard);
    Ok(())
}

pub async fn run_agent(
    mut config_rx: watch::Receiver<AgentConfig>,
    cancel: CancellationToken,
    status_tx: mpsc::Sender<AgentStatus>,
) -> anyhow::Result<()> {
    let provider = AppleMusicProvider;
    let mut previous = PlaybackSnapshot::empty();
    let mut interval = time::interval(config_rx.borrow().poll_interval());
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let _ = status_tx.send(AgentStatus::Idle).await;

    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                info!("mac-agent shutting down");
                break;
            }
            changed = config_rx.changed() => {
                if changed.is_err() {
                    break;
                }
                interval = time::interval(config_rx.borrow().poll_interval());
                interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
                info!(
                    api = %config_rx.borrow().api_base_url,
                    poll_interval_secs = config_rx.borrow().poll_interval().as_secs(),
                    "mac-agent config updated"
                );
            }
            _ = interval.tick() => {
                let config = config_rx.borrow().clone();
                if config.auth_token.is_empty() {
                    let _ = status_tx
                        .send(AgentStatus::Error(
                            "Configure auth token in Preferences".to_string(),
                        ))
                        .await;
                    continue;
                }

                let api_client =
                    ApiClient::new(config.api_base_url.clone(), config.auth_token.clone());

                let _ = status_tx.send(AgentStatus::Syncing).await;

                match poll_and_sync(&provider, &api_client, &mut previous).await {
                    Ok(true) => {
                        info!("sent now-playing update to API");
                        let label = if previous.track_name.is_empty() {
                            "Nothing playing".to_string()
                        } else {
                            format!(
                                "{} — {}",
                                previous.track_name, previous.artist_name
                            )
                        };
                        let _ = status_tx.send(AgentStatus::LastTrack(label)).await;
                    }
                    Ok(false) => {
                        let _ = status_tx.send(AgentStatus::Idle).await;
                    }
                    Err(err) => {
                        error!(error = %err, "poll cycle failed");
                        let _ = status_tx
                            .send(AgentStatus::Error(err.to_string()))
                            .await;
                    }
                }
            }
        }
    }

    Ok(())
}

pub async fn poll_and_sync(
    provider: &AppleMusicProvider,
    api_client: &ApiClient,
    previous: &mut PlaybackSnapshot,
) -> anyhow::Result<bool> {
    let current = match provider.current_track() {
        Ok(track) => track,
        Err(err) => {
            warn!(error = %err, "failed to read Apple Music state");
            return Ok(false);
        }
    };

    let snapshot = PlaybackSnapshot::from_track(current.as_ref());

    if !previous.has_changed(&snapshot) {
        return Ok(false);
    }

    let payload = build_update_request(provider, current.as_ref())?;
    api_client
        .post_now_playing(&payload)
        .await
        .context("failed to POST now-playing update")?;

    *previous = snapshot;
    Ok(true)
}

pub fn build_update_request(
    provider: &AppleMusicProvider,
    track: Option<&NowPlaying>,
) -> anyhow::Result<UpdateNowPlayingRequest> {
    match track {
        Some(now_playing) => {
            let mut request = UpdateNowPlayingRequest::from(now_playing.clone());
            request.artwork_base64 = provider
                .current_artwork()
                .ok()
                .flatten()
                .map(|artwork| STANDARD.encode(artwork.bytes));
            Ok(request)
        }
        None => Ok(UpdateNowPlayingRequest {
            track_name: String::new(),
            artist_name: String::new(),
            album_name: String::new(),
            artwork_url: None,
            artwork_base64: None,
            duration_seconds: None,
            position_seconds: None,
            is_playing: false,
        }),
    }
}
