mod apple_music;
mod error;

pub use apple_music::AppleMusicProvider;
pub use error::{MusicProviderError, Result};

use shared_types::NowPlaying;

/// Abstraction over music playback sources (Apple Music, Spotify, browser, etc.).
pub trait MusicProvider {
    fn current_track(&self) -> Result<Option<NowPlaying>>;
}
