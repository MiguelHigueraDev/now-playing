use shared_types::NowPlaying;

/// Lightweight view of playback state used for change detection between poll cycles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaybackSnapshot {
    pub track_name: String,
    pub artist_name: String,
    pub album_name: String,
    pub is_playing: bool,
}

impl PlaybackSnapshot {
    pub fn empty() -> Self {
        Self {
            track_name: String::new(),
            artist_name: String::new(),
            album_name: String::new(),
            is_playing: false,
        }
    }

    pub fn from_track(track: Option<&NowPlaying>) -> Self {
        match track {
            Some(now_playing) => Self {
                track_name: now_playing.track_name.clone(),
                artist_name: now_playing.artist_name.clone(),
                album_name: now_playing.album_name.clone(),
                is_playing: true,
            },
            None => Self::empty(),
        }
    }

    /// Returns true when track identity or play/pause state changed.
    pub fn has_changed(&self, other: &Self) -> bool {
        self != other
    }
}
