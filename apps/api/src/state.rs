use std::sync::{Arc, RwLock};

use shared_types::NowPlaying;

/// Cached album artwork served at `/api/now-playing/artwork`.
#[derive(Debug, Clone)]
pub struct StoredArtwork {
    pub bytes: Vec<u8>,
    pub content_type: String,
}

/// In-memory application state. The `now_playing` slot will later be backed by SQLite/Redis.
#[derive(Clone)]
pub struct AppState {
    pub auth_token: String,
    pub now_playing: Arc<RwLock<Option<NowPlaying>>>,
    pub artwork: Arc<RwLock<Option<StoredArtwork>>>,
}

impl AppState {
    pub fn new(auth_token: String) -> Self {
        Self {
            auth_token,
            now_playing: Arc::new(RwLock::new(None)),
            artwork: Arc::new(RwLock::new(None)),
        }
    }
}
