use std::collections::HashMap;

use image::imageops::FilterType;
use image::GenericImageView;

/// Maximum relative luminance for background gradient stops so light text stays readable.
const MAX_BG_LUMINANCE: f64 = 0.14;

/// Minimum perceptual distance between two extracted palette colors (0–1 scale).
const MIN_COLOR_DISTANCE: f64 = 0.12;

const FALLBACK_BG: &str = "#121116";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Extract the `count` most frequent distinct colors from encoded image bytes.
pub fn dominant_colors_from_image(bytes: &[u8], count: usize) -> Option<Vec<Rgb>> {
    let img = image::load_from_memory(bytes).ok()?;
    let thumb = img.resize_exact(64, 64, FilterType::Triangle);
    let mut buckets: HashMap<(u8, u8, u8), (u32, u64, u64, u64)> = HashMap::new();

    for pixel in thumb.pixels() {
        let [r, g, b, a] = pixel.2.0;
        if a < 128 {
            continue;
        }
        let color = Rgb { r, g, b };
        if is_neutral_background_pixel(color) {
            continue;
        }
        let key = (r >> 3, g >> 3, b >> 3);
        let entry = buckets.entry(key).or_insert((0, 0, 0, 0));
        entry.0 += 1;
        entry.1 += u64::from(r);
        entry.2 += u64::from(g);
        entry.3 += u64::from(b);
    }

    let mut ranked: Vec<_> = buckets.into_iter().collect();
    ranked.sort_by(|a, b| {
        let hits_a = a.1.0;
        let hits_b = b.1.0;
        hits_b.cmp(&hits_a).then_with(|| a.0.cmp(&b.0))
    });

    let mut palette = Vec::with_capacity(count);
    for (_, (hits, sum_r, sum_g, sum_b)) in ranked {
        if hits == 0 {
            break;
        }
        let candidate = Rgb {
            r: (sum_r / u64::from(hits)) as u8,
            g: (sum_g / u64::from(hits)) as u8,
            b: (sum_b / u64::from(hits)) as u8,
        };
        if palette
            .iter()
            .any(|existing| color_distance(existing, &candidate) < MIN_COLOR_DISTANCE)
        {
            continue;
        }
        palette.push(candidate);
        if palette.len() == count {
            break;
        }
    }

    if palette.is_empty() {
        None
    } else {
        Some(palette)
    }
}

/// Darken a color while preserving hue until it meets the legibility luminance cap.
pub fn darken_for_legibility(color: Rgb, max_luminance: f64) -> Rgb {
    let lum = relative_luminance(color);
    if lum <= max_luminance {
        return color;
    }

    let (h, s, mut l) = rgb_to_hsl(color);
    if s < 0.08 {
        return Rgb {
            r: 18,
            g: 17,
            b: 22,
        };
    }

    l = max_luminance.min(l * 0.35);
    hsl_to_rgb(h, s, l)
}

const FALLBACK_ACCENT: Rgb = Rgb {
    r: 245,
    g: 208,
    b: 138,
};

