#!/bin/bash
set -euo pipefail

# =============================================================================
# build_image.sh - Tworzy obraz SD piCorePlayer z pre-instalowanym woodaudio
# Wymaga: Docker, curl, xz
# Użycie: ./build.sh tcz && ./build_image.sh
# =============================================================================

PCP_VERSION="11.1.0"
PCP_32_URL="https://repo.pcplayer.org/insitu/piCorePlayer${PCP_VERSION}/piCorePlayer${PCP_VERSION}.img.xz"
PCP_32_SHA256="c29e81e37b4631bf18ab27f82dd1e5bec7da2bc35aabc6985b8189b757bc19b5"
OUTPUT_NAME="woodaudio-pcp${PCP_VERSION}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WORK_DIR="${SCRIPT_DIR}/build_image_work"
DIST_DIR="${SCRIPT_DIR}/dist"

BASE_IMAGE_XZ="${WORK_DIR}/pcp${PCP_VERSION}.img.xz"
BASE_IMAGE="${WORK_DIR}/pcp${PCP_VERSION}.img"
OUTPUT_IMAGE="${SCRIPT_DIR}/${OUTPUT_NAME}.img"

# --- Check prerequisites ---

if [ ! -f "${DIST_DIR}/woodaudio.tcz" ]; then
    echo "ERROR: Nie znaleziono ${DIST_DIR}/woodaudio.tcz"
    echo "Najpierw uruchom: ARCH=armv7 ./build.sh tcz"
    exit 1
fi

if ! command -v docker &>/dev/null; then
    echo "ERROR: Docker nie jest zainstalowany"
    exit 1
fi

if ! command -v curl &>/dev/null; then
    echo "ERROR: curl nie jest zainstalowany"
    exit 1
fi

if ! command -v xz &>/dev/null; then
    echo "ERROR: xz-utils nie są zainstalowane"
    exit 1
fi

echo "==> build_image.sh - piCorePlayer ${PCP_VERSION} + woodaudio"
echo ""

# --- Step 1: Download base image ---

mkdir -p "${WORK_DIR}"

if [ ! -f "${BASE_IMAGE}" ]; then
    if [ ! -f "${BASE_IMAGE_XZ}" ]; then
        echo "[1/4] Pobieranie piCorePlayer ${PCP_VERSION} (32-bit)..."
        curl -L -o "${BASE_IMAGE_XZ}" "${PCP_32_URL}"
        echo "    Sprawdzanie SHA256..."
        echo "${PCP_32_SHA256}  ${BASE_IMAGE_XZ}" | shasum -a 256 -c
    fi

    echo "[1/4] Decompresja obrazu..."
    xz -dc "${BASE_IMAGE_XZ}" > "${BASE_IMAGE}"
    echo "    Rozmiar: $(du -h "${BASE_IMAGE}" | cut -f1)"
else
    echo "[1/4] Obraz już pobrany: ${BASE_IMAGE}"
fi

# --- Step 2: Prepare files ---

echo "[2/4] Kopiowanie plików woodaudio..."
cp "${DIST_DIR}/woodaudio.tcz" "${WORK_DIR}/"
cp "${DIST_DIR}/woodaudio.tcz.dep" "${WORK_DIR}/"
cp "${DIST_DIR}/woodaudio.tcz.md5.txt" "${WORK_DIR}/"
cp "${SCRIPT_DIR}/sample_config.ini" "${WORK_DIR}/"
cp "${BASE_IMAGE}" "${WORK_DIR}/input.img"

# --- Step 3: Modify image via Docker ---

echo "[3/4] Modyfikowanie obrazu w Docker (to może chwilę potrwać)..."

docker run --rm --privileged \
    -v "${WORK_DIR}:/work:rw" \
    ubuntu:22.04 \
    bash -c '
set -euxo pipefail

# Install needed tools
apt-get update -qq && apt-get install -y -qq --no-install-recommends \
    util-linux mount fdisk xz-utils file tar gzip >/dev/null 2>&1

IMG="/work/input.img"

# Copy image to container-local storage for reliable loop device access
cp "$IMG" /tmp/input.img
IMG_LOCAL="/tmp/input.img"

# Find partition layout
echo "=== Partition table ==="
fdisk -l "$IMG_LOCAL"

# Set up loop device with partition scanning
LOOP_DEV=$(losetup -f)
losetup -P "$LOOP_DEV" "$IMG_LOCAL"
echo "Loop device: $LOOP_DEV"
ls -la ${LOOP_DEV}*

# Identify the ext4 data partition (usually partition 2, ~300-500MB)
PART=""
for p in ${LOOP_DEV}p*; do
    FSTYPE=$(blkid -o value -s TYPE "$p" 2>/dev/null || true)
    echo "  $p: $FSTYPE"
    if [ "$FSTYPE" = "ext4" ]; then
        PART="$p"
    fi
done

if [ -z "$PART" ]; then
    echo "ERROR: Nie znaleziono partycji ext4"
    losetup -d "$LOOP_DEV"
    exit 1
fi
echo "Znaleziono partycję danych: $PART"

# Mount the data partition
mount "$PART" /mnt
echo "Zawartość partycji danych:"
ls -la /mnt/

# --- Inject woodaudio TCZ ---
echo ""
echo "==> Kopiowanie woodaudio.tcz do tce/optional/..."
mkdir -p /mnt/tce/optional
cp -v /work/woodaudio.tcz /mnt/tce/optional/
cp -v /work/woodaudio.tcz.dep /mnt/tce/optional/
cp -v /work/woodaudio.tcz.md5.txt /mnt/tce/optional/

