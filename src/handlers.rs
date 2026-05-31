use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderValue, StatusCode},
    response::Response,
    Json,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use chrono::Utc;
use serde_json::{json, Value};
use crate::models::{GetNowPlayingResponse, UpdateNowPlayingRequest};
use tracing::info;

use crate::error::ApiError;
use crate::state::{AppState, StoredArtwork};
use crate::svg::{render_now_playing_svg, SvgRenderInput};

const ARTWORK_URL: &str = "/api/now-playing/artwork";

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

pub async fn get_now_playing_image(State(state): State<AppState>) -> Result<Response, ApiError> {
    let now_guard = state
        .now_playing
        .read()
        .map_err(|_| ApiError::Internal)?;

    let Some(current) = now_guard.as_ref() else {
        return Err(ApiError::NotFound);
    };

    let art_guard = state.artwork.read().map_err(|_| ApiError::Internal)?;
    let artwork = (*art_guard).as_ref();

    let svg = render_now_playing_svg(SvgRenderInput {
        now_playing: current,
        artwork,
        at: Utc::now(),
    });

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/svg+xml; charset=utf-8")
        .header(
            header::CACHE_CONTROL,
            HeaderValue::from_static("no-cache, max-age=0"),
        )
        .body(Body::from(svg))
        .map_err(|_| ApiError::Internal)?)
}

pub async fn get_artwork(State(state): State<AppState>) -> Result<Response, ApiError> {
    let guard = state.artwork.read().map_err(|_| ApiError::Internal)?;

    let Some(artwork) = guard.as_ref() else {
        return Err(ApiError::NotFound);
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, artwork.content_type.as_str())
        .header(header::CACHE_CONTROL, "public, max-age=3600")
        .body(Body::from(artwork.bytes.clone()))
        .map_err(|_| ApiError::Internal)?)
}

pub async fn update_now_playing(
    State(state): State<AppState>,
    Json(payload): Json<UpdateNowPlayingRequest>,
) -> Result<StatusCode, ApiError> {
    let artwork_url = store_artwork(&state, payload.artwork_base64.as_deref())?;
    let now_playing = payload.into_now_playing(artwork_url);

    info!(
        track = %now_playing.track_name,
        artist = %now_playing.artist_name,
        is_playing = now_playing.is_playing,
        duration_seconds = ?now_playing.duration_seconds,
        position_seconds = ?now_playing.position_seconds,
        has_artwork = now_playing.artwork_url.is_some(),
        "received now-playing update"
    );

    let mut guard = state
        .now_playing
        .write()
        .map_err(|_| ApiError::Internal)?;

    *guard = Some(now_playing);

    Ok(StatusCode::NO_CONTENT)
}

fn store_artwork(
    state: &AppState,
    artwork_base64: Option<&str>,
) -> Result<Option<String>, ApiError> {
    let mut guard = state.artwork.write().map_err(|_| ApiError::Internal)?;

    let Some(encoded) = artwork_base64 else {
        *guard = None;
        return Ok(None);
    };

    let bytes = STANDARD
        .decode(encoded.trim())
        .map_err(|_| ApiError::BadRequest("invalid artwork_base64".into()))?;

    if bytes.is_empty() {
        *guard = None;
        return Ok(None);
    }

    let content_type = if bytes.starts_with(b"\x89PNG") {
        "image/png"
    } else {
        "image/jpeg"
    };

    *guard = Some(StoredArtwork {
        bytes,
        content_type: content_type.to_string(),
    });

    Ok(Some(ARTWORK_URL.to_string()))
}
