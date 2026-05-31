#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if ! command -v cargo-packager >/dev/null 2>&1; then
  echo "Installing cargo-packager..."
  cargo install cargo-packager --locked
fi

cargo packager --release -p mac-agent

echo
echo "Built app bundle: target/release/Now Playing.app"
