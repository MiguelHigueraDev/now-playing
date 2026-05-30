use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Canonical representation of the currently playing track.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NowPlaying {
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub artwork_url: Option<String>,
    pub duration_seconds: Option<u32>,
    pub position_seconds: Option<u32>,
    pub is_playing: bool,
    pub listened_at: DateTime<Utc>,
}

/// Payload sent by the mac-agent when playback state changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNowPlayingRequest {
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub artwork_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artwork_base64: Option<String>,
    pub duration_seconds: Option<u32>,
    pub position_seconds: Option<u32>,
    pub is_playing: bool,
}

impl UpdateNowPlayingRequest {
    /// Merge incoming fields with a server-side timestamp.
    pub fn into_now_playing(self, artwork_url: Option<String>) -> NowPlaying {
        NowPlaying {
            track_name: self.track_name,
            artist_name: self.artist_name,
            album_name: self.album_name,
            artwork_url: artwork_url.or(self.artwork_url),
            duration_seconds: self.duration_seconds,
            position_seconds: self.position_seconds,
            is_playing: self.is_playing,
            listened_at: Utc::now(),
        }
    }
}

/// Public API response for the current playback state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetNowPlayingResponse {
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub artwork_url: Option<String>,
    pub duration_seconds: Option<u32>,
    pub position_seconds: Option<u32>,
    pub is_playing: bool,
    pub listened_at: DateTime<Utc>,
}

impl From<NowPlaying> for GetNowPlayingResponse {
    fn from(value: NowPlaying) -> Self {
        Self {
            track_name: value.track_name,
            artist_name: value.artist_name,
            album_name: value.album_name,
            artwork_url: value.artwork_url,
            duration_seconds: value.duration_seconds,
            position_seconds: value.position_seconds,
            is_playing: value.is_playing,
            listened_at: value.listened_at,
        }
    }
}

impl From<NowPlaying> for UpdateNowPlayingRequest {
    fn from(value: NowPlaying) -> Self {
        Self {
            track_name: value.track_name,
            artist_name: value.artist_name,
            album_name: value.album_name,
            artwork_url: value.artwork_url,
            artwork_base64: None,
            duration_seconds: value.duration_seconds,
            position_seconds: value.position_seconds,
            is_playing: value.is_playing,
        }
    }
}
