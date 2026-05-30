use std::fs;
use std::path::PathBuf;
use std::process::Command;

use chrono::Utc;
use shared_types::NowPlaying;
use tracing::{debug, warn};

use crate::error::{MusicProviderError, Result};
use crate::MusicProvider;

const TRACK_APPLESCRIPT: &str = r#"
tell application "Music"
    set ps to player state as string
    if ps is "playing" or ps is "paused" then
        set t to current track
        set isPlaying to ps is "playing"
        return name of t & "||" & artist of t & "||" & album of t & "||" & (duration of t as string) & "||" & (player position as string) & "||" & isPlaying
    else
        return "NOT_PLAYING"
    end if
end tell
"#;

/// Album artwork bytes extracted from the Music app.
#[derive(Debug, Clone)]
pub struct TrackArtwork {
    pub bytes: Vec<u8>,
    pub content_type: &'static str,
}

/// Reads the current Apple Music playback state via `osascript`.
#[derive(Debug, Default, Clone, Copy)]
pub struct AppleMusicProvider;

impl AppleMusicProvider {
    fn run_applescript(&self, script: &str) -> Result<String> {
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(MusicProviderError::AppleScriptFailed(
                stderr.trim().to_string(),
            ));
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Exports the current track's artwork to a temp file and reads it back.
    pub fn current_artwork(&self) -> Result<Option<TrackArtwork>> {
        let path = Self::artwork_cache_path();
        let path_str = path.to_string_lossy();
        let script = format!(
            r#"
tell application "Music"
    set ps to player state as string
    if ps is not "playing" and ps is not "paused" then
        return "NOT_PLAYING"
    end if
    set t to current track
    if (count of artworks of t) is 0 then
        return "NO_ART"
    end if
    set artPath to "{path_str}"
    tell artwork 1 of t
        if format is «class PNG » then
            set ext to ".png"
        else
            set ext to ".jpg"
        end if
        set srcBytes to raw data
    end tell
    if ext is ".png" then
        set artPath to my replace_text(artPath, ".jpg", ".png")
    end if
    set outFile to open for access POSIX file artPath with write permission
    set eof outFile to 0
    write srcBytes to outFile
    close access outFile
    return artPath
end tell

on replace_text(sourceText, findText, replaceText)
    set AppleScript's text item delimiters to findText
    set parts to text items of sourceText
    set AppleScript's text item delimiters to replaceText
    return parts as text
end replace_text
"#
        );

        let raw = self.run_applescript(&script)?;
        if raw == "NOT_PLAYING" || raw == "NO_ART" {
            return Ok(None);
        }

        let content_type = if raw.ends_with(".png") {
            "image/png"
        } else {
            "image/jpeg"
        };

        let bytes = fs::read(&raw).map_err(|err| {
            MusicProviderError::AppleScriptFailed(format!("failed to read artwork file: {err}"))
        })?;

        if bytes.is_empty() {
            return Ok(None);
        }

        Ok(Some(TrackArtwork {
            bytes,
            content_type,
        }))
    }

    fn artwork_cache_path() -> PathBuf {
        std::env::temp_dir().join("now-playing-artwork.jpg")
    }

    fn parse_seconds(raw: &str) -> Option<u32> {
        let normalized = raw.trim().replace(',', ".");
        normalized
            .parse::<f64>()
            .ok()
            .map(|value| value.round().max(0.0) as u32)
    }

    fn parse_output(raw: &str) -> Result<Option<NowPlaying>> {
        if raw == "NOT_PLAYING" {
            return Ok(None);
        }

        let parts: Vec<&str> = raw.split("||").collect();
        if parts.len() != 6 {
            return Err(MusicProviderError::UnexpectedOutput(raw.to_string()));
        }

        let is_playing = matches!(parts[5].trim(), "true" | "1");

        Ok(Some(NowPlaying {
            track_name: parts[0].trim().to_string(),
            artist_name: parts[1].trim().to_string(),
            album_name: parts[2].trim().to_string(),
            artwork_url: None,
            duration_seconds: Self::parse_seconds(parts[3]),
            position_seconds: Self::parse_seconds(parts[4]),
            is_playing,
            listened_at: Utc::now(),
        }))
    }
}

impl MusicProvider for AppleMusicProvider {
    fn current_track(&self) -> Result<Option<NowPlaying>> {
        let raw = self.run_applescript(TRACK_APPLESCRIPT)?;
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
        let result =
            AppleMusicProvider::parse_output("Song||Artist||Album||258,56||19,31||true").unwrap();
        let track = result.unwrap();
        assert_eq!(track.track_name, "Song");
        assert_eq!(track.artist_name, "Artist");
        assert_eq!(track.album_name, "Album");
        assert_eq!(track.duration_seconds, Some(259));
        assert_eq!(track.position_seconds, Some(19));
        assert!(track.is_playing);
    }

    #[test]
    fn parse_paused_track() {
        let result =
            AppleMusicProvider::parse_output("Song||Artist||Album||180||45||false").unwrap();
        let track = result.unwrap();
        assert!(!track.is_playing);
        assert_eq!(track.position_seconds, Some(45));
    }

    #[test]
    fn parse_invalid_output() {
        let result = AppleMusicProvider::parse_output("bad");
        assert!(result.is_err());
    }
}
