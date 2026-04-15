#!/usr/bin/env bash
# Record a demo clip of vitals-cosmic and encode it into assets/demo.gif.
#
# Usage:  scripts/record-demo.sh [duration-seconds]
# Default duration is 12s. You pick the region when gpu-screen-recorder
# prompts via the xdg-desktop-portal picker.
#
# Dependencies (Gentoo):
#   sudo emerge --ask media-video/gpu-screen-recorder media-video/ffmpeg
#
# Note: media-video/gpu-screen-recorder lives in the 'guru' overlay.
# If you don't have guru enabled yet:
#   sudo eselect repository enable guru && sudo emaint sync -r guru

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

need gpu-screen-recorder media-video/gpu-screen-recorder
need ffmpeg              media-video/ffmpeg

WORK="$(mktemp -d)"
trap 'rm -rf "$WORK"' EXIT

MP4="$WORK/capture.mp4"
PALETTE="$WORK/palette.png"
mkdir -p "$(dirname "$OUT_GIF")"

echo ">> Pick the region in the portal dialog. Recording ${DURATION}s..."
gpu-screen-recorder \
    -w portal \
    -f 60 \
    -c mp4 \
    -o "$MP4" &
REC_PID=$!
sleep "$DURATION"
kill -INT "$REC_PID" 2>/dev/null || true
wait "$REC_PID" 2>/dev/null || true

if [[ ! -s "$MP4" ]]; then
    echo "error: capture file is empty; did you cancel the portal picker?" >&2
    exit 1
fi

# Two-pass gif encoding: generate an adaptive palette, then use it with
# Bayer dithering. This matches gifski-grade quality using only ffmpeg.
echo ">> Generating adaptive palette..."
ffmpeg -loglevel error -y -i "$MP4" \
    -vf "fps=${FPS},scale=${WIDTH}:-1:flags=lanczos,palettegen=stats_mode=diff" \
    "$PALETTE"

echo ">> Encoding $OUT_GIF..."
ffmpeg -loglevel error -y -i "$MP4" -i "$PALETTE" \
    -filter_complex "fps=${FPS},scale=${WIDTH}:-1:flags=lanczos[x];[x][1:v]paletteuse=dither=bayer:bayer_scale=5:diff_mode=rectangle" \
    "$OUT_GIF"

SIZE_BYTES=$(stat -c '%s' "$OUT_GIF")
SIZE_MB=$(awk "BEGIN{printf \"%.2f\", $SIZE_BYTES/1024/1024}")
echo ">> Done: $OUT_GIF (${SIZE_MB} MB)"

if (( SIZE_BYTES > 5 * 1024 * 1024 )); then
    echo "warning: gif is over 5 MB -- consider a shorter duration" >&2
fi
