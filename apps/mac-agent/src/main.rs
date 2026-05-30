mod agent;
mod api_client;
mod config;

use anyhow::Context;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use dotenvy::dotenv;
use music_provider::{AppleMusicProvider, MusicProvider};
use shared_types::{NowPlaying, UpdateNowPlayingRequest};
use tokio::time;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::agent::PlaybackSnapshot;
use crate::api_client::ApiClient;
use crate::config::AgentConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info,mac_agent=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AgentConfig::from_env()?;
    let provider = AppleMusicProvider;
    let api_client = ApiClient::new(config.api_base_url.clone(), config.auth_token.clone());

    info!(
        api = %config.api_base_url,
        poll_interval_secs = config.poll_interval.as_secs(),
        "mac-agent started"
    );

    let mut previous = PlaybackSnapshot::empty();
    let mut interval = time::interval(config.poll_interval);

    loop {
        interval.tick().await;

        match poll_and_sync(&provider, &api_client, &mut previous).await {
            Ok(sent) if sent => info!("sent now-playing update to API"),
            Ok(_) => {}
            Err(err) => error!(error = %err, "poll cycle failed"),
        }
    }
}

async fn poll_and_sync(
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

fn build_update_request(
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
