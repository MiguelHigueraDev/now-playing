# now-playing

Track what you're listening to in Apple Music on macOS and expose it in real time through a small Rust API.

## Architecture

```
mac-agent (polls Apple Music)
        │
        ▼ POST /api/now-playing
      api (Axum)
        │
        ▼ GET /api/now-playing
   website / widgets
```

| Crate / App | Role |
|---|---|
| `crates/shared-types` | Shared DTOs (`NowPlaying`, request/response types) |
| `crates/music-provider` | `MusicProvider` trait + Apple Music via AppleScript |
| `apps/api` | Axum backend storing latest playback state in memory |
| `apps/mac-agent` | macOS menu bar agent that sends updates on change |

## Prerequisites

- [Rust](https://rustup.rs/) (stable)
- macOS with Apple Music installed (for the agent)
- Apple Music must be allowed to respond to AppleScript (System Settings → Privacy & Security → Automation)

## Setup

1. Clone the repo and enter the workspace:

```bash
cd now-playing
```

2. Copy environment variables:

```bash
cp .env.example .env
```

3. Build the workspace:

```bash
cargo build
```

## Running

Start the API in one terminal:

```bash
cargo run -p api
```

### Menu bar app (recommended)

Build the macOS app bundle:

```bash
./scripts/build-mac-agent.sh
```

Install and launch:

1. Open `target/release/Now Playing.app`
2. Drag **Now Playing** to `/Applications`
3. Launch the app from Applications
4. Grant **Automation** access for Apple Music when prompted (System Settings → Privacy & Security → Automation)
5. Click the menu bar icon → **Preferences…** and set:
   - API Base URL (default `http://localhost:3000`)
   - Auth Token (same value as `NOW_PLAYING_TOKEN` in your API `.env`)
   - Poll interval (2–5 seconds)
6. Optional: enable **Enable at Login** from the menu bar menu

Config is stored at `~/Library/Application Support/Now Playing/config.toml`. Logs are written to `~/Library/Application Support/Now Playing/logs/agent.log`.

The app runs as a menu bar agent (no Dock icon). Use **Quit** from the menu to stop it.

### CLI mode (development)

For local debugging without the menu bar shell, run the agent from a terminal with a `.env` file:

```bash
cargo run -p mac-agent
```

The agent polls every 2–5 seconds (default: 3) and only POSTs when the track or play/pause state changes.

## API

### Health check

```bash
curl http://localhost:3000/health
# {"ok":true}
```

### Get current track

```bash
curl http://localhost:3000/api/now-playing
```

### Get now-playing widget (SVG)

Renders album art, track/artist/album text, and a seek bar. Progress is extrapolated from `listened_at`, `position_seconds`, and the current time while the track is playing.

```bash
curl http://localhost:3000/api/now-playing/image -o now-playing.svg
```

Open the file in a browser, or embed it: `<img src="http://localhost:3000/api/now-playing/image" alt="Now playing" />`.

### Update current track (agent only)

```bash
curl -X POST http://localhost:3000/api/now-playing \
  -H "Authorization: Bearer secret-token" \
  -H "Content-Type: application/json" \
  -d '{
    "track_name": "Song Name",
    "artist_name": "Artist",
    "album_name": "Album",
    "is_playing": true
  }'
```

## Environment variables

| Variable | Used by | Default | Description |
|---|---|---|---|
| `NOW_PLAYING_TOKEN` | api, mac-agent | — | Bearer token for POST auth |
| `API_HOST` | api | `0.0.0.0` | Bind host |
| `API_PORT` | api | `3000` | Bind port |
| `API_BASE_URL` | mac-agent | `http://localhost:3000` | API base URL |
| `POLL_INTERVAL_SECS` | mac-agent (CLI) | `3` | Poll interval (2–5 seconds) |
| `RUST_LOG` | both | `info` | Tracing filter |

The packaged menu bar app stores `api_base_url`, `auth_token`, and `poll_interval_secs` in `~/Library/Application Support/Now Playing/config.toml` instead of using environment variables.

## Project layout

```
now-playing/
├── Cargo.toml              # workspace root
├── apps/
│   ├── api/                # Axum backend
│   └── mac-agent/          # macOS menu bar music tracker
├── scripts/
│   └── build-mac-agent.sh  # build Now Playing.app
├── crates/
│   ├── shared-types/       # shared DTOs
│   └── music-provider/     # Apple Music integration
├── .env.example
└── README.md
```

## Next steps

- ~~Add an /api/image endpoint to render an image with the data from /api/now-playing~~ (`GET /api/now-playing/image`)
- ~~Package `mac-agent` as a menu bar daemon~~ (menu bar app with Preferences and Login Item)
- Persist state with SQLx + SQLite
- Add Redis for pub/sub and caching
- Add Spotify and other providers via `MusicProvider`

## License

MIT
