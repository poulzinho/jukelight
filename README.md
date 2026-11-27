# Jukelight

Jukelight shows what your BluOS player is playing on a wall-mounted screen:
artwork, title, artists, album and progress; a diagram of the signal from
the source through the player to the speakers, with the audio quality at
each step; and what's up next, grouped by work or artist.

It follows the player itself and, if you use it, Music Assistant, including
Music Assistant's speaker groups. It turns the screen on when music starts
and off a while after it stops. It only reads; it never controls playback.

> Status: in daily use on one reference setup (see below). Settings are
> read from `.env`; an admin app is planned.

## Quickstart

You need Docker with the Compose plugin, on a machine in the same network as
your BluOS player.

```sh
git clone https://github.com/poulzinho/jukelight.git
cd jukelight
cp .env.example .env      # then set BLUOS_HOST to your player's address
docker compose up --build -d
```

Open `http://<this machine>:8080` in a browser. Make the window full screen
for the kiosk view.

The first build compiles the backend and the kiosk and takes a few minutes.

### Configuration

Set these in `.env` (see `.env.example`):

| Variable | Required | Default | Meaning |
| --- | --- | --- | --- |
| `BLUOS_HOST` | yes | | IP address or host name of your BluOS player |
| `BLUOS_PORT` | no | `11000` | The player's API port |
| `HTTP_PORT` | no | `8080` | Port for the kiosk and the API |
| `LOG_LEVEL` | no | `info` | `trace`, `debug`, `info`, `warn` or `error` |
| `DISPLAY_GRACE_MINUTES` | no | `10` | Minutes the screen stays on after playback stops (0 to 1440). Needs the display agent on the host, see [docs/reference-setup.md](docs/reference-setup.md) |
| `INPUT_DISPLAY_MINUTES` | no | `90` | Minutes an audio input (for example a CD player) keeps the screen on after its signal started (0 to 1440); the player cannot tell when the signal stops |
| `JUKELIGHT_DEVICE` | no | | `fake:<scenario>` runs without a player (see below) |
| `MA_URL` | no | | Music Assistant's address, e.g. `http://192.168.1.50:8095` |
| `MA_TOKEN` | with `MA_URL` | | A long-lived Music Assistant token |
| `BRAND_ASSETS_DIR` | no | `/brand-assets` | The folder with your brand logos; `docker-compose.yml` mounts `./brand-assets` there (see below) |
| `SPEAKER_NAMES` | no | | Your names for speakers, by the name Music Assistant shows: a JSON object such as `{"Example Portable Speaker": "Kitchen"}`. An invalid value only logs a warning |
| `PLAYER_OUTPUT_NAME` | no | | The speaker plugged into the player's own output, shown under the player |

`.env` is git-ignored. Never commit it.

### Music Assistant

When Music Assistant plays to your player, the player only sees one long
stream. With `MA_URL` and `MA_TOKEN` set, Jukelight asks Music Assistant
what is playing and shows the track, the queue, the source quality and the
album colours. Set both or neither: without them, Music Assistant streams
show as "Stream".

Create the token in Music Assistant under your user's profile, as a
long-lived token. Use a dedicated Music Assistant user without admin
rights: Jukelight only reads.

When Music Assistant plays to a group of speakers, it sends to the player
over AirPlay or Sendspin. Jukelight then asks Music Assistant whether the
AirPlay session is its own, shows the group's track and queue, and draws
every speaker of the group with the format it receives. A phone's AirPlay
stays "AirPlay". Name the speakers with `SPEAKER_NAMES`.

### Try it without a player

A fake player with scripted scenarios is built in:

```sh
JUKELIGHT_DEVICE=fake:playing-with-queue docker compose up --build
```

Scenarios: `playing-with-queue`, `cast-without-queue`, `long-title`,
`no-album`, `no-artwork`, `hi-res`, `album-colours`, `optical-input`,
`paused`, `stopped`, `offline`, `music-assistant`,
`music-assistant-unavailable`, `long-signal`, `upsampled`,
`unknown-quality`, `group`, `group-dropped`, `group-many`,
`group-long-names`, `airplay`, `bluetooth`, `radio`, `long-artists`,
`classical`.

### Switching the screen

Jukelight decides when the screen should be on: while music plays, and for
`DISPLAY_GRACE_MINUTES` after it stops. A small display agent on the host
follows that decision and switches the monitor, through its power saving or
a smart plug. Without the agent, the screen simply stays on.
[docs/reference-setup.md](docs/reference-setup.md) shows how to install it.

## Reference setup

Jukelight is built and tested on a mini PC with Ubuntu Server driving a 4K
monitor in kiosk mode. [docs/reference-setup.md](docs/reference-setup.md)
describes that setup: hardware, packages, the kiosk session, the display
agent and an optional smart plug, and the monitor settings. Other setups may
work but are not tested.

## Development

Requirements: Rust (stable), Node.js 24.

```sh
# Backend: format, lint, test
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace

# Kiosk: lint, typecheck, test
cd web/kiosk
npm ci
npm run lint && npm run typecheck && npm test

# Run against the fake player while working on the kiosk
JUKELIGHT_DEVICE=fake:playing-with-queue cargo run --bin jukelight   # API on :8080
npm run dev                                                          # kiosk with hot reload

# End-to-end screenshots (landscape and portrait, 4K)
npm run build && (cd ../.. && cargo build --bin jukelight)
npx playwright install chromium
npm run e2e
```

### Layout

```
crates/
  domain/            playback model, change detection, Hi-Res rule (no IO)
  application/       ports and services: session arbitration, player monitor
  adapter-bluos/     follows a BluOS player (long-polling)
  adapter-music-assistant/
                     asks Music Assistant what it plays, including groups
  adapter-fake/      scripted fake player
  adapter-artwork/   artwork proxy with opaque ids
  adapter-http/      REST and SSE API, embedded kiosk
  app/               configuration and wiring (the `jukelight` binary)
web/kiosk/           the kiosk screen (React, Tailwind)
host/display-agent/  switches the monitor on the host
openspec/            specifications and changes
```

### Brand logos

Jukelight ships no brand logos: the music services', Music Assistant's, the
player maker's and the Hi-Res Audio logo are their owners' trademarks. The
kiosk shows an icon and the name instead. To show logos, put
your own copies into `brand-assets/` next to `.env`, named by brand
(`qobuz.svg`, `spotify.svg`, `music-assistant.svg`, `bluesound.png`,
`hi-res.png`, …), in the variant for dark backgrounds. The folder is
git-ignored; Jukelight picks up added or replaced files within a few
seconds. [docs/reference-setup.md](docs/reference-setup.md) lists the
brands and where to get their official logos.

### Privacy

Test fixtures are recorded from real players but contain example values only
(`192.168.1.x`, `aa:bb:cc:dd:ee:ff`); a test enforces this. Record new
responses into a `fixtures/raw/` folder (git-ignored) and replace personal
values before committing.

## Contributing and security

See [CONTRIBUTING.md](CONTRIBUTING.md) for how changes are made, and
[SECURITY.md](SECURITY.md) for how to report a vulnerability.

## Trademarks

BluOS and Bluesound, Qobuz, Spotify, TIDAL, Deezer, Apple Music, AirPlay,
Plex, Jellyfin, Navidrome, Subsonic, Music Assistant, Bluetooth, Hi-Res
Audio and the other product names mentioned here are trademarks of their
respective owners.
Jukelight is an independent project, not affiliated with or endorsed by
any of them; it names their products only to say what it works with. For
that reason the repository contains none of their logos: whoever runs
Jukelight supplies them (see [Brand logos](#brand-logos)).

## Licence

MIT, see [LICENSE](LICENSE). The kiosk bundles the Figtree typeface, under
the SIL Open Font License 1.1.
