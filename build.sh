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
  tcz             Build + create TCZ package (SquashFS) for piCore
  pcp-image       Build TCZ + create piCorePlayer SD image with woodaudio pre-installed
  install-pcp     Install woodaudio.tcz on a running piCorePlayer (via SSH)
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
  PCP_HOST, PCP_USER  piCorePlayer host/user for install-pcp (default: 192.168.1.60, tc)

Examples:
  ./build.sh build
  ARCH=armv7 ./build.sh dist
  ARCH=armv7 ./build.sh tcz
  ./build.sh pcp-image
  ./build.sh install-pcp
  PCP_HOST=192.168.1.42 ./build.sh install-pcp
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
export SLINT_KMS_ROTATION=90

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

do_tcz() {
    local features="$1"
    local rebuild="$2"
    local version

    version=$(grep '^version = ' "${SCRIPT_DIR}/Cargo.toml" | head -1 | sed 's/.*"\(.*\)"/\1/')

    do_build "$features" "" "$rebuild"

    echo "==> Przygotowywanie paczki TCZ..."
    rm -rf "${DIST_DIR}"
    mkdir -p "${DIST_DIR}"

    docker run --rm -i \
        -e "TARGET=${TARGET}" \
        -e "VERSION=${version}" \
        -v "${SCRIPT_DIR}:/app" \
        "${IMAGE_NAME}" \
        bash << 'INNERSCRIPT'
set -euo pipefail

APP="/app"
DIST="${APP}/dist"
TARGET="${TARGET:-armv7-unknown-linux-gnueabihf}"
VERSION="${VERSION:-0.5.0}"
TCZ_ROOT="/tmp/tcz-package"
TCZ_NAME="woodaudio"

# --- Library search ---
SYSTEM_LIBS="\
libc.so\
|libm.so\
|libpthread.so\
|librt.so\
|libdl.so\
|libgcc_s.so\
|ld-linux"

PICORE_LIBS="\
libdrm\
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
    *armv7*) READELF="arm-linux-gnueabihf-readelf"; STRIP="arm-linux-gnueabihf-strip" ;;
    *)        READELF="readelf"; STRIP="strip" ;;
esac

SEARCH_DIRS="\
/usr/lib/arm-linux-gnueabihf \
/lib/arm-linux-gnueabihf \
/usr/arm-linux-gnueabihf/lib \
/usr/lib \
/lib"

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

lib_to_tcz() {
    local lib="$1"
    case "$lib" in
        libasound*)  echo "libasound.tcz" ;;
        libdrm*)     echo "libdrm.tcz" ;;
        libudev*)    echo "udev-lib.tcz" ;;
        libinput*)   echo "libinput.tcz" ;;
        libxkbcommon*) echo "libxkbcommon.tcz" ;;
        libssl*)     echo "openssl.tcz" ;;
        libcrypto*)  echo "openssl.tcz" ;;
        libevdev*)   echo "libevdev.tcz" ;;
        libmtdev*)   echo "mtdev.tcz" ;;
        libglib*)    echo "glib2.tcz" ;;
        libgobject*) echo "glib2.tcz" ;;
        libpcre*)    echo "pcre2.tcz" ;;
        libffi*)     echo "libffi7.tcz" ;;
        libexpat*)   echo "expat2.tcz" ;;
    esac
}

# --- Build TCZ structure ---
rm -rf "$TCZ_ROOT"
mkdir -p "${TCZ_ROOT}/usr/local/bin"
mkdir -p "${TCZ_ROOT}/usr/local/lib"
mkdir -p "${TCZ_ROOT}/usr/local/etc/woodaudio"
mkdir -p "${TCZ_ROOT}/usr/local/tce.installed"

echo "  [1/5] Kopiowanie i stripowanie binary..."
cp "${APP}/target/${TARGET}/release/woodaudio-player" "${TCZ_ROOT}/usr/local/bin/woodaudio-player"
${STRIP} --strip-unneeded "${TCZ_ROOT}/usr/local/bin/woodaudio-player"
echo "       Binary: $(ls -lh "${TCZ_ROOT}/usr/local/bin/woodaudio-player" | awk '{print $5}')"

echo "  [2/5] Zbieranie bibliotek współdzielonych..."

QUEUE_FILE="/tmp/queue.txt"
DONE_FILE="/tmp/done.txt"
EXCLUDED_FILE="/tmp/excluded_libs.txt"
> "$QUEUE_FILE"
> "$DONE_FILE"
> "$EXCLUDED_FILE"

echo "${TCZ_ROOT}/usr/local/bin/woodaudio-player" >> "$QUEUE_FILE"

