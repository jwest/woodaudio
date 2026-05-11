#!/bin/bash
set -eo pipefail

IMAGE_NAME="${IMAGE_NAME:-woodaudio-builder:arm64}"
DIST_DIR="$(cd "$(dirname "$0")" && pwd)/dist"
FEATURES="${FEATURES:-gui,gpio}"

echo "==> woodaudio distribution exporter"
echo "    Image:       ${IMAGE_NAME}"
echo "    Dist dir:    ${DIST_DIR}"
echo "    Features:    ${FEATURES}"
echo ""

# Ensure image exists
if ! docker image inspect "${IMAGE_NAME}" &>/dev/null; then
    echo "ERROR: Docker image '${IMAGE_NAME}' not found."
    echo "       Build it first with: docker build -f Dockerfile.arm64 -t ${IMAGE_NAME} ."
    exit 1
fi

# Build if binary doesn't exist yet
if [ ! -f "target/release/woodaudio-player" ]; then
    echo "==> Building woodaudio for aarch64..."
    docker run --rm -v "$(pwd):/app" "${IMAGE_NAME}" \
        cargo build --release --features "${FEATURES}"
fi

# Check the local binary file type
if [ -f "target/release/woodaudio-player" ]; then
    file_type=$(file "target/release/woodaudio-player" 2>/dev/null || echo "unknown")
    echo "==> Local binary: ${file_type}"
fi

echo "==> Preparing dist directory..."
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}/libs"

echo "==> Exporting binary and libraries from Docker container..."

# Pipe the inner script into docker to avoid macOS bash 3.2 quoting issues with -c '...'
docker run --rm -i -v "$(pwd):/app" "${IMAGE_NAME}" bash << 'INNERSCRIPT'
set -euo pipefail

DIST="/app/dist"
LIBS="${DIST}/libs"
BINARY="${DIST}/woodaudio-player"

echo "  [1/3] Copying binary..."
cp "/app/target/release/woodaudio-player" "${BINARY}"
chmod 755 "${BINARY}"
echo "       Binary: $(basename $BINARY) ($(du -h "${BINARY}" | cut -f1))"

echo "  [2/3] Collecting shared library dependencies..."
rm -f /tmp/lib_list.txt
touch /tmp/lib_list.txt

# Core system libraries - NEVER bundle
SYSTEM_LIBS="\
libc.so\
|libm.so\
|libpthread.so\
|librt.so\
|libdl.so\
|libgcc_s.so\
|ld-linux"

# piCore tcz-provided libraries (from user package list)
PICORE_LIBS="\
libdrm\
|libudev\
|libinput\
|libxkbcommon\
|libasound\
|libssl\
|libcrypto\
|libevdev\
|libmtdev\
|libglib\
|libgobject\
|libpcre\
|libffi\
|libexpat"

ldd "${BINARY}" 2>/dev/null | grep "=> /" | awk "{print \$3}" >> /tmp/lib_list.txt

sort -u /tmp/lib_list.txt | while read -r LIB; do
    [ -e "$LIB" ] || continue
    BNAME=$(basename "$LIB")
    if echo "$BNAME" | grep -qE "$SYSTEM_LIBS"; then
        echo "       (skip system lib) $BNAME"
        continue
    fi
    if echo "$BNAME" | grep -qE "$PICORE_LIBS"; then
        echo "       (skip piCore-provided lib) $BNAME"
        continue
    fi
    REAL=$(readlink -f "$LIB" 2>/dev/null || echo "$LIB")
    NAME=$(basename "$REAL")
    DEST="${LIBS}/${NAME}"
    [ -f "$DEST" ] || cp -f "$REAL" "$DEST" 2>/dev/null || true
    BNAME2=$(basename "$LIB")
    if [ "$BNAME2" != "$NAME" ] && [ ! -e "${LIBS}/${BNAME2}" ]; then
        ln -sf "$NAME" "${LIBS}/${BNAME2}" 2>/dev/null || true
    fi
done
rm -f /tmp/lib_list.txt

echo "  [3/3] Verifying..."
echo "       Libraries in libs/: $(ls -1 "${LIBS}" | wc -l)"
echo ""

echo "       Checking library resolution against system..."
UNSATISFIED=$(LD_LIBRARY_PATH="${LIBS}:/lib:/usr/lib" ldd "${BINARY}" 2>/dev/null | grep "not found" || true)
if [ -n "$UNSATISFIED" ]; then
    echo "       WARNING: Some libraries may be missing:"
    echo "$UNSATISFIED" | sed "s/^/       /"
else
    echo "       All libraries resolved OK!"
fi
INNERSCRIPT

echo ""
echo "==> Creating run scripts..."

# Create run.sh for piCore
cat > "${DIST_DIR}/run.sh" << 'RUNEOF'
#!/bin/sh
DIR="$(cd "$(dirname "$0")" && pwd)"
export LD_LIBRARY_PATH="${DIR}/libs"

# Find system dynamic linker
for loader in /lib/ld-linux-aarch64.so.1 /lib/ld-linux.so.3 /lib/ld-linux.so.2 /lib/ld-linux-arm64.so.1; do
    if [ -f "$loader" ]; then
        exec "$loader" "${DIR}/woodaudio-player" "$@"
    fi
done

echo "ERROR: No dynamic linker found!"
exit 1
RUNEOF
chmod 755 "${DIST_DIR}/run.sh"

# Create a convenience startup script for autostart
cat > "${DIST_DIR}/start-woodaudio.sh" << 'STARTEOF'
#!/bin/sh
# For /opt/woodaudio/start-woodaudio.sh
# Can be used in autostart (.profile, /etc/rc.local, or systemd)
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"
exec ./run.sh "$@"
STARTEOF
chmod 755 "${DIST_DIR}/start-woodaudio.sh"

echo ""
echo "==> Dist directory created at: ${DIST_DIR}"
echo ""
echo "    Contents:"
du -sh "${DIST_DIR}/woodaudio-player" "${DIST_DIR}/libs"
echo "    libs/ contains $(ls -1 "${DIST_DIR}/libs" | wc -l) files"
echo ""
echo "==> Deploy to piCore:"
echo "    scp -r dist/* pi@<ip>:/opt/woodaudio/"
echo "    ssh pi@<ip> /opt/woodaudio/run.sh"
echo ""
