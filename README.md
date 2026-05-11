# woodaudio

## Przygotowanie Raspberry Pi

```bash
sudo apt-get install git openssl libssl-dev libudev-dev libasound2-dev libxkbcommon-dev libinput-dev libgbm-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/jwest/woodaudio
```

```bash
sudo apt-get install plymouth plymouth-themes
sudo nano /boot/firmware/cmdline.txt
append to file: quiet splash plymouth.ignore-serial-consoles logo.nologo
```

/boot/firmware/config.txt:
```
dtoverlay=vc4-kms-v3d,noaudio
dtoverlay=vc4-kms-dsi-waveshare-panel,4_0_inch,rotation=90
```

hifiberry configuration:
https://www.hifiberry.com/docs/software/configuring-linux-3-18-x/

waveshare 4inch lcd
https://www.waveshare.com/wiki/4inch_DSI_LCD

```bash
sudo apt install sysfsutils
sudo nano /etc/sysfs.conf
```
```
devices/system/cpu/cpu0/cpufreq/scaling_governor = powersave
devices/system/cpu/cpu1/cpufreq/scaling_governor = powersave
devices/system/cpu/cpu2/cpufreq/scaling_governor = powersave
devices/system/cpu/cpu3/cpufreq/scaling_governor = powersave
```
```bash
sudo systemctl restart sysfsutils
```

---

## Cross-kompilacja (budowanie na macOS → Raspberry Pi)

Budowanie natywne na Raspberry Pi jest możliwe, ale bardzo wolne. Zamiast tego można skompilować woodaudio na macOS przy użyciu Dockera, uzyskując gotowy binary dla aarch64 Linux.

### Wymagania

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) dla macOS (Apple Silicon)
- Ok. 3 GB wolnego miejsca na dysku

### Szybki start

```bash
# Zbuduj obraz i skompiluj (jedna komenda)
./build-cross.sh

# Binary znajduje się w:
# target/release/woodaudio-player
```

### Komendy krok po kroku

```bash
# 1. Zbuduj obraz z toolchainem (jednorazowo)
docker build -f Dockerfile.arm64 -t woodaudio-builder:arm64 .

# 2. Cross-kompilacja
docker run --rm -v "$(pwd):/app" woodaudio-builder:arm64 \
    cargo build --release --features gui,gpio

# 3. Skopiuj na Raspberry Pi
scp target/release/woodaudio-player pi@<ip>:~/
```

### Opcje build-cross.sh

```bash
# Użyj niestandardowych feature'ów
./build-cross.sh --features "gui,gpio"

# Przebuduj obraz Dockera (gdy zmienisz zależności)
./build-cross.sh --rebuild

# Skopiuj binarkę po zbudowaniu
./build-cross.sh --copy-dest /tmp/woodaudio

# Pomoc
./build-cross.sh --help
```

### Wymagane pakiety runtime na Pi

Binary wymaga tych bibliotek na Raspberry Pi (zainstalowane przez `apt-get`:

```bash
sudo apt-get install libdrm2 libgbm1 libinput10 libudev1 \
                     libasound2 libxkbcommon0 libssl3
```

Są one domyślnie dostępne w Raspberry Pi OS Bookworm (wersja 64-bit).

### Jak to działa

`Dockerfile.arm64` bazuje na `arm64v8/ubuntu:22.04` — obrazie natywnym dla architektury ARM64. Ponieważ Docker Desktop na Apple Silicon (M1/M2/M3/M4) uruchamia maszynę wirtualną ARM64 Linux, kontener działa natywnie **bez emulacji x86**. Rust kompiluje kod dla `aarch64-unknown-linux-gnu`, który jest tożsamy z architekturą kontenera. Efekt: szybka kompilacja i brak problemów z cross-kompilacją C zależności.

---

## Deploy na piCore (Tiny Core Linux)

piCore to minimalistyczna dystrybucja bez menedżera pakietów apt. Zamiast instalować brakujące biblioteki pojedynczo, dostarczamy wszystkie `.so` razem z binarką.

### Budowanie + eksport (macOS)

```bash
# Krok 1: Zbuduj obraz narzędziowy (jednorazowo)
docker build -f Dockerfile.arm64 -t woodaudio-builder:arm64 .

# Krok 2: Zbuduj i wyeksportuj binary + biblioteki do katalogu dist/
./export-dist.sh
```

Katalog `dist/` zawiera:
```
dist/
├── woodaudio-player      # Binary 35MB
├── libs/                  # Wszystkie .so (biblioteki + loader)
│   ├── libgbm.so.1
│   ├── libdrm.so.2
│   ├── libinput.so.10
│   ├── libasound.so.2
│   ├── libc.so.6
│   └── ...
├── run.sh                 # Uruchomienie (LD_LIBRARY_PATH + własny linker)
└── start-woodaudio.sh     # Skrypt startowy do autostartu
```

### Wdrożenie na piCore

```bash
# Na macOS: skopiuj cały katalog dist/ na Pi
scp -r dist/* pi@<ip>:/opt/woodaudio/

# Na piCore: uruchom
/opt/woodaudio/run.sh
```

### Autostart na piCore

Dodaj do `/opt/bootlocal.sh` lub `.profile`:

```bash
/opt/woodaudio/start-woodaudio.sh &
```

### Struktura plików

| Plik | Opis |
|------|------|
| `Dockerfile.arm64` | Obraz Docker z toolchainem i zależnościami |
| `Dockerfile.aarch64` | (Opcjonalny) obraz dla `cross-rs` na hoście x86_64 Linux |
| `Cross.toml` | Konfiguracja dla narzędzia `cross` |
| `.cargo/config.toml` | Ustawienia linkera dla targetu aarch64 |
| `build-cross.sh` | Skrypt do budowania binarki (bez eksportu libs) |
| `export-dist.sh` | Skrypt do budowania + eksportu dist/ (z libs) |
| `dist/` | Katalog z gotowym do wdrożenia zestawem (generowany) |
| `dist/run.sh` | Wrapper uruchamiający z własnym `LD_LIBRARY_PATH` |
| `dist/start-woodaudio.sh` | Skrypt startowy do autostartu |
