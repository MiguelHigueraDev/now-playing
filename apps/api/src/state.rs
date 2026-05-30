use std::sync::{Arc, RwLock};

use shared_types::NowPlaying;

/// In-memory application state. The `now_playing` slot will later be backed by SQLite/Redis.
#[derive(Clone)]
pub struct AppState {
    pub auth_token: String,
    pub now_playing: Arc<RwLock<Option<NowPlaying>>>,
}

impl AppState {
    pub fn new(auth_token: String) -> Self {
        Self {
            auth_token,
            now_playing: Arc::new(RwLock::new(None)),
        }
    }
}
