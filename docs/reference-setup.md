# Reference setup

The host setup Jukelight is built and tested on: a mini PC running Ubuntu
Server, driving a wall-mounted 4K monitor. Other setups may work, but this is
the only one we test. Adapt it to your own hardware as needed.

## Contents

1. [What runs where](#what-runs-where)
2. [Hardware](#hardware)
3. [Operating system](#operating-system)
4. [Software](#software)
5. [Jukelight](#jukelight)
6. [Kiosk session](#kiosk-session)
7. [Who logs in where](#who-logs-in-where)
8. [Monitor settings](#monitor-settings)
9. [Working over SSH](#working-over-ssh)
10. [Checks after setup](#checks-after-setup)
11. [Troubleshooting](#troubleshooting)
12. [Tested versions](#tested-versions)

## What runs where

Jukelight runs in a Docker container. The screen side runs directly on the
host, because it needs the monitor.

```
  Host (Ubuntu Server, no desktop environment)
  +--------------------------------------------------------------+
  |  Docker:  jukelight container  (backend + kiosk)             |
  |                 ^  http://localhost:<HTTP_PORT>              |
  |                 |                                            |
  |  jukelight-kiosk.service (system): sway on tty1 as "kiosk"   |
  |    +-- Chrome in kiosk mode -> Jukelight                     |
  |  jukelight-display-agent.service (kiosk's user service):     |
  |    follows /api/v1/display and switches the monitor          |
  |                                                              |
  |  /etc/jukelight/host.env: JUKELIGHT_URL for both services    |
  +--------------------------------------------------------------+
            | HDMI (here through a wireless HDMI extender)
            v
  Monitor
```

Without the display agent, the monitor simply stays on.

## Hardware

| Item | Reference setup | Minimum for this role |
| --- | --- | --- |
| Machine | Lenovo ThinkCentre M920q Tiny, Intel Core i7-9700T, 32 GB RAM, 512 GB SSD | 64-bit x86, 2 cores, 4 GB RAM, 10 GB free disk |
| Graphics | Intel UHD Graphics 630 (integrated) | A GPU with open-source Mesa drivers and 4K output |
| Video output | HDMI, 4K at 30 Hz, through a wireless HDMI extender (Digitus DS-55323) | DisplayPort or HDMI capable of 4K |
| Monitor | LG 27UD68-W (27", 4K), mounted in portrait | Sleeps on signal loss and wakes on signal, or switches on by itself when its power returns |
| Smart plug | Shelly Plug M Gen3 (Schuko) for the monitor | Only where the monitor doesn't sleep on signal loss: a plug or relay that a command can switch |
| Music player | Bluesound Node (BluOS) | A BluOS player |
| Network | Same LAN as the player | Same LAN as the player |

DisplayPort gives 4K at 60 Hz; the UHD 630 supports HDMI 1.4 natively, which
limits 4K to 30 Hz. That is acceptable for this mostly static screen.

**The wireless extender.** Its receiver identifies itself to the host
(`swaymsg` lists the output as `VIP UHD4000`), not as the monitor. When the
host stops sending a picture, the receiver shows its own logo, so the monitor
never sleeps from signal loss. A smart plug switches it instead: see
[7. Smart plug](#7-smart-plug-optional). The receiver needs USB power; take it
from its own USB power adapter or the monitor, not from the music player.

## Operating system

- **Ubuntu Server 26.04 LTS**, without a desktop environment, with OpenSSH
  so the host can be managed remotely.
- The time zone, so the kiosk's clock shows local time (`Europe/Berlin` as
  an example), and the locales the kiosk uses:

  ```sh
  sudo timedatectl set-timezone Europe/Berlin
  locale -a        # de_DE.utf8 (or another locale) must be listed
  ```

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
| sway | Ubuntu package `sway` | Minimal Wayland compositor. `swaymsg "output * power off"` and `power on` switch the output |
| Mesa graphics drivers | Ubuntu packages (usually installed already) | Hardware rendering for sway and the browser |
| Browser | Google Chrome (`.deb` from Google) | Kiosk mode. Ubuntu's `chromium` package is a snap, which can be unreliable with Wayland and GPU access outside a desktop environment |
| Fonts | `fonts-noto-core`, `fonts-noto-cjk`, `fonts-noto-color-emoji` | Jukelight bundles its own font, but track titles can contain Japanese, Chinese or Korean characters and emoji, which need system fallback fonts |
| `curl`, `git` | Ubuntu packages | The display agent and the kiosk use `curl`; `git` fetches Jukelight |
| `ddcutil` (optional) | Ubuntu package | Fallback if the monitor does not sleep and wake on signal |

Install the Ubuntu packages:

```sh
sudo apt install sway curl git fonts-noto-core fonts-noto-cjk fonts-noto-color-emoji
```

**Docker**, from Docker's apt repository:

```sh
sudo apt install -y ca-certificates curl
sudo install -m 0755 -d /etc/apt/keyrings
sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc
sudo chmod a+r /etc/apt/keyrings/docker.asc
sudo tee /etc/apt/sources.list.d/docker.sources > /dev/null <<EOF
Types: deb
URIs: https://download.docker.com/linux/ubuntu
Suites: $(. /etc/os-release && echo "${UBUNTU_CODENAME:-$VERSION_CODENAME}")
Components: stable
Signed-By: /etc/apt/keyrings/docker.asc
EOF
sudo apt update
sudo apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin
```

A new Ubuntu LTS is sometimes added to Docker's repository a few weeks after
its release. Until then, Ubuntu's own packages work: `sudo apt install
docker.io docker-compose-v2`.

Let your own user run Docker, and start Docker at boot:

```sh
sudo usermod -aG docker "$USER"     # log out and in again to apply
sudo systemctl enable --now docker
```

Membership of `docker` is as powerful as root (see
[Who logs in where](#who-logs-in-where)): add only your own account, never
the kiosk user.

**Google Chrome**, from Google's package (it also adds Google's apt
repository, so Chrome updates with the system):

```sh
cd /tmp
curl -fLO https://dl.google.com/linux/direct/google-chrome-stable_current_amd64.deb
sudo apt install -y ./google-chrome-stable_current_amd64.deb
rm google-chrome-stable_current_amd64.deb
```

## Jukelight

As your own user (not the kiosk user), in a folder of your choice:

```sh
git clone https://github.com/poulzinho/jukelight.git
cd jukelight
cp .env.example .env      # set BLUOS_HOST, and the other values you need
docker compose up --build -d
docker compose ps         # "healthy" after about 30 seconds
curl -s localhost:8080/api/v1/player
```

The container restarts by itself after a reboot (`restart: unless-stopped`),
unless it was stopped with `docker compose stop` or `down`.

**After an update** (`git pull` and `docker compose up --build -d`), the
kiosk reloads by itself: when the open page reconnects to the new backend,
it notices the new kiosk build and reloads once. An update without kiosk
changes doesn't reload it. If the kiosk ever keeps showing the old version,
see [Troubleshooting](#troubleshooting).

If you use a port other than 8080, set `HTTP_PORT` in `.env`; the address in
`/etc/jukelight/host.env` (next section) must match it.

For a private copy of the repository, give the host a read-only deploy key
(an SSH key added under the repository's Settings, Deploy keys, without
write access) instead of your own credentials.

## Kiosk session

### 1. Tell the host where Jukelight runs

The kiosk and the display agent run outside Docker and read Jukelight's
address from one file:

```sh
sudo install -d /etc/jukelight
echo 'JUKELIGHT_URL=http://localhost:8080' | sudo tee /etc/jukelight/host.env
```

Jukelight's own `.env` stays private to Docker: it can contain the Music
Assistant token, which the kiosk user must not read.

### 2. Create the kiosk user

```sh
sudo useradd --create-home --groups video,render kiosk
sudo loginctl enable-linger kiosk   # lets its user services run without a login
```

Do not give it a password, `sudo` or the `docker` group: see
[Who logs in where](#who-logs-in-where).

### 3. Configure sway

`/home/kiosk/.config/sway/config` for the kiosk user. It replaces sway's
default configuration entirely, so the kiosk has no bar and no key bindings:

```sh
sudo -u kiosk mkdir -p /home/kiosk/.config/sway
sudo -u kiosk tee /home/kiosk/.config/sway/config > /dev/null <<'EOF'
# Jukelight kiosk. One output, no bar, no idle or blanking: only Jukelight
# decides when the screen turns off.
output * bg #000000 solid_color
# Portrait mounting: 270 or 90 depending on how the monitor hangs. Remove
# the line for landscape.
output * transform 270

# Hide the pointer: move it to a corner at start, hide it after 1 s.
seat * hide_cursor 1000
exec swaymsg seat seat0 cursor set 10000 10000
default_border none

# Start Chrome once Jukelight answers (it may start later than sway at boot).
# JUKELIGHT_URL comes from /etc/jukelight/host.env (jukelight-kiosk.service).
# LANGUAGE and LC_ALL set the clock's format (here 24 hours): on Linux
# Chrome ignores --lang, and the login session replaces LANG with the
# system locale.
exec sh -c 'until curl -sf "$JUKELIGHT_URL/api/v1/player" > /dev/null; do sleep 2; done; exec env LANGUAGE=de LC_ALL=de_DE.UTF-8 google-chrome --kiosk --noerrdialogs --no-first-run --disable-background-timer-throttling --disable-renderer-backgrounding --ozone-platform=wayland "$JUKELIGHT_URL"'

# The display agent's user service needs sway's socket: restart it, so every
# new sway session gets an agent with its socket.
exec "systemctl --user import-environment SWAYSOCK WAYLAND_DISPLAY; systemctl --user restart jukelight-display-agent"
EOF
```

`output *` applies to whichever output is connected, so the configuration
survives switching between DisplayPort and HDMI. For a 12-hour clock or
another language, use another locale in the Chrome line (for example
`LANGUAGE=en LC_ALL=en_US.UTF-8`); it must be installed (`locale -a`). Do not run `swayidle` or any
other idle daemon.

### 4. Chrome policies

Chrome in kiosk mode still shows some of its own prompts. When its language
(from the sway configuration, step 3) differs from the page's, it offers to
translate the page, and the prompt stays on the screen. Switch translation
off with a Chrome policy, which applies to every Chrome on the host:

```sh
sudo mkdir -p /etc/opt/chrome/policies/managed
echo '{ "TranslateEnabled": false }' | sudo tee /etc/opt/chrome/policies/managed/jukelight.json
```

Jukelight's page itself asks browsers not to translate it (`translate="no"`),
which prevents the prompt; the policy also covers other pages and older
Jukelight versions. The command-line flag `--disable-features=Translate`
does not stop the prompt (checked with Chrome 156). A normal Chrome window on the host shows
the active policies at `chrome://policy`.

### 5. Start sway on tty1 as a service

A system service runs sway as the kiosk user on tty1, with a login session
but without a login prompt or a shell, and starts it again if it exits:

```sh
sudo tee /etc/systemd/system/jukelight-kiosk.service > /dev/null <<'EOF'
[Unit]
Description=Jukelight kiosk (sway on tty1)
After=systemd-user-sessions.service
Conflicts=getty@tty1.service

[Service]
EnvironmentFile=/etc/jukelight/host.env
User=kiosk
PAMName=login
TTYPath=/dev/tty1
StandardInput=tty
UtmpIdentifier=tty1
# Show tty1 first; sway needs the active console.
ExecStartPre=+/usr/bin/chvt 1
ExecStart=/usr/bin/sway
Restart=always
RestartSec=5

[Install]
WantedBy=graphical.target
EOF
sudo systemctl daemon-reload
sudo systemctl set-default graphical.target
sudo systemctl enable --now jukelight-kiosk
```

Check it: `systemctl status jukelight-kiosk` is active, and
`loginctl list-sessions` lists a session of `kiosk` on `seat0`, `tty1`.
After a change to the sway configuration: `sudo systemctl restart
jukelight-kiosk`.

### 6. Display agent

The agent follows Jukelight's display decisions and switches the monitor
with `swaymsg`: on while music plays, off a grace period after it stops
(`DISPLAY_GRACE_MINUTES` in `.env`, 10 minutes by default). It is a shell
script run as a systemd user service of the kiosk user, started by the sway
configuration (step 3).

From the repository, as your own user:

```sh
# The folders first, as the kiosk user: install -D would create them as root.
sudo -u kiosk mkdir -p /home/kiosk/.local/bin /home/kiosk/.config/systemd/user
sudo install -o kiosk -g kiosk -m 0755 host/display-agent/jukelight-display-agent /home/kiosk/.local/bin/jukelight-display-agent
sudo install -o kiosk -g kiosk -m 0644 host/display-agent/jukelight-display-agent.service /home/kiosk/.config/systemd/user/jukelight-display-agent.service
sudo systemctl restart jukelight-kiosk
```

The service reads `JUKELIGHT_URL` from `/etc/jukelight/host.env`. Other
settings, with `systemctl --user edit jukelight-display-agent` as the kiosk
user (see [Working over SSH](#working-over-ssh)):

| Variable | Default | Meaning |
| --- | --- | --- |
| `JUKELIGHT_URL` | `http://localhost:8080` | Where Jukelight runs |
| `RETRY_SECONDS` | `5` | Wait between connection attempts |
| `SILENCE_SECONDS` | `60` | Reconnect after this long without data or heartbeat |
| `FAILSAFE_SECONDS` | `600` | Switch the monitor off after this long without Jukelight |
| `POWER_ON_COMMAND` | none | Run after switching the output on, for example to switch a smart plug on |
| `POWER_OFF_COMMAND` | none | Run after switching the output off, also by the fail-safe |

The service also reads `/home/kiosk/.config/jukelight/display-agent.env`
if it exists: private settings, such as the power commands, go there
(step 7).

### 7. Smart plug (optional)

Needed when switching the output doesn't put the monitor to sleep, as with
the reference setup's wireless extender. The display agent then also
switches the monitor's power with a smart plug: after switching the output
off, and after switching it on, so the picture is ready when the monitor
wakes (about 2 seconds on the reference setup, with the extender's logo
briefly). The monitor must switch on by itself when its power returns; the
LG does. When the plug doesn't answer, the agent logs it once and tries
again until it works or the decision changes: first after `RETRY_SECONDS`,
then after twice the previous wait, up to 5 minutes.

The reference setup uses a Shelly Plug M Gen3, switched over the local
network. Any plug or relay that a command can switch works the same way.

**Set up the plug** in its own web page, not in the Shelly app: join the
plug's Wi-Fi (`ShellyPlugMG3-…`) and open `http://192.168.33.1`.

1. Join your Wi-Fi, and give the plug a fixed address (a DHCP reservation
   in your router).
2. Update the firmware.
3. Turn authentication on and set a password (the user is `admin`). Use
   letters and digits only, and make it long instead: in the credentials
   file below, quotes and backslashes need escaping, and a password with a
   `'` was refused there although it worked at curl's prompt.
4. Turn off the cloud, Matter, Bluetooth and the plug's own access point.
   Matter off, because any paired controller could otherwise switch the
   monitor behind Jukelight's back.
5. Make the plug switch on after a power cut, so the monitor comes back.
   The web page doesn't offer this setting; set it from the host (curl asks
   for the password, so it stays out of your shell history):

   ```sh
   PLUG=192.0.2.10   # the plug's address
   curl -s -g --digest -u admin "http://$PLUG/rpc/Switch.SetConfig?id=0&config={\"initial_state\":\"on\"}"; echo
   curl -s --digest -u admin "http://$PLUG/rpc/Switch.GetConfig?id=0"; echo   # "initial_state":"on"
   ```

**Give the agent the plug.** Two files that only the kiosk user can read:
the plug's password in a curl credentials file, and the commands in the
agent's private settings file. The password then appears neither in the
commands nor in the process list. As your own user:

```sh
sudo -u kiosk mkdir -p /home/kiosk/.config/jukelight
sudo -u kiosk install -m 0600 /dev/null /home/kiosk/.config/jukelight/plug.netrc
sudo -u kiosk install -m 0600 /dev/null /home/kiosk/.config/jukelight/display-agent.env
sudoedit -u kiosk /home/kiosk/.config/jukelight/plug.netrc
sudoedit -u kiosk /home/kiosk/.config/jukelight/display-agent.env
```

`plug.netrc`, with the plug's address and password:

```
machine 192.0.2.10 login admin password the-plug-password
```

`display-agent.env`, with the plug's address. Keep the single quotes around
each command: systemd removes them, and the URL's `&` must stay inside the
double quotes.

```sh
POWER_ON_COMMAND='curl -fsS --max-time 5 --digest --netrc-file /home/kiosk/.config/jukelight/plug.netrc "http://192.0.2.10/rpc/Switch.Set?id=0&on=true"'
POWER_OFF_COMMAND='curl -fsS --max-time 5 --digest --netrc-file /home/kiosk/.config/jukelight/plug.netrc "http://192.0.2.10/rpc/Switch.Set?id=0&on=false"'
```

`-f` makes a wrong password a failure, so the agent logs it and retries.
Then reload the agent (see [Working over SSH](#working-over-ssh) for `k`):

```sh
k systemctl --user daemon-reload
k systemctl --user restart jukelight-display-agent
```

## Who logs in where

| Where | Who | How |
| --- | --- | --- |
| tty1 (the monitor) | `kiosk` | Started by `jukelight-kiosk.service`; no prompt, no shell, no key bindings |
| tty2 to tty6 (Ctrl+Alt+F2 to F6 on the host's keyboard) | anyone with an account | The usual login prompt with a password |
| SSH | your own account | As usual; the `kiosk` user cannot log in (no password) |

The kiosk user has a locked password and belongs only to `kiosk`, `video` and
`render`. Check it:

```sh
sudo passwd -S kiosk   # "L" means locked
id kiosk
```

Keep it that way: no password, no `sudo` or `docker` group, no `bindsym`
lines in its sway configuration. Why:

- **No password:** nobody can log in as `kiosk` with a password, on a
  console or over SSH. The kiosk session does not need one: the service
  starts it.
- **No `sudo` or `docker`:** the kiosk runs a browser that shows a web page;
  if that browser were ever compromised, the attacker would hold only the
  kiosk user's rights. `docker` would give them the whole host (see below).
- **`video` and `render` only:** `render` gives Chrome the GPU for hardware
  rendering; `video` gives access to the display devices.
- **No key bindings:** someone at the host's keyboard cannot open a terminal
  or quit sway.

Two more steps close what a locked password leaves open:

```sh
# No login shell: the account cannot be used to log in interactively at all.
sudo usermod -s /usr/sbin/nologin kiosk
# SSH refuses the kiosk user even with a key.
echo 'DenyUsers kiosk' | sudo tee /etc/ssh/sshd_config.d/jukelight.conf
sudo sshd -t && sudo systemctl reload ssh
sudo systemctl restart jukelight-kiosk     # the kiosk still starts
```

- **Login shell:** a locked password stops password logins, but not other
  ways in, such as an SSH key someone adds to `/home/kiosk/.ssh`. Without a
  login shell there is nothing to log in to. The kiosk does not need one:
  the service starts sway directly, and sway's `exec` lines run `sh`
  explicitly.
- **`DenyUsers`:** Ubuntu's SSH server accepts key logins for accounts with
  a locked password. `DenyUsers kiosk` makes it refuse the kiosk user
  whatever the credentials. `sshd -t` checks the configuration before the
  reload, so a typo cannot lock you out.

**Your own account** is effectively root: members of the `docker` group can
start a container with access to the whole host. Protect it like a root
account: SSH keys only (`PasswordAuthentication no` in the SSH server's
configuration) and a password nobody else knows.

## Monitor settings

LG 27UD68-W, in the on-screen menu (Settings, then General):

| Setting | Value | Why |
| --- | --- | --- |
| Automatic Standby | Off | Otherwise the monitor may sleep while music plays |
| Input | The HDMI or DisplayPort input in use | |
| Automatic input switching | Off | Keeps the monitor on the right input while asleep |
| Deep Sleep Mode | Test both values | It saves power, but on DisplayPort it can stop the monitor from waking (see Troubleshooting) |

The LG switches on by itself when its power returns, which a switched power
socket relies on (see Troubleshooting).

## Working over SSH

The kiosk's sway session and user services belong to the kiosk user. From
your own account over SSH:

```sh
# swaymsg: the socket lies in the kiosk user's private runtime folder.
S=$(sudo -u kiosk sh -c 'ls /run/user/$(id -u)/sway-ipc.*.sock')
sw() { sudo -u kiosk SWAYSOCK="$S" swaymsg "$@"; }
sw -p -t get_outputs            # outputs, mode, scale, rotation
sw 'output * power off'         # and 'output * power on'

# The display agent's user service.
k() { sudo -u kiosk XDG_RUNTIME_DIR=/run/user/$(id -u kiosk) "$@"; }
k systemctl --user status jukelight-display-agent
k journalctl --user -u jukelight-display-agent -f   # "display on", "display off"
```

`swaymsg` prints JSON when its output goes into a pipe; `-p` keeps the
readable form.

## Checks after setup

| Check | Command or place | Expected |
| --- | --- | --- |
| Resolution and refresh rate | `sw -p -t get_outputs` | 3840x2160 at 60 Hz on DisplayPort, 30 Hz on HDMI (acceptable) |
| Rotation | the monitor | Jukelight upright; otherwise swap 270 and 90 in the sway configuration |
| Pointer | the monitor | No pointer after a second |
| Hardware acceleration | `chrome://gpu` in the kiosk browser | Hardware accelerated, not software only |
| Monitor sleeps | `sw 'output * power off'` | The monitor goes into standby (not with a wireless extender, see Troubleshooting) |
| Monitor wakes | `sw 'output * power on'` | Picture back in under 5 seconds |
| Page stays current | Play music with the output off, then turn it on | The screen shows the current track at once, without reloading |
| Restart | `sudo reboot` | The monitor shows Jukelight again without any input |
| Smart plug (step 7) | Play music, stop it and wait out the grace period; `sudo -u kiosk curl -s --digest --netrc-file /home/kiosk/.config/jukelight/plug.netrc "http://$PLUG/rpc/Switch.GetStatus?id=0"` (no password to type) | While playing `"output":true`; off `"apower":0.0`; `"source":"HTTP_in"`, never `matter` |
| Agent has the commands | `sudo grep -ao 'POWER_[A-Z_]*=[^[:cntrl:]]*' /proc/$(pgrep -u kiosk -f -o jukelight-display-agent)/environ` | Both commands, each with its full URL (no password: it is in `plug.netrc`) |

## Troubleshooting

**The monitor stays on with an extender's logo.** A wireless HDMI extender's
receiver shows its own logo when the host stops sending (measured with the
Digitus DS-55323: still there after 10 minutes), so the monitor never sleeps.
Switching the output off then only blanks the picture. Switch the monitor's
power with a smart plug instead ([step 7](#7-smart-plug-optional)); this
needs a monitor that switches on by itself when its power returns, as the
LG does.

**The agent's log shows `power command for on failed` (or `off`).** The
plug didn't answer or refused the command. The line before it is curl's own
message: a timeout means a wrong address or the plug is offline; HTTP 401
means a wrong password in `plug.netrc`, or a `machine` there that doesn't
match the address in the commands. The agent retries with growing waits (up
to 5 minutes) and logs `succeeded after retrying` once it works.

**curl reports HTTP 429.** The plug refuses everything for a while after
several refused passwords. The agent's retries slow down to one every 5
minutes, so the lock can expire, but to fix the password quickly, stop the
agent (`k systemctl --user stop jukelight-display-agent`), then pull the
plug out of the wall for 10 seconds (the monitor comes back on by itself)
or wait 10 minutes. Fix the password in `plug.netrc` (letters and digits
only), test once, exactly as the agent does, and start the agent again when
it prints `200`:

```sh
sudo -u kiosk curl -s -o /dev/null -w '%{http_code}\n' --digest --netrc-file /home/kiosk/.config/jukelight/plug.netrc "http://$PLUG/rpc/Switch.GetStatus?id=0"
```

**The agent's log shows `swaymsg failed to switch the display`.** The agent
cannot reach sway, usually because sway was restarted and the agent still
has the old session's socket. The sway configuration (step 3) restarts the
agent with every new session, and the agent stops by itself after 3 failed
switches in a row, so systemd starts it again with the current socket. If
the messages continue, check that the sway configuration has `restart`
(not `start`) in its last line, and restart the agent by hand:
`k systemctl --user restart jukelight-display-agent` (see
[Working over SSH](#working-over-ssh)).

**The monitor does not wake, or the output disappears in sway.** Some monitors
drop the DisplayPort connection in deep sleep. The output then vanishes and
may not come back. Try, in this order:

1. Turn Deep Sleep Mode off in the monitor's menu.
2. Re-enable the output when it reappears (sway configuration).
3. Use HDMI instead (4K at 30 Hz).
4. Use DDC/CI (`ddcutil setvcp D6 ...`) as a last resort; many monitors stop
   answering DDC once asleep, and wireless extenders do not pass it on.

**The monitor shows sway's grey default look with a bar, a pointer and small
text.** The kiosk's sway configuration is missing or unreadable: check
`/home/kiosk/.config/sway/config` (with `sudo`) and restart
`jukelight-kiosk`.

**The kiosk keeps showing the old version after an update.** The page
reloads by itself when it reconnects to a backend with a new kiosk build,
once per build. If it doesn't (for example, an update from before this
feature, or a page that never reconnected), restart the kiosk:
`sudo systemctl restart jukelight-kiosk`.

**The browser shows a black or blank page, or Chrome does not start.** Check
that the container runs (`docker compose ps`), that `JUKELIGHT_URL` in
`/etc/jukelight/host.env` answers (`curl -s "$JUKELIGHT_URL/api/v1/player"`),
and `journalctl -u jukelight-kiosk -b`.

**Animations stutter.** Check `chrome://gpu`: if rendering is in software
mode, the browser is not using the GPU. Check that the kiosk user is in the
`video` and `render` groups.

## Tested versions

Recorded on the reference host.

| Component | Version |
| --- | --- |
| Ubuntu Server | 26.04 LTS |
| Kernel | 7.0.0-38-generic |
| sway | 1.11 |
| Google Chrome | 156.0.8078.79 |
| Docker Engine | 29.9.0 |
| Docker Compose | v5.6.0 |
| Mesa | 26.0.8 (`libgl1-mesa-dri` 26.0.8-1ubuntu0.3) |
| BluOS player firmware | 4.18.18 (Bluesound Node) |
