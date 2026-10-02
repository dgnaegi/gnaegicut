#!/bin/sh
# Downloads the multilingual whisper.cpp base model (~142 MB) used for auto-captions.
set -e
cd "$(dirname "$0")/.."
mkdir -p models
curl -L --fail -o models/ggml-base.bin \
  https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin
