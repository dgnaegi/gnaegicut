#!/bin/sh
# One-time setup on macOS: toolchain, ffmpeg (with subtitle filters), whisper.cpp, caption model.
set -e
cd "$(dirname "$0")/.."
command -v brew >/dev/null || { echo "Install Homebrew first: https://brew.sh"; exit 1; }
brew install rustup ffmpeg-full whisper-cpp
rustup default stable
[ -f models/ggml-base.bin ] || sh scripts/get-model.sh
echo "Done. Run: PATH=\"\$(brew --prefix rustup)/bin:\$PATH\" cargo run --release"
