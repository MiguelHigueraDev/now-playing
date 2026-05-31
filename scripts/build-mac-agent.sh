#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DERIVED="$ROOT/build/xcode"
APP_SRC="$DERIVED/Build/Products/Release/Now Playing.app"
APP_DST="$ROOT/target/release/Now Playing.app"

xcodebuild \
  -project apps/NowPlaying/NowPlaying.xcodeproj \
  -scheme NowPlaying \
  -configuration Release \
  -derivedDataPath "$DERIVED" \
  CODE_SIGN_IDENTITY="${CODE_SIGN_IDENTITY:--}" \
  CODE_SIGNING_ALLOWED="${CODE_SIGNING_ALLOWED:-NO}" \
  build

mkdir -p "$(dirname "$APP_DST")"
rm -rf "$APP_DST"
cp -R "$APP_SRC" "$APP_DST"

echo
echo "Built app bundle: $APP_DST"
