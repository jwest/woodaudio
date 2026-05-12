#!/bin/bash
set -euo pipefail

# =============================================================================
# woodaudio - build.sh
# All-in-one skrypt do cross-kompilacji i deployu na Raspberry Pi
# Użycie: ./build.sh [komenda]
# =============================================================================

# --- Predefiniowane zmienne (dostosuj do swojej konfiguracji) ---

# Docker
IMAGE_NAME="${IMAGE_NAME:-woodaudio-builder:arm64}"
DOCKERFILE="${DOCKERFILE:-Dockerfile.arm64}"
BUILD_DIR="/app"
PLATFORM="${PLATFORM:-}"

# Rust
TARGET="${TARGET:-aarch64-unknown-linux-gnu}"
FEATURES="${FEATURES:-gui,gpio}"
BINARY_NAME="${BINARY_NAME:-woodaudio-player}"

# Raspberry Pi — adres i użytkownik
PI_HOST="${PI_HOST:-192.168.1.100}"
PI_USER="${PI_USER:-pi}"
PI_DEST_RPIOS="${PI_DEST_RPIOS:-/home/pi}"
PI_DEST_PICORE="${PI_DEST_PICORE:-/opt/woodaudio}"

# Runtime dependencies dla Raspberry Pi OS (apt)
RPIOS_DEPS="${RPIOS_DEPS:-libdrm2 libgbm1 libinput10 libudev1 libasound2 libxkbcommon0 libssl3}"

# Ścieżki lokalne
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DIST_DIR="${SCRIPT_DIR}/dist"
BINARY_PATH="${SCRIPT_DIR}/target/release/${BINARY_NAME}"

# --- Pomoc ---

print_usage() {
    cat <<EOF
woodaudio - build tool

Usage: $0 <command> [options]

Commands:
  image           Build/rebuild Docker builder image
  build           Build binary for aarch64 (auto-builds image if missing)
  dist            Build + bundle binary with .so libraries for piCore
  deploy-rpios    Build + copy binary to Pi + install runtime deps
  deploy-picore   Build + dist + copy everything to Pi (piCore)
  clean           Remove target/ and dist/
  help            Show this help

Options (for build/dist):
  --features F   Features to enable (default: gui,gpio)
  --image NAME   Docker image name
  --rebuild      Force rebuild of Docker image
  --copy-dest P  Copy binary to PATH after build

Environment variables:
  IMAGE_NAME, DOCKERFILE, FEATURES, TARGET, BINARY_NAME
  PI_HOST, PI_USER, PI_DEST_RPIOS, PI_DEST_PICORE

Examples:
  ./build.sh build
  ./build.sh dist
  PI_HOST=192.168.1.42 ./build.sh deploy-rpios
  ./build.sh clean
EOF
}

# --- Funkcje pomocnicze ---

ensure_image() {
    if ! docker image inspect "${IMAGE_NAME}" &>/dev/null; then
        echo "==> Budowanie obrazu Docker ${IMAGE_NAME}..."
        docker build -f "${SCRIPT_DIR}/${DOCKERFILE}" -t "${IMAGE_NAME}" "${SCRIPT_DIR}"
    fi
}

# --- Funkcje główne ---

do_image() {
    echo "==> Budowanie obrazu Docker ${IMAGE_NAME}..."
    docker build -f "${SCRIPT_DIR}/${DOCKERFILE}" -t "${IMAGE_NAME}" "${SCRIPT_DIR}"
    echo "==> Gotowe!"
}

