# Jukelight

Jukelight shows what your BluOS player is playing on a wall-mounted screen:
artwork, title, artist, album, progress, audio quality and what's up next.

It only reads from the player; it never controls playback.

> Status: early. The current version follows a BluOS player and shows its
> playback in a browser. Music Assistant support, switching the screen on and
> off, and an admin app are planned.

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
| `JUKELIGHT_DEVICE` | no | | `fake:<scenario>` runs without a player (see below) |
| `MA_URL` | no | | Music Assistant's address, e.g. `http://192.168.1.50:8095` |
| `MA_TOKEN` | with `MA_URL` | | A long-lived Music Assistant token |

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

Playback to a group of speakers is not recognised yet: Music Assistant then
sends to the player over AirPlay, and Jukelight shows "AirPlay" without
the track details.

### Try it without a player

A fake player with scripted scenarios is built in:

```sh
JUKELIGHT_DEVICE=fake:playing-with-queue docker compose up --build
```

Scenarios: `playing-with-queue`, `cast-without-queue`, `long-title`,
`no-artwork`, `hi-res`, `album-colours`, `paused`, `stopped`, `offline`.

## Reference setup

Jukelight is built and tested on a mini PC with Ubuntu Server driving a 4K
monitor in kiosk mode. [docs/reference-setup.md](docs/reference-setup.md)
describes that setup: hardware, packages, the kiosk session and the monitor
settings. Other setups may work but are not tested.

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
  adapter-fake/      scripted fake player
  adapter-artwork/   artwork proxy with opaque ids
  adapter-http/      REST and SSE API, embedded kiosk
  app/               configuration and wiring (the `jukelight` binary)
web/kiosk/           the kiosk screen (React, Tailwind)
openspec/            specifications and changes
```

### Privacy

Test fixtures are recorded from real players but contain example values only
(`192.168.1.x`, `aa:bb:cc:dd:ee:ff`); a test enforces this. Record new
responses into a `fixtures/raw/` folder (git-ignored) and replace personal
values before committing.

## Licence

MIT, see [LICENSE](LICENSE).
