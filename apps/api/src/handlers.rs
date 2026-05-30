use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use shared_types::{GetNowPlayingResponse, UpdateNowPlayingRequest};
use tracing::info;

use crate::error::ApiError;
use crate::state::AppState;

pub async fn health() -> Json<Value> {
    Json(json!({ "ok": true }))
}

pub async fn get_now_playing(
    State(state): State<AppState>,
) -> Result<Json<GetNowPlayingResponse>, ApiError> {
    let guard = state
        .now_playing
        .read()
        .map_err(|_| ApiError::Internal)?;

    let Some(current) = guard.as_ref() else {
        return Err(ApiError::NotFound);
    };

    Ok(Json(current.clone().into()))
}

pub async fn update_now_playing(
    State(state): State<AppState>,
    Json(payload): Json<UpdateNowPlayingRequest>,
) -> Result<StatusCode, ApiError> {
    let now_playing = payload.into_now_playing();

    info!(
        track = %now_playing.track_name,
        artist = %now_playing.artist_name,
        is_playing = now_playing.is_playing,
        "received now-playing update"
    );

    let mut guard = state
        .now_playing
        .write()
        .map_err(|_| ApiError::Internal)?;

    *guard = Some(now_playing);

    Ok(StatusCode::NO_CONTENT)
}