do_build() {
    local features="$1"
    local copy_dest="$2"
    local rebuild="$3"

    if $rebuild; then
        do_image
    else
        ensure_image
    fi

    echo "==> Kompilacja ${BINARY_NAME} dla aarch64 (features: ${features})..."

    docker run --rm ${PLATFORM:+--platform "$PLATFORM"} \
        -v "${SCRIPT_DIR}:${BUILD_DIR}" \
        "${IMAGE_NAME}" \
        cargo build --release --features "${features}"

    echo ""
    echo "==> Kompilacja zakończona!"
    echo "    Binary: ${BINARY_PATH}"
    echo "    Rozmiar: $(du -h "${BINARY_PATH}" | cut -f1)"

    if [ -n "$copy_dest" ]; then
        echo "==> Kopiowanie binary do ${copy_dest}..."
        cp "${BINARY_PATH}" "$copy_dest"
        echo "    Gotowe."
    fi
}

do_dist() {
    local features="$1"
    local rebuild="$2"

    do_build "$features" "" "$rebuild"

    echo "==> Przygotowywanie katalogu dist/..."
    rm -rf "${DIST_DIR}"
    mkdir -p "${DIST_DIR}/libs"

    echo "==> Eksportowanie binary i bibliotek z kontenera Docker..."

    docker run --rm -i -v "${SCRIPT_DIR}:/app" "${IMAGE_NAME}" bash << 'INNERSCRIPT'
set -euo pipefail

DIST="/app/dist"
LIBS="${DIST}/libs"
BINARY="${DIST}/woodaudio-player"

echo "  [1/3] Kopiowanie binary..."
cp "/app/target/release/woodaudio-player" "${BINARY}"
chmod 755 "${BINARY}"
echo "       Binary: $(basename $BINARY) ($(du -h "${BINARY}" | cut -f1))"

echo "  [2/3] Zbieranie zależności bibliotek współdzielonych..."
rm -f /tmp/lib_list.txt
touch /tmp/lib_list.txt

# Biblioteki systemowe — NIGDY nie pakujemy
SYSTEM_LIBS="\
libc.so\
|libm.so\
|libpthread.so\
|librt.so\
|libdl.so\
|libgcc_s.so\
|ld-linux"

# Biblioteki dostarczane przez piCore jako tcz
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
        echo "       (pomijam systemową) $BNAME"
        continue
    fi
    if echo "$BNAME" | grep -qE "$PICORE_LIBS"; then
        echo "       (pomijam piCore-provided) $BNAME"
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

echo "  [3/3] Weryfikacja..."
echo "       Liczba bibliotek w libs/: $(ls -1 "${LIBS}" | wc -l)"
echo ""

echo "       Sprawdzanie czy wszystkie zależności są spełnione..."
UNSATISFIED=$(LD_LIBRARY_PATH="${LIBS}:/lib:/usr/lib" ldd "${BINARY}" 2>/dev/null | grep "not found" || true)
if [ -n "$UNSATISFIED" ]; then
    echo "       UWAGA: Brakujące biblioteki:"
    echo "$UNSATISFIED" | sed "s/^/       /"
else
    echo "       Wszystkie biblioteki OK!"
fi
INNERSCRIPT

    echo "==> Tworzenie skryptów uruchomieniowych..."

    cat > "${DIST_DIR}/run.sh" << 'RUNEOF'
#!/bin/sh
DIR="$(cd "$(dirname "$0")" && pwd)"
export LD_LIBRARY_PATH="${DIR}/libs"

for loader in /lib/ld-linux-aarch64.so.1 /lib/ld-linux.so.3 /lib/ld-linux.so.2 /lib/ld-linux-arm64.so.1; do
    if [ -f "$loader" ]; then
        exec "$loader" "${DIR}/woodaudio-player" "$@" >> /tmp/woodaudio.log 2>&1
    fi
done

echo "ERROR: No dynamic linker found!"
exit 1
RUNEOF
    chmod 755 "${DIST_DIR}/run.sh"

    cat > "${DIST_DIR}/start-woodaudio.sh" << 'STARTEOF'
#!/bin/sh
DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$DIR"
exec ./run.sh "$@"
STARTEOF
    chmod 755 "${DIST_DIR}/start-woodaudio.sh"

    echo ""
    echo "==> Katalog dist/ utworzony: ${DIST_DIR}"
    echo ""
    echo "    Zawartość:"
    du -sh "${DIST_DIR}/woodaudio-player" "${DIST_DIR}/libs" 2>/dev/null || true
    echo "    libs/ zawiera $(ls -1 "${DIST_DIR}/libs" | wc -l) plików"
    echo ""
    echo "==> Deploy na piCore:"
    echo "    scp -r dist/* ${PI_USER}@${PI_HOST}:${PI_DEST_PICORE}/"
    echo "    ssh ${PI_USER}@${PI_HOST} ${PI_DEST_PICORE}/run.sh"
}

