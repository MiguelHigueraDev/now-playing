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
| `apps/mac-agent` | macOS polling agent that sends updates on change |

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

Start the macOS agent in another terminal (with Apple Music playing):

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
| `POLL_INTERVAL_SECS` | mac-agent | `3` | Poll interval (2–5 seconds) |
| `RUST_LOG` | both | `info` | Tracing filter |

## Project layout

```
now-playing/
├── Cargo.toml              # workspace root
├── apps/
│   ├── api/                # Axum backend
│   └── mac-agent/          # macOS music tracker
├── crates/
│   ├── shared-types/       # shared DTOs
│   └── music-provider/     # Apple Music integration
├── .env.example
└── README.md
```

## Next steps

- ~~Add an /api/image endpoint to render an image with the data from /api/now-playing~~ (`GET /api/now-playing/image`)
- Package `mac-agent` as a menu bar daemon
- Persist state with SQLx + SQLite
- Add Redis for pub/sub and caching
- Add Spotify and other providers via `MusicProvider`

## License

MIT
