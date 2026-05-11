#!/bin/bash
set -euo pipefail

IMAGE_NAME="${IMAGE_NAME:-woodaudio-builder:arm64}"
DOCKERFILE="${DOCKERFILE:-Dockerfile.arm64}"
BUILD_DIR="/app"
PLATFORM="${PLATFORM:-}"

print_usage() {
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --features FEATURES   Features to enable (default: gui,gpio)"
    echo "  --image NAME          Docker image name (default: woodaudio-builder:arm64)"
    echo "  --dockerfile FILE     Dockerfile to use (default: Dockerfile.arm64)"
    echo "  --rebuild             Force rebuild of Docker image"
    echo "  --copy-dest PATH      Copy binary to PATH after build"
    echo "  --help                Show this help"
    echo ""
    echo "Environment variables:"
    echo "  IMAGE_NAME     Same as --image"
    echo "  DOCKERFILE     Same as --dockerfile"
}

FEATURES="gui,gpio"
COPY_DEST=""
REBUILD=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --features)     FEATURES="$2"; shift 2 ;;
        --image)        IMAGE_NAME="$2"; shift 2 ;;
        --dockerfile)   DOCKERFILE="$2"; shift 2 ;;
        --rebuild)      REBUILD=true; shift ;;
        --copy-dest)    COPY_DEST="$2"; shift 2 ;;
        --help)         print_usage; exit 0 ;;
        *)              echo "Unknown option: $1"; print_usage; exit 1 ;;
    esac
done

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

if $REBUILD; then
    echo "==> Rebuilding Docker image ${IMAGE_NAME}..."
    docker build -f "${SCRIPT_DIR}/${DOCKERFILE}" -t "${IMAGE_NAME}" "${SCRIPT_DIR}"
fi

if ! docker image inspect "${IMAGE_NAME}" &>/dev/null; then
    echo "==> Building Docker image ${IMAGE_NAME}..."
    docker build -f "${SCRIPT_DIR}/${DOCKERFILE}" -t "${IMAGE_NAME}" "${SCRIPT_DIR}"
fi

echo "==> Building woodaudio for aarch64 (features: ${FEATURES})..."
PLATFORM_ARGS=()
if [ -n "$PLATFORM" ]; then
    PLATFORM_ARGS=(--platform "$PLATFORM")
fi
docker run --rm "${PLATFORM_ARGS[@]}" \
    -v "${SCRIPT_DIR}:${BUILD_DIR}" \
    "${IMAGE_NAME}" \
    cargo build --release --features "${FEATURES}"

echo ""
echo "==> Build complete!"
echo "    Binary: ${SCRIPT_DIR}/target/release/woodaudio-player"
echo "    Size:   $(du -h "${SCRIPT_DIR}/target/release/woodaudio-player" | cut -f1)"

if [ -n "$COPY_DEST" ]; then
    echo "==> Copying binary to ${COPY_DEST}..."
    cp "${SCRIPT_DIR}/target/release/woodaudio-player" "$COPY_DEST"
    echo "    Done."
fi

echo ""
echo "==> Next steps:"
echo "    1. Copy binary to Raspberry Pi:"
echo "       scp target/release/woodaudio-player pi@<ip>:~/"
echo "    2. On Pi, install runtime deps:"
echo "       sudo apt-get install libdrm2 libgbm1 libinput10 libudev1 libasound2 libxkbcommon0 libssl3"
echo "    3. Run:"
echo "       ./woodaudio-player --config /path/to/config.ini"