do_deploy_rpios() {
    local features="$1"
    local rebuild="$2"

    do_build "$features" "" "$rebuild"

    echo "==> Deploy na Raspberry Pi OS (${PI_USER}@${PI_HOST})..."
    echo "  [1/3] Kopiowanie binary..."
    scp "${BINARY_PATH}" "${PI_USER}@${PI_HOST}:${PI_DEST_RPIOS}/"

    echo "  [2/3] Instalowanie zależności runtime..."
    ssh "${PI_USER}@${PI_HOST}" "sudo apt-get update && sudo apt-get install -y ${RPIOS_DEPS}"

    echo "  [3/3] Gotowe!"
    echo ""
    echo "  Uruchomienie na Pi:"
    echo "    ssh ${PI_USER}@${PI_HOST}"
    echo "    ${PI_DEST_RPIOS}/woodaudio-player --config /path/to/config.ini"
}

do_deploy_picore() {
    local features="$1"
    local rebuild="$2"

    do_dist "$features" "$rebuild"

    echo "==> Deploy na piCore (${PI_USER}@${PI_HOST})..."
    echo "  [1/3] Tworzenie katalogu ${PI_DEST_PICORE} na zdalnym hoście..."
    ssh "${PI_USER}@${PI_HOST}" "mkdir -p ${PI_DEST_PICORE}"

    echo "  [2/3] Kopiowanie dist/ na ${PI_HOST}:${PI_DEST_PICORE}/..."
    scp -r "${DIST_DIR}/." "${PI_USER}@${PI_HOST}:${PI_DEST_PICORE}/"

    echo "  [3/3] Gotowe!"
    echo ""
    echo "  Uruchomienie na Pi:"
    echo "    ssh ${PI_USER}@${PI_HOST}"
    echo "    ${PI_DEST_PICORE}/run.sh"
}

do_clean() {
    echo "==> Czyszczenie target/ i dist/..."
    rm -rf "${SCRIPT_DIR}/target" "${DIST_DIR}"
    echo "    Gotowe."
}

# --- Main ---

SUBCOMMAND=""
FEATURES_ARG=""
COPY_DEST=""
REBUILD=false

if [ $# -eq 0 ]; then
    print_usage
    exit 0
fi

SUBCOMMAND="$1"
shift

while [[ $# -gt 0 ]]; do
    case "$1" in
        --features)  FEATURES_ARG="$2"; shift 2 ;;
        --image)     IMAGE_NAME="$2"; shift 2 ;;
        --rebuild)   REBUILD=true; shift ;;
        --copy-dest) COPY_DEST="$2"; shift 2 ;;
        --help)      print_usage; exit 0 ;;
        *)           echo "Nieznana opcja: $1"; print_usage; exit 1 ;;
    esac
done

FEATURES="${FEATURES_ARG:-$FEATURES}"

case "$SUBCOMMAND" in
    image)        do_image ;;
    build)        do_build "$FEATURES" "$COPY_DEST" "$REBUILD" ;;
    dist)         do_dist "$FEATURES" "$REBUILD" ;;
    deploy-rpios) do_deploy_rpios "$FEATURES" "$REBUILD" ;;
    deploy-picore) do_deploy_picore "$FEATURES" "$REBUILD" ;;
    clean)        do_clean ;;
    help|--help|-h) print_usage ;;
    *)            echo "Nieznana komenda: $SUBCOMMAND"; print_usage; exit 1 ;;
esac