while [ -s "$QUEUE_FILE" ]; do
    CURRENT=$(head -1 "$QUEUE_FILE")
    sed -i '1d' "$QUEUE_FILE"
    grep -qxF "$CURRENT" "$DONE_FILE" 2>/dev/null && continue
    echo "$CURRENT" >> "$DONE_FILE"

    ${READELF} -d "$CURRENT" 2>/dev/null | grep "NEEDED" | awk '{print $5}' | tr -d '[]' | while read -r SONAME; do
        [ -z "$SONAME" ] && continue
        if is_ignored "$SONAME"; then
            echo "$SONAME" >> "$EXCLUDED_FILE"
            continue
        fi
        FOUND=$(find_lib "$SONAME")
        if [ -n "$FOUND" ]; then
            REAL=$(readlink -f "$FOUND" 2>/dev/null || echo "$FOUND")
            NAME=$(basename "$REAL")
            DEST="${TCZ_ROOT}/usr/local/lib/${NAME}"
            if [ ! -f "$DEST" ]; then
                cp -f "$REAL" "$DEST" 2>/dev/null || true
                if [ "$SONAME" != "$NAME" ] && [ ! -e "${TCZ_ROOT}/usr/local/lib/${SONAME}" ]; then
                    ln -sf "$NAME" "${TCZ_ROOT}/usr/local/lib/${SONAME}" 2>/dev/null || true
                fi
                echo "$REAL" >> "$QUEUE_FILE"
                echo "       (dodano) $SONAME -> ${NAME}"
            fi
        else
            echo "       (BRAK!) $SONAME (wymagany przez $(basename "$CURRENT"))"
        fi
    done
done
rm -f "$QUEUE_FILE" "$DONE_FILE"

echo "       Bibliotek w /usr/local/lib/: $(ls -1 "${TCZ_ROOT}/usr/local/lib" | grep -v '^\.' | wc -l)"

echo "  [3/5] Tworzenie plików konfiguracyjnych..."
cp "${APP}/sample_config.ini" "${TCZ_ROOT}/usr/local/etc/woodaudio/config.ini.sample"

cat > "${TCZ_ROOT}/usr/local/tce.installed/${TCZ_NAME}" << 'TCESCRIPT'
#!/bin/sh
SAMPLE="/usr/local/etc/woodaudio/config.ini.sample"
CONFIG_DIR="/home/tc/.config/woodaudio"
CONFIG_FILE="$CONFIG_DIR/config.ini"
if [ ! -f "$CONFIG_FILE" ]; then
    mkdir -p "$CONFIG_DIR"
    cp "$SAMPLE" "$CONFIG_FILE"
    chmod 666 "$CONFIG_FILE"
fi
TCESCRIPT
chmod 755 "${TCZ_ROOT}/usr/local/tce.installed/${TCZ_NAME}"

echo "  [4/5] Tworzenie plików metadanych..."


cat > "${DIST}/${TCZ_NAME}.tcz.info" << INFOEOF
Title:          ${TCZ_NAME}.tcz
Description:    Tidal HiFi audio player for Raspberry Pi with touch GUI
Version:        ${VERSION}
Author:         jwest
Original-site:  https://github.com/jwest/woodaudio
Copying-policy: MIT
Size:           $(du -sh "${TCZ_ROOT}" | cut -f1)
Extension_by:   woodaudio team
Tags:           audio tidal hifi player raspberry-pi
Comments:       Requires KMS/DRM display and ALSA audio.
                Configure device tree in /boot/config.txt.
                ----
                Usage: woodaudio-player --config /usr/local/etc/woodaudio/config.ini
                ----
                Requires Tidal HiFi subscription.
Change-log:     $(date +%Y-%m-%d) First version, ${VERSION}
Current:        $(date +%Y-%m-%d) Version ${VERSION}
INFOEOF


if [ -s "$EXCLUDED_FILE" ]; then
    sort -u "$EXCLUDED_FILE" | while read -r lib; do
        lib_to_tcz "$lib"
    done | sort -u > "${DIST}/${TCZ_NAME}.tcz.dep"
    echo "       .dep: $(wc -l < "${DIST}/${TCZ_NAME}.tcz.dep") zależności"
else
    > "${DIST}/${TCZ_NAME}.tcz.dep"
    echo "       .dep: brak zależności"
fi
rm -f "$EXCLUDED_FILE"

echo "  [5/5] Tworzenie SquashFS..."
cd /tmp
mksquashfs "${TCZ_ROOT}" "${DIST}/${TCZ_NAME}.tcz" -b 4k -no-xattrs
echo "       TCZ: $(ls -lh "${DIST}/${TCZ_NAME}.tcz" | awk '{print $5}')"

unsquashfs -l "${DIST}/${TCZ_NAME}.tcz" 2>/dev/null | sed 's/squashfs-root//' > "${DIST}/${TCZ_NAME}.tcz.list"

