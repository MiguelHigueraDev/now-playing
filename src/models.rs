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

/// Payload sent by the menu bar agent when playback state changes.
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

/// Playback position at `at`, extrapolating from `listened_at` while the track is playing.
pub fn extrapolated_position_seconds(now: &NowPlaying, at: DateTime<Utc>) -> u32 {
    let base = now.position_seconds.unwrap_or(0);
    if !now.is_playing {
        return base;
    }

    let elapsed = (at - now.listened_at).num_seconds().max(0) as u32;
    let current = base.saturating_add(elapsed);

    match now.duration_seconds {
        Some(duration) => current.min(duration),
        None => current,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn extrapolates_position_while_playing() {
        let listened_at = Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let now_playing = NowPlaying {
            track_name: String::new(),
            artist_name: String::new(),
            album_name: String::new(),
            artwork_url: None,
            duration_seconds: Some(300),
            position_seconds: Some(60),
            is_playing: true,
            listened_at,
        };

        let at = listened_at + chrono::Duration::seconds(45);
        assert_eq!(extrapolated_position_seconds(&now_playing, at), 105);
    }

    #[test]
    fn freezes_position_when_paused() {
        let listened_at = Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let now_playing = NowPlaying {
            track_name: String::new(),
            artist_name: String::new(),
            album_name: String::new(),
            artwork_url: None,
            duration_seconds: Some(300),
            position_seconds: Some(60),
            is_playing: false,
            listened_at,
        };

        let at = listened_at + chrono::Duration::seconds(120);
        assert_eq!(extrapolated_position_seconds(&now_playing, at), 60);
    }

    #[test]
    fn clamps_to_duration() {
        let listened_at = Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let now_playing = NowPlaying {
            track_name: String::new(),
            artist_name: String::new(),
            album_name: String::new(),
            artwork_url: None,
            duration_seconds: Some(100),
            position_seconds: Some(90),
            is_playing: true,
            listened_at,
        };

        let at = listened_at + chrono::Duration::seconds(30);
        assert_eq!(extrapolated_position_seconds(&now_playing, at), 100);
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
