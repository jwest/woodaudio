#!/bin/bash
set -euo pipefail

# =============================================================================
# woodaudio - build.sh
# All-in-one skrypt do cross-kompilacji i deployu na Raspberry Pi
# Użycie: ./build.sh [komenda]
# =============================================================================

# --- Predefiniowane zmienne (dostosuj do swojej konfiguracji) ---

# Architektura: 'arm64' (domyślnie, 64-bit) lub 'armv7' (32-bit piCore)
ARCH="${ARCH:-arm64}"
BUILD_DIR="/app"
PLATFORM="${PLATFORM:-}"

# Rust — automatyczny wybór targetu na podstawie ARCH
case "$ARCH" in
    arm64)
        TARGET="${TARGET:-aarch64-unknown-linux-gnu}"
        DOCKERFILE="${DOCKERFILE:-Dockerfile.arm64}"
        IMAGE_NAME="${IMAGE_NAME:-woodaudio-builder:arm64}"
        ;;
    armv7)
        TARGET="${TARGET:-armv7-unknown-linux-gnueabihf}"
        DOCKERFILE="${DOCKERFILE:-Dockerfile.armv7}"
        IMAGE_NAME="${IMAGE_NAME:-woodaudio-builder:armv7}"
        ;;
    *)
        echo "ERROR: Nieznana architektura ARCH='${ARCH}'. Dozwolone: arm64, armv7."
        exit 1
        ;;
esac

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
BINARY_PATH="${SCRIPT_DIR}/target/${TARGET}/release/${BINARY_NAME}"

# --- Pomoc ---

print_usage() {
    cat <<EOF
woodaudio - build tool

Usage: $0 <command> [options]

Commands:
  image           Build/rebuild Docker builder image
  build           Build binary (auto-builds image if missing)
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
  ARCH            Target architecture: arm64 (default) or armv7
  FEATURES, TARGET, BINARY_NAME
  DOCKERFILE, IMAGE_NAME
  PI_HOST, PI_USER, PI_DEST_RPIOS, PI_DEST_PICORE

Examples:
  ./build.sh build
  ARCH=armv7 ./build.sh dist
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

    echo "==> Kompilacja ${BINARY_NAME} dla ${TARGET} (features: ${features})..."

    docker run --rm ${PLATFORM:+--platform "$PLATFORM"} \
        -v "${SCRIPT_DIR}:${BUILD_DIR}" \
        "${IMAGE_NAME}" \
        cargo build --release --target "${TARGET}" --features "${features}"

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

    docker run --rm -i -e "TARGET=${TARGET}" -v "${SCRIPT_DIR}:/app" "${IMAGE_NAME}" bash << 'INNERSCRIPT'
set -euo pipefail

DIST="/app/dist"
LIBS="${DIST}/libs"
BINARY="${DIST}/woodaudio-player"
TARGET="${TARGET:-aarch64-unknown-linux-gnu}"

echo "  [1/3] Kopiowanie binary..."
cp "/app/target/${TARGET}/release/woodaudio-player" "${BINARY}"
chmod 755 "${BINARY}"
echo "       Binary: $(basename $BINARY) ($(du -h "${BINARY}" | cut -f1))"

echo "  [2/3] Zbieranie zależności bibliotek współdzielonych..."

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

case "$TARGET" in
    *armv7*) READELF="arm-linux-gnueabihf-readelf" ;;
    *)        READELF="readelf" ;;
esac

SEARCH_DIRS="\
/usr/lib/arm-linux-gnueabihf \
/lib/arm-linux-gnueabihf \
/usr/arm-linux-gnueabihf/lib \
/usr/lib \
/lib \
/opt/vc/lib"

is_ignored() {
    local lib="$1"
    if echo "$lib" | grep -qE "$SYSTEM_LIBS"; then return 0; fi
    if echo "$lib" | grep -qE "$PICORE_LIBS"; then return 0; fi
    return 1
}

find_lib() {
    local soname="$1"
    for dir in $SEARCH_DIRS; do
        for f in "${dir}/${soname}" "${dir}/${soname%.so*}.so"; do
            [ -f "$f" ] && echo "$f" && return 0
        done
    done
    return 1
}

bundle_lib() {
    local soname="$1"
    local found="$2"
    local real="$3"
    local name="$4"
    local dest="${LIBS}/${name}"
    [ -f "$dest" ] || cp -f "$real" "$dest" 2>/dev/null || true
    if [ "$soname" != "$name" ] && [ ! -e "${LIBS}/${soname}" ]; then
        ln -sf "$name" "${LIBS}/${soname}" 2>/dev/null || true
    fi
}

echo "       === Rekurencyjne zbieranie zależności ==="

# Kolejka: pliki do sprawdzenia (sciezki bezwzgledne)
QUEUE_FILE="/tmp/queue.txt"
DONE_FILE="/tmp/done.txt"
> "$QUEUE_FILE"
> "$DONE_FILE"

# Dodaj binarke
echo "$BINARY" >> "$QUEUE_FILE"

while [ -s "$QUEUE_FILE" ]; do
    # Pobierz pierwszy element
    CURRENT=$(head -1 "$QUEUE_FILE")
    sed -i '1d' "$QUEUE_FILE"

    # Pomin juz sprawdzone
    grep -qxF "$CURRENT" "$DONE_FILE" 2>/dev/null && continue
    echo "$CURRENT" >> "$DONE_FILE"

    # Odczytaj NEEDED
    ${READELF} -d "$CURRENT" 2>/dev/null | grep "NEEDED" | awk '{print $5}' | tr -d '[]' | while read -r SONAME; do
        [ -z "$SONAME" ] && continue
        if is_ignored "$SONAME"; then
            case "$CURRENT" in
                "$BINARY") echo "       (pomijam) $SONAME" ;;
            esac
            continue
        fi
        FOUND=$(find_lib "$SONAME")
        if [ -n "$FOUND" ]; then
            REAL=$(readlink -f "$FOUND" 2>/dev/null || echo "$FOUND")
            NAME=$(basename "$REAL")
            # Jesli jeszcze nie dodany
            if [ ! -f "${LIBS}/${NAME}" ]; then
                bundle_lib "$SONAME" "$FOUND" "$REAL" "$NAME"
                echo "       (dodano) $SONAME -> ${NAME}"
                # Dodaj do kolejki (tylko jesli to nie jest symlink-dangling)
                if [ -f "$REAL" ]; then
                    echo "$REAL" >> "$QUEUE_FILE"
                fi
            fi
        else
            echo "       (BRAK!) $SONAME (wymagany przez $(basename "$CURRENT"))"
        fi
    done
done
rm -f "$QUEUE_FILE" "$DONE_FILE"
echo "       === Koniec ==="

echo "  [3/3] Weryfikacja..."
echo "       Liczba bibliotek w libs/: $(ls -1 "${LIBS}" | wc -l)"
echo ""
INNERSCRIPT

    echo "==> Tworzenie skryptów uruchomieniowych..."

    cat > "${DIST_DIR}/run.sh" << 'RUNEOF'
#!/bin/sh
DIR="$(cd "$(dirname "$0")" && pwd)"
export LD_LIBRARY_PATH="${DIR}/libs"

for loader in /lib/ld-linux-armhf.so.3 /lib/ld-linux.so.3 /lib/ld-linux-aarch64.so.1 /lib/ld-linux.so.2; do
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
