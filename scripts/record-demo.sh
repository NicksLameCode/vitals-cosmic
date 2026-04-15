#!/usr/bin/env bash
# Record a demo clip of vitals-cosmic and encode it into assets/demo.gif.
#
# Usage:  scripts/record-demo.sh [duration-seconds]
# Default duration is 12s. You pick the region when gpu-screen-recorder
# prompts via the xdg-desktop-portal picker.
#
# Dependencies (Gentoo):
#   sudo emerge --ask gui-apps/gpu-screen-recorder media-video/ffmpeg media-gfx/gifski

set -euo pipefail

DURATION="${1:-12}"
REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT_GIF="$REPO_ROOT/assets/demo.gif"
WIDTH=1280
FPS=15

need() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "error: missing '$1' on PATH" >&2
        echo "install with: sudo emerge --ask $2" >&2
        exit 1
    }
}

need gpu-screen-recorder gui-apps/gpu-screen-recorder
need ffmpeg                media-video/ffmpeg
need gifski                media-gfx/gifski

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

MP4="$WORK/capture.mp4"
FRAMES="$WORK/frames"
mkdir -p "$FRAMES"

echo ">> Pick the region in the portal dialog. Recording ${DURATION}s..."
gpu-screen-recorder \
    -w portal \
    -f 60 \
    -c mp4 \
    -a default_output \
    -o "$MP4" &
REC_PID=$!
sleep "$DURATION"
kill -INT "$REC_PID" 2>/dev/null || true
wait "$REC_PID" 2>/dev/null || true

if [[ ! -s "$MP4" ]]; then
    echo "error: capture file is empty; did you cancel the portal picker?" >&2
    exit 1
fi

echo ">> Extracting frames at ${FPS} fps, width ${WIDTH}px..."
ffmpeg -loglevel error -y -i "$MP4" \
    -vf "fps=${FPS},scale=${WIDTH}:-1:flags=lanczos" \
    "$FRAMES/%04d.png"

echo ">> Encoding to $OUT_GIF with gifski..."
mkdir -p "$(dirname "$OUT_GIF")"
gifski \
    --fps "$FPS" \
    --width "$WIDTH" \
    --quality 90 \
    --output "$OUT_GIF" \
    "$FRAMES"/*.png

SIZE_BYTES=$(stat -c '%s' "$OUT_GIF")
SIZE_MB=$(awk "BEGIN{printf \"%.2f\", $SIZE_BYTES/1024/1024}")
echo ">> Done: $OUT_GIF (${SIZE_MB} MB)"

if (( SIZE_BYTES > 5 * 1024 * 1024 )); then
    echo "warning: gif is over 5 MB -- consider a shorter duration or lower --quality" >&2
fi
