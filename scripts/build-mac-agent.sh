#!/usr/bin/env bash
# Build the macOS menu bar agent with xcodebuild.
# CODE_SIGNING_ALLOWED defaults to NO (unsigned bundle). Set CODE_SIGNING_ALLOWED=YES
# when downstream consumers require a signed app bundle.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PRODUCT_NAME="${PRODUCT_NAME:-Now Playing}"
DERIVED="$ROOT/build/xcode"
APP_SRC="$DERIVED/Build/Products/Release/${PRODUCT_NAME}.app"
APP_DST="$ROOT/target/release/${PRODUCT_NAME}.app"

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