echo "==> Dodawanie do onboot.lst..."
if [ -f /mnt/tce/onboot.lst ]; then
    grep -qxF "woodaudio.tcz" /mnt/tce/onboot.lst 2>/dev/null || \
        echo "woodaudio.tcz" >> /mnt/tce/onboot.lst
    echo "onboot.lst:"
    cat /mnt/tce/onboot.lst
else
    echo "woodaudio.tcz" > /mnt/tce/onboot.lst
    echo "Utworzono onboot.lst"
fi

# --- Prepare mydata.tgz with config + bootlocal.sh ---
echo "==> Przygotowywanie mydata.tgz..."

MYDATA_EXISTING=""
if [ -f /mnt/tce/mydata.tgz ]; then
    MYDATA_EXISTING="/mnt/tce/mydata.tgz"
    echo "Istniejący mydata.tgz znaleziony"
fi

# Create staging directory
mkdir -p /tmp/mydata_staging

# Copy existing backup if present
if [ -n "$MYDATA_EXISTING" ]; then
    tar xzf "$MYDATA_EXISTING" -C /tmp/mydata_staging 2>/dev/null || true
fi

# Add opt/bootlocal.sh (auto-start)
mkdir -p /tmp/mydata_staging/opt
if [ -f /tmp/mydata_staging/opt/bootlocal.sh ]; then
    if grep -q "woodaudio-player" /tmp/mydata_staging/opt/bootlocal.sh 2>/dev/null; then
        sed -i 's|.*woodaudio-player.*|SLINT_KMS_ROTATION=90 /usr/local/bin/woodaudio-player --config /home/tc/.config/woodaudio/config.ini \&|' /tmp/mydata_staging/opt/bootlocal.sh
    else
        printf "\n# Start woodaudio\nSLINT_KMS_ROTATION=90 /usr/local/bin/woodaudio-player --config /home/tc/.config/woodaudio/config.ini &\n" >> /tmp/mydata_staging/opt/bootlocal.sh
    fi
else
    cat > /tmp/mydata_staging/opt/bootlocal.sh << "EOF"
#!/bin/sh
# Start woodaudio
SLINT_KMS_ROTATION=90 /usr/local/bin/woodaudio-player --config /home/tc/.config/woodaudio/config.ini &
EOF
    chmod +x /tmp/mydata_staging/opt/bootlocal.sh
fi
echo "bootlocal.sh:"
cat /tmp/mydata_staging/opt/bootlocal.sh

# Add home/tc/.config/woodaudio/config.ini (sample config)
mkdir -p /tmp/mydata_staging/home/tc/.config/woodaudio
cp /work/sample_config.ini /tmp/mydata_staging/home/tc/.config/woodaudio/config.ini
echo "config.ini skopiowany"

# Add files to .filetool.lst if not already there
CFG_PATH="home/tc/.config/woodaudio"
if [ -f /tmp/mydata_staging/opt/.filetool.lst ]; then
    grep -qxF "opt/bootlocal.sh" /tmp/mydata_staging/opt/.filetool.lst 2>/dev/null || \
        echo "opt/bootlocal.sh" >> /tmp/mydata_staging/opt/.filetool.lst
    grep -qxF "$CFG_PATH" /tmp/mydata_staging/opt/.filetool.lst 2>/dev/null || \
        echo "$CFG_PATH" >> /tmp/mydata_staging/opt/.filetool.lst
    true
else
    mkdir -p /tmp/mydata_staging/opt
    printf "opt/bootlocal.sh\n%s\n" "$CFG_PATH" > /tmp/mydata_staging/opt/.filetool.lst
fi

# Create mydata.tgz
cd /tmp/mydata_staging
tar czf /mnt/tce/mydata.tgz .
cd /
echo "mydata.tgz zawiera:"
tar tzf /mnt/tce/mydata.tgz

# --- Cleanup ---
umount /mnt
losetup -d "$LOOP_DEV"

# Copy modified image back to work dir
cp "$IMG_LOCAL" /work/modified.img

echo ""
echo "=== Modyfikacja zakończona ==="
'

# --- Step 4: Copy result ---

echo "[4/4] Kopiowanie gotowego obrazu..."
cp "${WORK_DIR}/modified.img" "${OUTPUT_IMAGE}"
rm -f "${WORK_DIR}/modified.img" "${WORK_DIR}/input.img" "${WORK_DIR}/woodaudio.tcz" "${WORK_DIR}/woodaudio.tcz.dep" "${WORK_DIR}/woodaudio.tcz.md5.txt" "${WORK_DIR}/sample_config.ini"

echo ""
echo "================================================"
echo " Gotowe! Obraz:"
echo "   ${OUTPUT_IMAGE}"
echo " Rozmiar: $(du -h "${OUTPUT_IMAGE}" | cut -f1)"
echo "================================================"
echo ""
echo "Flashowanie na kartę SD:"
echo "   Raspberry Pi Imager -> wybierz niestandardowy obraz"
echo "   lub:"
echo "   dd if=${OUTPUT_NAME}.img of=/dev/sdX bs=4M status=progress"
echo "   (zastąp /dev/sdX swoją kartą SD)"
echo ""
echo "Po uruchomieniu Raspberry Pi z tej karty:"
echo "   - woodaudio.tcz załaduje się automatycznie"
echo "   - woodaudio-player wystartuje z /etc/woodaudio/config.ini"
echo "   - skonfiguruj swoje konto Tidal w config.ini przez SSH"
