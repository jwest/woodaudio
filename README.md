```
sudo apt-get install git openssl libssl-dev libudev-dev libasound2-dev libxkbcommon-dev libinput-dev libgbm-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/jwest/woodaudio
```

```
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
https://www.waveshare.com/wiki/4inch_DSI_LCD?srsltid=AfmBOopveAYJakBDdYDAp-91MISTAYb-g1gS0d5N9CSk-M4fb0lqguJa

sudo apt install sysfsutils
sudo nano /etc/sysfs.conf
```
devices/system/cpu/cpu0/cpufreq/scaling_governor = powersave
devices/system/cpu/cpu1/cpufreq/scaling_governor = powersave
devices/system/cpu/cpu2/cpufreq/scaling_governor = powersave
devices/system/cpu/cpu3/cpufreq/scaling_governor = powersave
```
sudo systemctl restart sysfsutils