md5sum "${DIST}/${TCZ_NAME}.tcz" | cut -d' ' -f1 > "${DIST}/${TCZ_NAME}.tcz.md5.txt"

rm -rf "$TCZ_ROOT"

echo ""
echo "=== Podsumowanie ==="
echo "  woodaudio.tcz:         $(ls -lh "${DIST}/${TCZ_NAME}.tcz" | awk '{print $5}')"
echo "  woodaudio.tcz.info     $(ls -lh "${DIST}/${TCZ_NAME}.tcz.info" | awk '{print $5}')"
echo "  woodaudio.tcz.dep      $(wc -l < "${DIST}/${TCZ_NAME}.tcz.dep") dependencies"
echo "  woodaudio.tcz.list     $(wc -l < "${DIST}/${TCZ_NAME}.tcz.list") files"
echo "  woodaudio.tcz.md5.txt  generated"
INNERSCRIPT

    echo ""
    echo "==> Gotowe! Paczka TCZ w: ${DIST_DIR}/"
    echo "    Zainstaluj na piCore:"
    echo "      tce-load -i ${DIST_DIR}/woodaudio.tcz"
    echo "      woodaudio-player --config /usr/local/etc/woodaudio/config.ini"
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

do_install_pcp() {
    local features="$1"
    local rebuild="$2"
    local pcp_host="${PCP_HOST:-192.168.1.60}"
    local pcp_user="${PCP_USER:-tc}"
    local tce_base="/mnt/mmcblk0p2/tce"

    # Build TCZ if missing
    if [ ! -f "${DIST_DIR}/woodaudio.tcz" ]; then
        echo "==> Brak woodaudio.tcz, buduję..."
        ARCH=armv7 do_tcz "$features" "$rebuild"
    fi

    echo "==> Instalacja woodaudio.tcz na piCorePlayer (${pcp_user}@${pcp_host})..."

    echo "  [1/5] Kopiowanie woodaudio.tcz do ${tce_base}/optional/..."
    ssh "${pcp_user}@${pcp_host}" "mkdir -p ${tce_base}/optional"
    scp "${DIST_DIR}/woodaudio.tcz" "${pcp_user}@${pcp_host}:${tce_base}/optional/"
    scp "${DIST_DIR}/woodaudio.tcz.dep" "${pcp_user}@${pcp_host}:${tce_base}/optional/"
    scp "${DIST_DIR}/woodaudio.tcz.md5.txt" "${pcp_user}@${pcp_host}:${tce_base}/optional/"

    echo "  [2/5] Rejestracja w onboot.lst..."
    ssh "${pcp_user}@${pcp_host}" "grep -qxF 'woodaudio.tcz' ${tce_base}/onboot.lst 2>/dev/null || echo 'woodaudio.tcz' >> ${tce_base}/onboot.lst"

    echo "  [3/5] Konfiguracja autostartu (bootlocal.sh)..."
    ssh "${pcp_user}@${pcp_host}" "TCE_BASE=${tce_base}" sh -s << 'REMOTESCRIPT'
set -eu
tce_base="${TCE_BASE:-/mnt/mmcblk0p2/tce}"

mkdir -p /tmp/wa-bootlocal
if [ -f "${tce_base}/mydata.tgz" ]; then
    tar xzf "${tce_base}/mydata.tgz" -C /tmp/wa-bootlocal 2>/dev/null || true
fi

mkdir -p /tmp/wa-bootlocal/opt
if [ -f /tmp/wa-bootlocal/opt/bootlocal.sh ]; then
    if grep -q "woodaudio-player" /tmp/wa-bootlocal/opt/bootlocal.sh 2>/dev/null; then
        sed -i 's|.*woodaudio-player.*|SLINT_KMS_ROTATION=90 /usr/local/bin/woodaudio-player --config /home/tc/.config/woodaudio/config.ini \&|' /tmp/wa-bootlocal/opt/bootlocal.sh
    else
        printf '\n# Start woodaudio\nSLINT_KMS_ROTATION=90 /usr/local/bin/woodaudio-player --config /home/tc/.config/woodaudio/config.ini &\n' >> /tmp/wa-bootlocal/opt/bootlocal.sh
    fi
else
    cat > /tmp/wa-bootlocal/opt/bootlocal.sh << 'BOOTLOCAL'
#!/bin/sh
# Start woodaudio
SLINT_KMS_ROTATION=90 /usr/local/bin/woodaudio-player --config /home/tc/.config/woodaudio/config.ini &
BOOTLOCAL
    chmod +x /tmp/wa-bootlocal/opt/bootlocal.sh
fi

CFG_PATH="home/tc/.config/woodaudio"
if [ -f /tmp/wa-bootlocal/opt/.filetool.lst ]; then
    grep -qxF 'opt/bootlocal.sh' /tmp/wa-bootlocal/opt/.filetool.lst 2>/dev/null || echo 'opt/bootlocal.sh' >> /tmp/wa-bootlocal/opt/.filetool.lst
    grep -qxF "$CFG_PATH" /tmp/wa-bootlocal/opt/.filetool.lst 2>/dev/null || echo "$CFG_PATH" >> /tmp/wa-bootlocal/opt/.filetool.lst
else
    printf 'opt/bootlocal.sh\n%s\n' "$CFG_PATH" > /tmp/wa-bootlocal/opt/.filetool.lst
fi

cd /tmp/wa-bootlocal
tar czf "${tce_base}/mydata.tgz" .
cd /
rm -rf /tmp/wa-bootlocal
# Also update live bootlocal.sh
if grep -q "woodaudio-player" /opt/bootlocal.sh 2>/dev/null; then
    sed -i 's|.*woodaudio-player.*|SLINT_KMS_ROTATION=90 /usr/local/bin/woodaudio-player --config /home/tc/.config/woodaudio/config.ini \&|' /opt/bootlocal.sh
else
    printf '\n# Start woodaudio\nSLINT_KMS_ROTATION=90 /usr/local/bin/woodaudio-player --config /home/tc/.config/woodaudio/config.ini &\n' >> /opt/bootlocal.sh
fi
# Upewnij się, że config woodaudio jest w .filetool.lst (live)
if [ -f /opt/.filetool.lst ]; then
    grep -qxF "$CFG_PATH" /opt/.filetool.lst 2>/dev/null || echo "$CFG_PATH" >> /opt/.filetool.lst
fi
echo "mydata.tgz zaktualizowany"
REMOTESCRIPT

    echo "  [3b/5] Backup configu do mydata.tgz..."
    ssh "${pcp_user}@${pcp_host}" "sudo filetool.sh -b" || true

    echo "  [4/5] Instalowanie zależności woodaudio z repozytorium TCZ..."
    ssh "${pcp_user}@${pcp_host}" "while read dep; do
      depname=\"\${dep%.tcz}\"
      marker=\"/usr/local/tce.installed/\$depname\"
      tcz_file=\"${tce_base}/optional/\$dep\"
      if [ -f \"\$marker\" ] && [ ! -f \"\$tcz_file\" ]; then
        sudo rm -f \"\$marker\"
      fi
      tce-load -wi \"\$dep\" 2>&1 || true
    done < ${tce_base}/optional/woodaudio.tcz.dep"
    echo "  [4b/5] Ładowanie woodaudio.tcz..."
    ssh "${pcp_user}@${pcp_host}" "tce-load -i woodaudio.tcz 2>&1 || true"

    echo "  [5/6] Naprawa permisji configu..."
    ssh "${pcp_user}@${pcp_host}" "sudo chmod 666 /home/tc/.config/woodaudio/config.ini 2>/dev/null || true"

    echo "  [6/6] Uruchamianie woodaudio..."
    ssh "${pcp_user}@${pcp_host}" "nohup env SLINT_KMS_ROTATION=90 woodaudio-player --config /home/tc/.config/woodaudio/config.ini &" || true

    echo "  [7/7] Backup konfiguracji (tokeny) do mydata.tgz..."
    ssh "${pcp_user}@${pcp_host}" "sleep 5 && sudo filetool.sh -b 2>&1 || true"

    echo ""
    echo "==> Gotowe!"
    echo "    woodaudio.tcz zainstalowany na ${pcp_host}"
    echo "    Konfiguracja: /home/tc/.config/woodaudio/config.ini"
    echo "    Autostart przy bootowaniu przez /opt/bootlocal.sh"
    echo "    Uruchomiony teraz: woodaudio-player"
    echo ""
    echo "==> WAŻNE: Po pierwszym zalogowaniu przez link Tidal,"
    echo "    uruchom: ssh tc@${pcp_host} sudo filetool.sh -b"
    echo "    aby zapisać tokeny w backupie na stałe."
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
    tcz)          do_tcz "$FEATURES" "$REBUILD" ;;
    pcp-image)    ARCH=armv7 do_tcz "$FEATURES" "$REBUILD" && exec "${SCRIPT_DIR}/build_image.sh" ;;
    install-pcp)  do_install_pcp "$FEATURES" "$REBUILD" ;;
    deploy-rpios) do_deploy_rpios "$FEATURES" "$REBUILD" ;;
    deploy-picore) do_deploy_picore "$FEATURES" "$REBUILD" ;;
    clean)        do_clean ;;
    help|--help|-h) print_usage ;;
    *)            echo "Nieznana komenda: $SUBCOMMAND"; print_usage; exit 1 ;;
esac
