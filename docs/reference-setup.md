# Reference setup

The host setup Jukelight is built and tested on: a mini PC running Ubuntu
Server, driving a wall-mounted 4K monitor. Other setups may work, but this is
the only one we test. Adapt it to your own hardware as needed.

## Contents

1. [What runs where](#what-runs-where)
2. [Hardware](#hardware)
3. [Operating system](#operating-system)
4. [Software](#software)
5. [Kiosk session](#kiosk-session)
6. [Monitor settings](#monitor-settings)
7. [Checks after setup](#checks-after-setup)
8. [Troubleshooting](#troubleshooting)
9. [Tested versions](#tested-versions)

## What runs where

Jukelight runs in a Docker container. The screen side runs directly on the
host, because it needs the monitor.

```
  Host (Ubuntu Server, no desktop environment)
  +--------------------------------------------------------------+
  |  Docker:  jukelight container  (backend + kiosk + admin)     |
  |                 ^  http://localhost:8080                     |
  |                 |                                            |
  |  tty1 auto-login (user "kiosk")                              |
  |    +-- sway (Wayland compositor: no bar, no idle daemon)     |
  |          +-- browser in kiosk mode -> http://localhost:8080  |
  |    +-- display agent (systemd user service): follows         |
  |        /api/v1/display and turns the output on or off        |
  +--------------------------------------------------------------+
            | DisplayPort (preferred) or HDMI (fallback)
            v
  Monitor (sleeps without a signal, wakes when the signal returns)
```

The display agent arrives with the display power feature. Until then, the
monitor simply stays on.

## Hardware

| Item | Reference setup | Minimum for this role |
| --- | --- | --- |
| Machine | Lenovo ThinkCentre M920q Tiny, Intel Core i7-9700T, 32 GB RAM, 512 GB SSD | 64-bit x86, 2 cores, 4 GB RAM, 10 GB free disk |
| Graphics | Intel UHD Graphics 630 (integrated) | A GPU with open-source Mesa drivers and 4K output |
| Video output | DisplayPort 1.2 (4K at 60 Hz). HDMI as fallback, probably 4K at 30 Hz | DisplayPort or HDMI capable of 4K |
| Monitor | LG 27UD68-W (27", 4K) | Sleeps on signal loss and wakes on signal |
| Music player | Bluesound Node (BluOS) | A BluOS player |
| Network | Same LAN as the player | Same LAN as the player |

Why DisplayPort first: the UHD 630 supports HDMI 1.4 natively, which limits 4K
to 30 Hz. That is acceptable for this mostly static screen, so HDMI is a valid
fallback (see [Troubleshooting](#troubleshooting)).

## Operating system

- **Ubuntu Server 26.04 LTS**, without a desktop environment.
- The host must never suspend:

  ```sh
  sudo systemctl mask sleep.target suspend.target hibernate.target hybrid-sleep.target
  ```

- In the BIOS, enable "power on after power loss" (the exact name varies), so
  the host restarts after a power cut.

## Software

| Component | Source | Why |
| --- | --- | --- |
| Docker Engine and the Compose plugin | Docker's official apt repository (not the snap) | Runs Jukelight |
| sway | Ubuntu package `sway` | Minimal Wayland compositor. `swaymsg "output * power off"` and `power on` control the monitor |
| Mesa graphics drivers | Ubuntu packages (usually installed already) | Hardware rendering for sway and the browser |
| Browser | Google Chrome (`.deb` from Google's apt repository) | Kiosk mode. Ubuntu's `chromium` package is a snap, which can be unreliable with Wayland and GPU access outside a desktop environment |
| Fonts | `fonts-noto-core`, `fonts-noto-cjk`, `fonts-noto-color-emoji` | Jukelight bundles its own font, but track titles can contain Japanese, Chinese or Korean characters and emoji, which need system fallback fonts |
| `curl` | Ubuntu package | Used by the display agent |
| `ddcutil` (optional) | Ubuntu package | Fallback if the monitor does not sleep and wake on signal |

Install the Ubuntu packages:

```sh
sudo apt install sway curl fonts-noto-core fonts-noto-cjk fonts-noto-color-emoji
```

For Docker and Chrome, follow their official installation instructions for
Ubuntu. A new Ubuntu LTS is sometimes added to third-party repositories a few
weeks after its release: check that 26.04 is listed.

## Kiosk session

### 1. Create the kiosk user

```sh
sudo useradd --create-home --groups video,render kiosk
sudo loginctl enable-linger kiosk   # lets its user services run without a login
```

### 2. Log in automatically on tty1

Create `/etc/systemd/system/getty@tty1.service.d/autologin.conf`:

```ini
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin kiosk --noclear %I $TERM
```

Start sway from the kiosk user's `~/.profile`, only on tty1:

```sh
if [ "$(tty)" = "/dev/tty1" ]; then
  exec sway
fi
```

### 3. Configure sway

`~/.config/sway/config` for the kiosk user:

```
# One output, no bar, no idle or blanking: only Jukelight decides when the
# screen turns off.
output * bg #000000 solid_color
# Portrait monitor: uncomment and use the name from `swaymsg -t get_outputs`
# output DP-1 transform 90

seat * hide_cursor 1000
default_border none

exec google-chrome --kiosk --noerrdialogs --no-first-run \
  --disable-background-timer-throttling --disable-renderer-backgrounding \
  --ozone-platform=wayland http://localhost:8080
```

Do not run `swayidle` or any other idle daemon.

### 4. Start Jukelight

From the repository, on the host:

```sh
cp .env.example .env      # then set your player's address and other values
docker compose up --build -d
```

### 5. Display agent

Installed as a systemd user service for the kiosk user. The service file and
script are added with the display power feature.

## Monitor settings

LG 27UD68-W, in the on-screen menu (Settings, then General):

| Setting | Value | Why |
| --- | --- | --- |
| Automatic Standby | Off | Otherwise the monitor may sleep while music plays |
| Input | DisplayPort (or the HDMI port in use) | |
| Automatic input switching | Off | Keeps the monitor on the right input while asleep |
| Deep Sleep Mode | Test both values | It saves power, but on DisplayPort it can stop the monitor from waking (see Troubleshooting) |

## Checks after setup

| Check | Command or place | Expected |
| --- | --- | --- |
| Resolution and refresh rate | `swaymsg -t get_outputs` | 3840x2160 at 60 Hz on DisplayPort (30 Hz on HDMI is acceptable) |
| Hardware acceleration | `chrome://gpu` in the kiosk browser | Hardware accelerated, not software only |
| Monitor sleeps | `swaymsg "output * power off"` | The monitor goes into standby |
| Monitor wakes | `swaymsg "output * power on"` | Picture back in under 5 seconds |
| Page stays current | Play music with the output off, then turn it on | The screen shows the current track at once, without reloading |

Run the `swaymsg` commands as the kiosk user, inside its sway session.

## Troubleshooting

**The monitor does not wake, or the output disappears in sway.** Some monitors
drop the DisplayPort connection in deep sleep. The output then vanishes and
may not come back. Try, in this order:

1. Turn Deep Sleep Mode off in the monitor's menu.
2. Re-enable the output when it reappears (sway configuration).
3. Use HDMI instead (4K at 30 Hz).
4. Use DDC/CI (`ddcutil setvcp D6 ...`) as a last resort; many monitors stop
   answering DDC once asleep.

**The browser shows a black or blank page.** Check that the container runs
(`docker compose ps`) and that `http://localhost:8080` answers on the host.

**Animations stutter.** Check `chrome://gpu`: if rendering is in software
mode, the browser is not using the GPU. Check that the kiosk user is in the
`video` and `render` groups.

## Tested versions

Recorded on the reference host when the display power feature is set up.

| Component | Version |
| --- | --- |
| Ubuntu Server | 26.04 LTS |
| Kernel | (to record) |
| sway | (to record) |
| Google Chrome | (to record) |
| Docker Engine / Compose | (to record) |
| Mesa | (to record) |
| BluOS player firmware | (to record) |
