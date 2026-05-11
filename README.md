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

Budowanie natywne na Raspberry Pi jest możliwe, ale bardzo wolne. Zamiast tego można skompilować woodaudio na macOS przy użyciu Dockera — kontener ARM64 działa natywnie na Apple Silicon (M1–M4) **bez emulacji x86**, co daje szybką kompilację i brak problemów z cross-kompilacją C-zależności.

### Wymagania

- [Docker Desktop](https://www.docker.com/products/docker-desktop/) dla macOS (Apple Silicon)
- Ok. 3 GB wolnego miejsca na dysku

### Szybki start — jeden skrypt

```bash
# Kompilacja (automatycznie buduje obraz Docker, jeśli brak)
./build.sh build

# Binary: target/release/woodaudio-player
```

### Wszystkie komendy

| Komenda | Opis |
|---------|------|
| `./build.sh build` | Kompilacja dla aarch64 (auto-build obrazu jeśli brak) |
| `./build.sh dist` | Kompilacja + pakowanie dist/ z bibliotekami .so (dla piCore) |
| `./build.sh image` | Budowa/przebudowa obrazu Docker |
| `./build.sh deploy-rpios` | Kompilacja + scp binary + instalacja zależności na Raspberry Pi OS |
| `./build.sh deploy-picore` | Kompilacja + dist + scp dist/ na piCore |
| `./build.sh clean` | Usunięcie target/ i dist/ |
| `./build.sh help` | Pomoc |

### Opcje

```bash
# Użyj niestandardowych feature'ów
./build.sh build --features "gui"

# Przebuduj obraz Dockera (gdy zmienisz zależności)
./build.sh build --rebuild

# Kopiuj binarkę po zbudowaniu
./build.sh build --copy-dest /tmp/woodaudio

# Użyj innego obrazu
./build.sh build --image my-custom-image:latest
```

### Predefiniowane zmienne

Wszystkie zmienne konfiguracyjne znajdują się na początku `build.sh`. Można je zmienić bezpośrednio w pliku lub ustawić jako zmienne środowiskowe:

| Zmienna | Domyślna wartość | Opis |
|---------|-----------------|------|
| `IMAGE_NAME` | `woodaudio-builder:arm64` | Nazwa obrazu Docker |
| `DOCKERFILE` | `Dockerfile.arm64` | Plik Dockerfile |
| `FEATURES` | `gui,gpio` | Feature flags dla cargo |
| `TARGET` | `aarch64-unknown-linux-gnu` | Target kompilacji |
| `BINARY_NAME` | `woodaudio-player` | Nazwa pliku wynikowego |
| `PI_HOST` | `192.168.1.100` | Adres IP Raspberry Pi |
| `PI_USER` | `pi` | Użytkownik SSH na Pi |
| `PI_DEST_RPIOS` | `/home/pi` | Ścieżka docelowa na Raspberry Pi OS |
| `PI_DEST_PICORE` | `/opt/woodaudio` | Ścieżka docelowa na piCore |

**Przykład — deploy na Pi z innym adresem:**

```bash
PI_HOST=192.168.1.42 ./build.sh deploy-rpios
```

### Deploy na Raspberry Pi OS

```bash
# Krok 1: Skonfiguruj PI_HOST w build.sh (lub zmienna środowiskowa)

# Krok 2: Zbuduj i wdróż jednym poleceniem
./build.sh deploy-rpios

# Skrypt automatycznie:
#   1. Kompiluje binary
#   2. Kopiuje go na Pi przez scp
#   3. Instaluje zależności runtime przez apt
```

Binary wymaga tych bibliotek runtime na Pi:

```bash
sudo apt-get install libdrm2 libgbm1 libinput10 libudev1 \
                     libasound2 libxkbcommon0 libssl3
```

Są domyślnie dostępne w Raspberry Pi OS Bookworm (64-bit).

### Deploy na piCore (Tiny Core Linux)

piCore to minimalistyczna dystrybucja bez menedżera pakietów apt. Zamiast instalować brakujące biblioteki pojedynczo, dostarczamy wszystkie `.so` razem z binarką.

```bash
# Krok 1: Skonfiguruj PI_HOST w build.sh

# Krok 2: Zbuduj, spakuj i wdróż jednym poleceniem
./build.sh deploy-picore

# Skrypt automatycznie:
#   1. Kompiluje binary
#   2. Zbiera wszystkie wymagane .so do dist/libs/
#   3. Tworzy skrypty uruchomieniowe (run.sh, start-woodaudio.sh)
#   4. Kopiuje całość na Pi przez scp
```

Katalog `dist/` (generowany przez `./build.sh dist`):
```
dist/
├── woodaudio-player      # Binary
├── libs/                  # Biblioteki współdzielone .so
│   ├── libgbm.so.1
│   ├── libdrm.so.2
│   ├── libinput.so.10
│   ├── ...
├── run.sh                 # Uruchomienie (LD_LIBRARY_PATH + własny linker)
└── start-woodaudio.sh     # Skrypt startowy do autostartu
```

**Uruchomienie na piCore:**
```bash
/opt/woodaudio/run.sh
```

**Autostart** — dodaj do `/opt/bootlocal.sh` lub `.profile`:
```bash
/opt/woodaudio/start-woodaudio.sh &
```

### Jak to działa

`Dockerfile.arm64` bazuje na `arm64v8/ubuntu:22.04` — obrazie natywnym dla architektury ARM64. Docker Desktop na Apple Silicon uruchamia maszynę wirtualną ARM64 Linux, więc kontener działa natywnie. Rust kompiluje kod dla `aarch64-unknown-linux-gnu`, który jest tożsamy z architekturą kontenera.

### Struktura plików

| Plik | Opis |
|------|------|
| `build.sh` | **Główny skrypt** — kompilacja, pakowanie, deploy (all-in-one) |
| `Dockerfile.arm64` | Obraz Docker z toolchainem (Ubuntu ARM64) |
| `Dockerfile.aarch64` | (Opcjonalny) obraz dla `cross-rs` na hoście x86_64 Linux |
| `Cross.toml` | Konfiguracja dla narzędzia `cross` |
| `.cargo/config.toml` | Ustawienia linkera dla targetu aarch64 |
| `dist/` | Katalog z gotowym do wdrożenia zestawem (generowany przez `build.sh dist`) |
| `dist/run.sh` | Wrapper uruchamiający z własnym `LD_LIBRARY_PATH` |
| `dist/start-woodaudio.sh` | Skrypt startowy do autostartu |
