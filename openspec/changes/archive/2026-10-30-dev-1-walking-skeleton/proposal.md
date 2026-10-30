# DEV-1: Walking skeleton

Linear issue: DEV-1. Roadmap step 1 of 4 (walking skeleton).

## Why

Jukelight has a design and a throwaway prototype, but no product code. The
prototype proved the data is good enough for a nice screen; now we need the
thinnest end-to-end slice on the real architecture and stack: a BluOS
player's playback, shown on the kiosk screen, running in Docker on the
reference host. Every later step (Music Assistant, display power, admin)
plugs into this slice, so its boundaries must be right from the start.

## What Changes

- New Rust workspace with the hexagonal layout: domain, application (ports
  and services), adapters, and a composition root.
- Playback model in the domain: snapshot of what a player is doing, change
  detection that produces events, per-track position, audio quality and the
  Hi-Res rule.
- BluOS device monitor: follows a BluOS player by long-polling, reports its
  playback state, where the audio comes from (origin), and the content the
  player itself knows.
- Session arbitration with the device/session split. In this step no session
  provider exists yet, so the screen always shows the device's own content.
  Music Assistant streams therefore show only what BluOS reports ("Music
  Assistant"); real Music Assistant content follows in step 2.
- Artwork proxy: the browser only receives opaque artwork ids issued by the
  backend, never a player or CDN address.
- Published API (v1): current snapshot as JSON and live updates over SSE.
- Kiosk screen on the real frontend stack (React, Vite, TypeScript, Tailwind
  with Jukelight's theme tokens, bundled Inter): playing, paused, idle and
  offline states, in landscape and portrait.
- A fake device with scripted scenarios, for kiosk work and tests without a
  player.
- Delivery: Dockerfile (frontend embedded in the binary), a basic
  `docker-compose.yml`, `.env.example`, and CI that checks and builds
  everything (no image publishing).

Changes compared with the original design (`jukelight.md`), based on
measurements on real devices:

- BluOS `stream` state means "playing", not "radio"; the source comes from
  the service fields and the stream URL.
- One snapshot per player is built by arbitration between the device and a
  claiming session provider, instead of merging two sources field by field.
- The kiosk supports portrait screens as well as landscape.

## Capabilities

### New Capabilities

- `playback-monitoring`: the player snapshot, change detection and events,
  per-track position, audio quality and the Hi-Res rule.
- `device-bluos`: how a BluOS player is followed and translated into a
  device snapshot (state, origin, the player's own content, native queue).
- `session-arbitration`: which content the snapshot uses, depending on
  whether a session provider claims the device's origin.
- `artwork`: opaque artwork ids and the artwork proxy.
- `published-api`: the versioned JSON and SSE API the kiosk consumes.
- `kiosk-display`: the screen states and layouts of the kiosk.

### Modified Capabilities

None: there are no existing specs.

## Non-goals

- Music Assistant content, queue and quality (step 2).
- Larger CDN artwork versions and the tiny backdrop image (later).
- Display power: screen on/off, the display SSE and the host agent (step 3).
- Admin app, SQLite settings, token storage, custom Hi-Res label upload
  (step 4). The Hi-Res label shows as a plain text badge.
- Player discovery (LSDP). The player address comes from configuration.
- Animations beyond simple fades; the full animation set comes with the
  finished kiosk (step 4).
- Any playback control.

## Impact

- New code: `crates/` (Rust workspace), `web/kiosk/` (React app),
  `Dockerfile`, `docker-compose.yml`, `.env.example`, CI workflow.
- New dependencies: Rust (tokio, axum, reqwest, quick-xml, serde, tracing,
  thiserror, rust-embed), frontend (React, Vite, TypeScript, Tailwind CSS,
  @fontsource/inter, Vitest, Testing Library, Playwright).
- External systems: reads from a BluOS player on the LAN (HTTP, port 11000)
  and from artwork CDNs through the proxy. Nothing is written to them.
- Configuration: environment variables only in this step (player address,
  HTTP port, log level).
- Done when: CI is green, and `docker compose up --build` runs Jukelight
  against a real BluOS player and shows its playback in a browser. This step
  is accepted on a development machine; the first run on the reference host
  follows with step 3 (display power), which needs that host anyway.