/// Background mesh markup plus accent color for progress UI, from a single palette pass.
pub fn svg_theme_from_artwork(
    artwork_bytes: Option<&[u8]>,
    width: u32,
    height: u32,
) -> (String, String, String) {
    let Some(bytes) = artwork_bytes else {
        return (
            String::new(),
            format!(r#"<rect width="{width}" height="{height}" fill="{FALLBACK_BG}"/>"#),
            rgb_to_hex(FALLBACK_ACCENT),
        );
    };

    let Some(mut colors) = dominant_colors_from_image(bytes, 3) else {
        return (
            String::new(),
            format!(r#"<rect width="{width}" height="{height}" fill="{FALLBACK_BG}"/>"#),
            rgb_to_hex(FALLBACK_ACCENT),
        );
    };

    while colors.len() < 3 {
        colors.push(*colors.last().unwrap_or(&Rgb {
            r: 18,
            g: 17,
            b: 22,
        }));
    }

    let accent = rgb_to_hex(accent_from_dominant(colors[0]));
    let (defs, markup) = svg_background_from_palette(bytes, &colors, width, height);
    (defs, markup, accent)
}

/// Build SVG background markup: radial color blobs with a deterministic layout per artwork.
pub fn svg_background_from_artwork(
    artwork_bytes: Option<&[u8]>,
    width: u32,
    height: u32,
) -> (String, String) {
    let (defs, markup, _) = svg_theme_from_artwork(artwork_bytes, width, height);
    (defs, markup)
}

fn svg_background_from_palette(
    bytes: &[u8],
    colors: &[Rgb],
    width: u32,
    height: u32,
) -> (String, String) {
    let dark_colors: Vec<Rgb> = colors
        .iter()
        .map(|color| darken_for_legibility(*color, MAX_BG_LUMINANCE))
        .collect();
    let base = darkest_color(&dark_colors);

    let mut rng = DetRng::from_bytes(bytes);
    let mut gradient_defs = String::new();
    let mut blob_layers = String::new();

    const BLOB_COUNT: usize = 7;
    for index in 0..BLOB_COUNT {
        let cx = rng.range(0.02, 0.98);
        let cy = rng.range(0.0, 1.0);
        let radius = rng.range(0.42, 1.05);
        let color = dark_colors[index % dark_colors.len()];
        let id = format!("bg-blob-{index}");

        gradient_defs.push_str(&format!(
            r#"    <radialGradient id="{id}" cx="{cx:.4}" cy="{cy:.4}" r="{radius:.4}" gradientUnits="objectBoundingBox">
      <stop offset="0%" stop-color="{hex}" stop-opacity="0.92"/>
      <stop offset="45%" stop-color="{hex}" stop-opacity="0.38"/>
      <stop offset="100%" stop-color="{hex}" stop-opacity="0"/>
    </radialGradient>
"#,
            hex = rgb_to_hex(color)
        ));
        blob_layers.push_str(&format!(
            r#"      <rect width="{width}" height="{height}" fill="url(#{id})"/>"#,
        ));
    }

    let defs = format!(
        r#"    <filter id="bg-soften" x="-30%" y="-30%" width="160%" height="160%">
      <feGaussianBlur stdDeviation="32"/>
    </filter>
{gradient_defs}"#
    );

    let markup = format!(
        r#"    <rect width="{width}" height="{height}" fill="{}"/>
    <g filter="url(#bg-soften)">
{blob_layers}    </g>"#,
        rgb_to_hex(base),
        blob_layers = blob_layers
    );

    (defs, markup)
}

fn accent_from_dominant(color: Rgb) -> Rgb {
    let (h, s, mut l) = rgb_to_hsl(color);
    if s < 0.08 {
        return FALLBACK_ACCENT;
    }
    l = l.clamp(0.45, 0.72);
    hsl_to_rgb(h, s.max(0.35), l)
}

fn darkest_color(colors: &[Rgb]) -> Rgb {
    colors
        .iter()
        .copied()
        .min_by(|a, b| {
            relative_luminance(*a)
                .partial_cmp(&relative_luminance(*b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap_or(Rgb {
            r: 18,
            g: 17,
            b: 22,
        })
}

struct DetRng {
    state: u64,
}

impl DetRng {
    fn from_bytes(bytes: &[u8]) -> Self {
        let mut hash = 0xcbf29ce484222325u64;
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        Self { state: hash }
    }

    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    fn range(&mut self, min: f64, max: f64) -> f64 {
        let unit = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        min + unit * (max - min)
    }
}

pub fn rgb_to_hex(color: Rgb) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b)
}

pub fn relative_luminance(color: Rgb) -> f64 {
    let r = srgb_channel_to_linear(color.r);
    let g = srgb_channel_to_linear(color.g);
    let b = srgb_channel_to_linear(color.b);
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

fn srgb_channel_to_linear(channel: u8) -> f64 {
    let c = f64::from(channel) / 255.0;
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

fn color_distance(a: &Rgb, b: &Rgb) -> f64 {
    let ar = f64::from(a.r) / 255.0;
    let ag = f64::from(a.g) / 255.0;
    let ab = f64::from(a.b) / 255.0;
    let br = f64::from(b.r) / 255.0;
    let bg = f64::from(b.g) / 255.0;
    let bb = f64::from(b.b) / 255.0;
    ((ar - br).powi(2) + (ag - bg).powi(2) + (ab - bb).powi(2)).sqrt()
}

fn is_neutral_background_pixel(color: Rgb) -> bool {
    let lum = relative_luminance(color);
    lum > 0.92 || lum < 0.03
}

fn rgb_to_hsl(color: Rgb) -> (f64, f64, f64) {
    let r = f64::from(color.r) / 255.0;
    let g = f64::from(color.g) / 255.0;
    let b = f64::from(color.b) / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let l = (max + min) / 2.0;
    if delta == 0.0 {
        return (0.0, 0.0, l);
    }

    let s = if l < 0.5 {
        delta / (max + min)
    } else {
        delta / (2.0 - max - min)
    };

    let h = if max == r {
        ((g - b) / delta + if g < b { 6.0 } else { 0.0 }) / 6.0
    } else if max == g {
        ((b - r) / delta + 2.0) / 6.0
    } else {
        ((r - g) / delta + 4.0) / 6.0
    };

    (h, s, l)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> Rgb {
    if s == 0.0 {
        let v = (l * 255.0).round() as u8;
        return Rgb { r: v, g: v, b: v };
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    Rgb {
        r: (hue_to_rgb(p, q, h + 1.0 / 3.0) * 255.0).round() as u8,
        g: (hue_to_rgb(p, q, h) * 255.0).round() as u8,
        b: (hue_to_rgb(p, q, h - 1.0 / 3.0) * 255.0).round() as u8,
    }
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    #[test]
    fn darken_for_legibility_caps_luminance() {
        let white = Rgb {
            r: 255,
            g: 255,
            b: 255,
        };
        let dark = darken_for_legibility(white, MAX_BG_LUMINANCE);
        assert!(relative_luminance(dark) <= MAX_BG_LUMINANCE + 0.01);
    }

    #[test]
    fn dominant_colors_picks_distinct_regions() {
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(32, 32);
        for (x, y, pixel) in img.enumerate_pixels_mut() {
            *pixel = if x < 11 {
                Rgba([220, 40, 40, 255])
            } else if x < 22 {
                Rgba([40, 180, 80, 255])
            } else {
                Rgba([40, 80, 220, 255])
            };
            let _ = y;
        }
        let mut bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();

        let colors = dominant_colors_from_image(&bytes, 3).expect("palette");
        assert_eq!(colors.len(), 3);
    }

    #[test]
    fn svg_background_falls_back_without_artwork() {
        let (defs, markup) = svg_background_from_artwork(None, 720, 220);
        assert!(defs.is_empty());
        assert!(markup.contains(FALLBACK_BG));
    }

    #[test]
    fn svg_background_uses_mesh_with_artwork() {
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(16, 16);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([180, 60, 200, 255]);
        }
        let mut bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();

        let (defs, markup) = svg_background_from_artwork(Some(&bytes), 720, 220);
        assert!(defs.contains("bg-blob-0"));
        assert!(defs.contains("bg-soften"));
        assert!(markup.contains("filter=\"url(#bg-soften)\""));
    }

    #[test]
    fn accent_color_uses_dominant_hue() {
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(16, 16);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([220, 40, 40, 255]);
        }
        let mut bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();

        let (_, _, accent) = svg_theme_from_artwork(Some(&bytes), 720, 220);
        assert_ne!(accent, rgb_to_hex(FALLBACK_ACCENT));
        assert!(accent.starts_with('#'));
    }

    #[test]
    fn svg_background_is_deterministic_for_same_artwork() {
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(8, 8);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([90, 120, 200, 255]);
        }
        let mut bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .unwrap();

        let first = svg_background_from_artwork(Some(&bytes), 720, 220);
        let second = svg_background_from_artwork(Some(&bytes), 720, 220);
        assert_eq!(first, second);
    }
}
