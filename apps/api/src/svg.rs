use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use chrono::{DateTime, Utc};
use shared_types::{extrapolated_position_seconds, NowPlaying};

use crate::colors::svg_theme_from_artwork;
use crate::state::StoredArtwork;

const WIDTH: u32 = 720;
const HEIGHT: u32 = 220;
const PADDING: u32 = 28;
const ART_GAP_ABOVE_PROGRESS: u32 = 12;
const BAR_Y: u32 = HEIGHT - 44;
const BAR_WIDTH: u32 = WIDTH - PADDING * 2;
const BAR_H: u32 = 5;
const ART_RX: u32 = 16;
const CARD_RX: u32 = 20;
const FONT_SANS: &str =
    "ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif";
/// Album art bottom edge; progress UI starts below this with `ART_GAP_ABOVE_PROGRESS`.
const ART_BOTTOM: u32 = BAR_Y - ART_GAP_ABOVE_PROGRESS;
const ART_SIZE: u32 = ART_BOTTOM - PADDING;
const TEXT_X: u32 = PADDING + ART_SIZE + 28;

const TRACK_FONT: u32 = 26;
const ARTIST_FONT: u32 = 16;
const ALBUM_FONT: u32 = 13;
const IDLE_MESSAGE_FONT: u32 = 18;
const TRACK_TO_ARTIST: u32 = 30;
const ARTIST_TO_ALBUM: u32 = 24;
const IDLE_MESSAGE: &str = "Not listening to anything";

/// Baselines for track, artist, and album — vertically centered in the art column.
fn metadata_baselines() -> (u32, u32, u32) {
    let region_mid = (PADDING + ART_BOTTOM) / 2;
    let track_ascent = TRACK_FONT * 3 / 4;
    let album_descent = ALBUM_FONT / 4;
    let block_height = track_ascent + TRACK_TO_ARTIST + ARTIST_TO_ALBUM + album_descent;
    let block_top = region_mid.saturating_sub(block_height / 2);
    let track_y = block_top + track_ascent;
    let artist_y = track_y + TRACK_TO_ARTIST;
    let album_y = artist_y + ARTIST_TO_ALBUM;
    (track_y, artist_y, album_y)
}

#[derive(Debug, Clone)]
pub struct SvgRenderInput<'a> {
    pub now_playing: &'a NowPlaying,
    pub artwork: Option<&'a StoredArtwork>,
    pub at: DateTime<Utc>,
}

fn is_idle(now_playing: &NowPlaying) -> bool {
    now_playing.track_name.is_empty()
}

/// Baseline for the centered idle message — vertically centered above the progress bar.
fn idle_message_baseline() -> u32 {
    let region_mid = (PADDING + ART_BOTTOM) / 2;
    let ascent = IDLE_MESSAGE_FONT * 3 / 4;
    let descent = IDLE_MESSAGE_FONT / 4;
    region_mid + (ascent.saturating_sub(descent)) / 2
}

