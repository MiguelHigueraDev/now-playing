use std::process::Command;

use chrono::Utc;
use shared_types::NowPlaying;
use tracing::{debug, warn};

use crate::error::{MusicProviderError, Result};
use crate::MusicProvider;

const APPLESCRIPT: &str = r#"
tell application "Music"
    if player state is playing then
        return name of current track & "||" & artist of current track & "||" & album of current track
    else
        return "NOT_PLAYING"
    end if
end tell
"#;

/// Reads the current Apple Music playback state via `osascript`.
#[derive(Debug, Default, Clone, Copy)]
pub struct AppleMusicProvider;

impl AppleMusicProvider {
    fn run_applescript(&self) -> Result<String> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(APPLESCRIPT)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MusicProviderError::AppleScriptFailed(
                stderr.trim().to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    fn parse_output(raw: &str) -> Result<Option<NowPlaying>> {
        if raw == "NOT_PLAYING" {
            return Ok(None);
        }

        let parts: Vec<&str> = raw.split("||").collect();
        if parts.len() != 3 {
            return Err(MusicProviderError::UnexpectedOutput(raw.to_string()));
        }

        Ok(Some(NowPlaying {
            track_name: parts[0].trim().to_string(),
            artist_name: parts[1].trim().to_string(),
            album_name: parts[2].trim().to_string(),
            artwork_url: None,
            duration_seconds: None,
            position_seconds: None,
            is_playing: true,
            listened_at: Utc::now(),
        }))
    }
}

impl MusicProvider for AppleMusicProvider {
    fn current_track(&self) -> Result<Option<NowPlaying>> {
        let raw = self.run_applescript()?;
        debug!(raw_output = %raw, "Apple Music query result");

        match Self::parse_output(&raw) {
            Ok(track) => Ok(track),
            Err(err) => {
                warn!(error = %err, "failed to parse Apple Music output");
                Err(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AppleMusicProvider;

    #[test]
    fn parse_not_playing() {
        let result = AppleMusicProvider::parse_output("NOT_PLAYING").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn parse_track() {
        let result = AppleMusicProvider::parse_output("Song||Artist||Album").unwrap();
        let track = result.unwrap();
        assert_eq!(track.track_name, "Song");
        assert_eq!(track.artist_name, "Artist");
        assert_eq!(track.album_name, "Album");
        assert!(track.is_playing);
    }

    #[test]
    fn parse_invalid_output() {
        let result = AppleMusicProvider::parse_output("bad");
        assert!(result.is_err());
    }
}