pub fn render_now_playing_svg(input: SvgRenderInput<'_>) -> String {
    if is_idle(input.now_playing) {
        return render_idle_svg();
    }

    let position = extrapolated_position_seconds(input.now_playing, input.at);
    let duration = input.now_playing.duration_seconds.unwrap_or(0);
    let progress = if duration > 0 {
        (position as f64 / duration as f64).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let fill_width = (BAR_WIDTH as f64 * progress).round() as u32;
    let thumb_x = PADDING + fill_width;
    let duration_label_x = WIDTH - PADDING;
    let (track_y, artist_y, album_y) = metadata_baselines();
    let bar_center_y = BAR_Y + BAR_H / 2;
    let time_label_y = BAR_Y + 20;

    let track = truncate(&input.now_playing.track_name, 40);
    let artist = truncate(&input.now_playing.artist_name, 46);
    let album = truncate(&input.now_playing.album_name, 46);
    let position_label = format_mm_ss(position);
    let duration_label = format_mm_ss(duration);

    let art_image = artwork_image_href(input.artwork);
    let art_foreground = artwork_foreground(&art_image, ART_SIZE);
    let artwork_bytes = input.artwork.map(|art| art.bytes.as_slice());
    let (bg_gradient_defs, bg_markup, accent_color) =
        svg_theme_from_artwork(artwork_bytes, WIDTH, HEIGHT);
    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-label="Now playing: {track}">
  <defs>
{bg_gradient_defs}    <linearGradient id="art-shine" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#ffffff" stop-opacity="0.12"/>
      <stop offset="40%" stop-color="#ffffff" stop-opacity="0"/>
    </linearGradient>
    <filter id="art-shadow" x="-20%" y="-10%" width="140%" height="130%">
      <feDropShadow dx="0" dy="10" stdDeviation="14" flood-color="#000000" flood-opacity="0.55"/>
    </filter>
    <clipPath id="art-clip">
      <rect x="{PADDING}" y="{PADDING}" width="{ART_SIZE}" height="{ART_SIZE}" rx="{ART_RX}"/>
    </clipPath>
    <clipPath id="card-clip">
      <rect width="{WIDTH}" height="{HEIGHT}" rx="{CARD_RX}"/>
    </clipPath>
  </defs>
  <g clip-path="url(#card-clip)">
{bg_markup}
    <g filter="url(#art-shadow)" clip-path="url(#art-clip)">
      {art_foreground}
      <rect x="{PADDING}" y="{PADDING}" width="{ART_SIZE}" height="{ART_SIZE}" fill="url(#art-shine)" pointer-events="none"/>
    </g>
    <rect x="{PADDING}" y="{PADDING}" width="{ART_SIZE}" height="{ART_SIZE}" rx="{ART_RX}" fill="none" stroke="#ffffff" stroke-opacity="0.1" stroke-width="1"/>
    <rect x="{PADDING}" y="{PADDING}" width="{ART_SIZE}" height="{ART_SIZE}" rx="{ART_RX}" fill="none" stroke="#000000" stroke-opacity="0.35" stroke-width="1" transform="translate(0 1)"/>
    <text x="{TEXT_X}" y="{track_y}" fill="#faf8f5" font-family="{FONT_SANS}" font-size="{TRACK_FONT}" font-weight="700" letter-spacing="-0.02em">{track}</text>
    <text x="{TEXT_X}" y="{artist_y}" fill="#d8d4cc" font-family="{FONT_SANS}" font-size="{ARTIST_FONT}" font-weight="500">{artist}</text>
    <text x="{TEXT_X}" y="{album_y}" fill="#8f8a82" font-family="{FONT_SANS}" font-size="{ALBUM_FONT}" font-weight="400">{album}</text>
    <rect x="{PADDING}" y="{BAR_Y}" width="{BAR_WIDTH}" height="{BAR_H}" rx="3" fill="#ffffff" fill-opacity="0.08"/>
    <rect x="{PADDING}" y="{BAR_Y}" width="{fill_width}" height="{BAR_H}" rx="3" fill="{accent_color}"/>
    <circle cx="{thumb_x}" cy="{bar_center_y}" r="6" fill="{accent_color}" opacity="{thumb_opacity}"/>
    <text x="{PADDING}" y="{time_label_y}" fill="#a39e94" font-family="{FONT_SANS}" font-size="11" font-variant-numeric="tabular-nums" letter-spacing="0.04em">{position_label}</text>
    <text x="{duration_label_x}" y="{time_label_y}" fill="#a39e94" font-family="{FONT_SANS}" font-size="11" font-variant-numeric="tabular-nums" letter-spacing="0.04em" text-anchor="end">{duration_label}</text>
  </g>
  <rect width="{WIDTH}" height="{HEIGHT}" rx="{CARD_RX}" fill="none" stroke="#ffffff" stroke-opacity="0.07" stroke-width="1"/>
</svg>"##,
        thumb_opacity = if fill_width > 4 { "1" } else { "0" },
    )
}

fn render_idle_svg() -> String {
    let duration_label_x = WIDTH - PADDING;
    let time_label_y = BAR_Y + 20;
    let message_y = idle_message_baseline();
    let message_x = WIDTH / 2;
    let (_, bg_markup, _) = svg_theme_from_artwork(None, WIDTH, HEIGHT);

    format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-label="{IDLE_MESSAGE}">
  <defs>
    <clipPath id="card-clip">
      <rect width="{WIDTH}" height="{HEIGHT}" rx="{CARD_RX}"/>
    </clipPath>
  </defs>
  <g clip-path="url(#card-clip)">
{bg_markup}
    <text x="{message_x}" y="{message_y}" fill="#8f8a82" font-family="{FONT_SANS}" font-size="{IDLE_MESSAGE_FONT}" font-weight="500" text-anchor="middle">{IDLE_MESSAGE}</text>
    <rect x="{PADDING}" y="{BAR_Y}" width="{BAR_WIDTH}" height="{BAR_H}" rx="3" fill="#ffffff" fill-opacity="0.08"/>
    <text x="{PADDING}" y="{time_label_y}" fill="#a39e94" font-family="{FONT_SANS}" font-size="11" font-variant-numeric="tabular-nums" letter-spacing="0.04em">0:00</text>
    <text x="{duration_label_x}" y="{time_label_y}" fill="#a39e94" font-family="{FONT_SANS}" font-size="11" font-variant-numeric="tabular-nums" letter-spacing="0.04em" text-anchor="end">0:00</text>
  </g>
  <rect width="{WIDTH}" height="{HEIGHT}" rx="{CARD_RX}" fill="none" stroke="#ffffff" stroke-opacity="0.07" stroke-width="1"/>
</svg>"##,
    )
}

fn artwork_image_href(artwork: Option<&StoredArtwork>) -> Option<String> {
    let artwork = artwork?;
    let encoded = STANDARD.encode(&artwork.bytes);
    Some(format!("data:{};base64,{}", artwork.content_type, encoded))
}

fn artwork_foreground(href: &Option<String>, size: u32) -> String {
    let Some(href) = href else {
        let center_x = PADDING + size / 2;
        let center_y = PADDING + size / 2;
        return format!(
            r##"<rect x="{PADDING}" y="{PADDING}" width="{size}" height="{size}" fill="#1a191e"/>
  <text x="{center_x}" y="{center_y}" fill="#5c584f" font-family="{FONT_SANS}" font-size="52" text-anchor="middle" dominant-baseline="middle">♪</text>"##
        );
    };

    format!(
        r##"<image x="{PADDING}" y="{PADDING}" width="{size}" height="{size}" preserveAspectRatio="xMidYMid slice" href="{href}"/>"##
    )
}

fn format_mm_ss(total_seconds: u32) -> String {
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{minutes}:{seconds:02}")
}

fn truncate(text: &str, max_chars: usize) -> String {
    let escaped = escape_xml(text);
    if escaped.chars().count() <= max_chars {
        return escaped;
    }

    let mut out = String::new();
    for (index, ch) in escaped.chars().enumerate() {
        if index >= max_chars.saturating_sub(1) {
            out.push('…');
            break;
        }
        out.push(ch);
    }
    out
}

fn escape_xml(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::metadata_baselines;
    use super::*;
    use chrono::TimeZone;
    use shared_types::NowPlaying;

    #[test]
    fn renders_track_metadata() {
        let listened_at = Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap();
        let at = listened_at + chrono::Duration::seconds(30);
        let now_playing = NowPlaying {
            track_name: "Test Song".into(),
            artist_name: "Test Artist".into(),
            album_name: "Test Album".into(),
            artwork_url: None,
            duration_seconds: Some(200),
            position_seconds: Some(10),
            is_playing: true,
            listened_at,
        };

        let svg = render_now_playing_svg(SvgRenderInput {
            now_playing: &now_playing,
            artwork: None,
            at,
        });

        assert!(svg.contains("Test Song"));
        assert!(svg.contains("Test Artist"));
        assert!(svg.contains("Test Album"));
        assert!(!svg.contains("thumb-glow"));
        assert!(svg.contains(r#"y="176""#));
        assert!(PADDING + ART_SIZE + ART_GAP_ABOVE_PROGRESS <= BAR_Y);
    }

    #[test]
    fn metadata_is_vertically_centered_in_art_region() {
        let (track_y, _, album_y) = metadata_baselines();
        let region_mid = (PADDING + ART_BOTTOM) / 2;
        let block_mid = (track_y - TRACK_FONT * 3 / 4 + album_y + ALBUM_FONT / 4) / 2;
        let diff = block_mid.abs_diff(region_mid);
        assert!(diff <= 2, "expected mid {region_mid}, got {block_mid}");
    }

    #[test]
    fn renders_idle_state_without_album_art() {
        let listened_at = Utc::now();
        let now_playing = NowPlaying {
            track_name: String::new(),
            artist_name: String::new(),
            album_name: String::new(),
            artwork_url: None,
            duration_seconds: None,
            position_seconds: None,
            is_playing: false,
            listened_at,
        };

        let svg = render_now_playing_svg(SvgRenderInput {
            now_playing: &now_playing,
            artwork: None,
            at: listened_at,
        });

        assert!(svg.contains("Not listening to anything"));
        assert!(!svg.contains("art-clip"));
        assert!(!svg.contains("♪"));
    }

    #[test]
    fn escapes_special_characters() {
        let listened_at = Utc::now();
        let now_playing = NowPlaying {
            track_name: "Rock & Roll".into(),
            artist_name: "A < B".into(),
            album_name: "Album".into(),
            artwork_url: None,
            duration_seconds: Some(100),
            position_seconds: Some(0),
            is_playing: false,
            listened_at,
        };

        let svg = render_now_playing_svg(SvgRenderInput {
            now_playing: &now_playing,
            artwork: None,
            at: listened_at,
        });

        assert!(svg.contains("Rock &amp; Roll"));
        assert!(svg.contains("A &lt; B"));
        assert!(!svg.contains("PLAYING"));
        assert!(!svg.contains("PAUSED"));
    }
}
