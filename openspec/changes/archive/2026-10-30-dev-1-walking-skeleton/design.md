## Context

The repository has no product code yet. A throwaway prototype (TypeScript,
kept outside the repository) proved the data flow against a real BluOS
player and Music Assistant, and produced the measurements this design relies
on: BluOS reports `stream` while playing cast sessions, keeps a stale
`/Playlist` during cast sessions, counts `secs` across a whole Music
Assistant stream, and a long-poll only returns early on real changes. See
proposal.md for scope and motivation, and the specs for the required
behaviour.

Constraints that shape the approach:

- The reference host is an Intel mini PC (UHD 630 graphics) driving a 4K
  monitor through Chrome in kiosk mode; it also runs the container.
- The code is open source: no personal data in code, fixtures or docs.
- Later steps add a session provider (Music Assistant), display power, and
  SQLite settings. This step must leave clean seams for them without
  building them.

## Goals / Non-Goals

**Goals:**

- A Rust workspace whose crate graph enforces the dependency rule.
- The device/session split in place, with the arbiter working for zero
  providers, so step 2 only adds an adapter and its configuration.
- A frontend toolchain that already builds, tests and embeds the kiosk, so
  later steps only add screens.
- Tests on every layer, driven by the spec scenarios.

**Non-Goals:**

- Persistence of any kind (settings, artwork on disk).
- Session providers, display power, admin, discovery (see proposal).
- Published images or multi-architecture builds.

## Decisions

### 1. Workspace layout and the dependency rule

```
crates/
  domain/             playback model, change detection, Hi-Res rule
  application/        ports (DeviceMonitor, SessionProvider, ArtworkSource,
                      Clock) and services (SessionArbiter, PlayerMonitor)
  adapter-bluos/      DeviceMonitor for BluOS
  adapter-fake/       DeviceMonitor with scripted scenarios
  adapter-artwork/    artwork id registry, proxy fetches, cache
  adapter-http/       axum: REST, SSE, DTOs, /api/v1/art, embedded kiosk
  app/                configuration, wiring, process lifecycle
web/
  kiosk/              React app
```

`domain` depends on no async runtime, HTTP or XML crate. `application`
depends only on `domain`, plus `futures` for port signatures and tokio's
synchronisation primitives (`watch`, `broadcast`) for its services, never on
an IO client. Adapters depend on `application` and `domain`, never on each
other; `app` depends on everything. An architecture test reads `cargo
metadata` and fails the build when this rule is broken. `cargo deny` checks
licences, advisories and sources.

- Alternative: enforce the rule with `cargo deny` bans. Rejected: bans work
  by listing every crate allowed to pull in a banned crate, which breaks
  whenever a new dependency brings tokio along; the test checks the rule
  itself.

- Alternative: one crate with modules. Rejected: the boundary would rely on
  discipline; separate crates make a wrong dependency a compile error.
- Alternative: finer crates per bounded context now. Rejected: Display
  Control and Administration have no code in this step.

### 2. Device monitor and session providers instead of merged sources

Two ports replace the original `PlayerSource`:

- `DeviceMonitor`: a stream of device snapshots (connectivity, playback
  state, origin, the device's own content).
- `SessionProvider`: `claims(&Origin) -> bool` (pure, offline) and the
  current content.

`SessionArbiter` (application) combines the latest device snapshot with the
first provider that claims its origin and produces the domain snapshot
(spec: session-arbitration). In this step the provider list is empty; the
arbiter is still tested with fake providers.

- Alternative: merge two snapshots field by field (original design).
  Rejected: measured data showed per-field priority gives wrong positions
  in Music Assistant flow mode and needs many special cases.
- Alternative: use Music Assistant as the only source. Rejected: it would
  make the whole display depend on one optional system.

### 3. Snapshots in, events out

Adapters always deliver complete snapshots. The domain compares consecutive
snapshots and derives events (spec: playback-monitoring), with a 2-second
tolerance for position drift. This keeps change detection in one tested
place and lets the long-poll timeout refresh the position without noise.

- Alternative: adapters emit events. Rejected: every adapter would
  reimplement change detection, and sources differ (long-poll vs push).

### 4. Runtime and data flow

```
 DeviceMonitor task --watch<DeviceSnapshot>--> PlayerMonitor task
                                                 | arbiter + diff
                                                 v
                              watch<PlayerSnapshot> + broadcast<Event>
                                                 |
                                   adapter-http (REST, SSE clients)
```

Each adapter runs as a tokio task. `tokio::sync::watch` carries "latest
value" state (late subscribers get the current snapshot at once, which the
SSE spec requires); a `broadcast` channel carries events for later
consumers such as Display Control.

- Alternative: an actor framework. Rejected: three tasks and two channels do
  not need one.

### 5. BluOS adapter

- HTTP with `reqwest` (timeouts: long-poll 100 s + 10 s margin, other
  requests 5 s), XML with `quick-xml` into a flat field map. Parsing is a
  pure function from response text to a typed status, so fixture tests need
  no network.
- `/SyncStatus` once per (re)connection; `/Playlist` only when `pid`
  changes and only for native, non-cast service origins.
- Backoff 1, 2, 4 … 30 seconds; the player is offline after the first
  failed request.
- Recorded responses live in `crates/adapter-bluos/tests/fixtures/`.
  Recordings are made into `fixtures/raw/` (git-ignored) and committed only
  after replacing personal values. A test scans all fixture files and fails
  on an IP outside 192.168.1.0/24 or a MAC other than aa:bb:cc:dd:ee:ff.

### 6. Artwork

`adapter-artwork` keeps an in-memory registry from opaque id (first 16 hex
characters of the SHA-256 of the source URL) to source URL, and a bounded
LRU cache of images. `reqwest` (rustls, system certificate store) follows
redirects. Besides `http(s)` sources the proxy accepts base64 `data:` URLs,
which the fake device uses for generated covers. The registry is filled when
snapshots are built, so the proxy can only fetch issued ids; it remembers at
most 4096 ids, forgetting the oldest first.

- Alternative: on-disk cache (original design). Deferred: unnecessary until
  restarts become frequent; ids are re-issued when the kiosk refetches the
  snapshot after a reconnect.
- Alternative: pass source URLs to the browser. Rejected: leaks addresses
  and turns the proxy into an open fetcher.

### 7. HTTP adapter

axum with DTOs separate from domain types (camelCase JSON). SSE uses axum's
SSE support with keep-alive comments every 15 seconds. The SSE handler
subscribes to the snapshot `watch`, sends the current value first, then
forwards changes that the domain classified as more than normal progress.
On shutdown (SIGTERM or Ctrl-C), the live streams are ended first; otherwise a
graceful shutdown would wait forever for kiosks that never disconnect (found
during acceptance: `docker stop` hung until it killed the container). The
kiosk build (`web/kiosk/dist`) is embedded with `rust-embed` and served
at `/`, with an `index.html` fallback.

- Alternative: serve the kiosk from disk or a separate web server. Rejected:
  one binary is simpler to deploy and cannot drift from the API version.

### 8. Kiosk

- React, TypeScript (strict), Vite, Tailwind CSS v4, `@fontsource/inter`.
- Theme tokens in one CSS file: type scale in `vmin` (title 7.2, artist 4.4,
  album 3.2, times 2.4, badges 2.2, up next 2.6/2.2, clock 16), spacing in
  `vmin`/`vw`, colours from CSS variables (`--bg`, `--accent`, `--text`).
- Components: `Artwork`, `TrackInfo`, `Progress`, `Badges`, `UpNext`,
  `PauseOverlay`, `IdleScreen`, `ConnectionIndicator`.
- One `usePlayer` hook owns the `EventSource`, the latest snapshot and the
  connection state; no state library.
- Progress uses `transform: scaleX()`, recomputed every 250 ms with a short
  linear transform transition (simpler to test than a
  `requestAnimationFrame` loop, and just as smooth); pause dimming is an
  overlay's opacity; state changes are short opacity fades, disabled under
  `prefers-reduced-motion`.
- Layout follows `portrait:`/`landscape:` variants.
- Clock offset: the kiosk compares the server's `Date` header from
  `GET /api/v1/player` with its own clock and corrects `observedAt`, so a
  browser on another device with a skewed clock still shows correct
  progress.
- Development: the Vite dev server proxies `/api` to the backend.

### 9. Fake device

`adapter-fake` implements `DeviceMonitor` from scripted scenarios (playing
with queue, cast without queue, long title, no artwork, paused, stopped,
offline). Covers are generated SVG images passed as `data:` URLs, so no
image server is needed. `app` selects it with
`JUKELIGHT_DEVICE=fake:<scenario>`; the default is the BluOS adapter with
`BLUOS_HOST`.

### 10. Configuration

Environment variables only: `BLUOS_HOST` (required for the BluOS adapter),
`BLUOS_PORT` (default 11000), `HTTP_PORT` (default 8080), `LOG_LEVEL`
(default `info`), `JUKELIGHT_DEVICE` (optional). Invalid configuration stops
the process at start with a clear message. Settings move to SQLite in
step 4.

### 11. Container, compose and CI

- Dockerfile: stage 1 builds the kiosk with Node; stage 2 builds the Rust
  binary with `cargo-chef` for dependency caching and embeds the kiosk;
  stage 3 is a slim Debian runtime with only the binary and CA
  certificates, running as a non-root user, with a health check on
  `GET /api/v1/player`.
- `docker-compose.yml`: builds from source, publishes port 8080, reads
  `.env`. Bridge networking is enough for this step (outgoing HTTP to the
  player); host networking is only needed for discovery later.
- CI (GitHub Actions): Rust job (`cargo fmt --check`, `cargo clippy -D
  warnings`, `cargo test`, `cargo deny check`), web job (lint, typecheck,
  Vitest), end-to-end job (build the binary, run it with a fake scenario,
  Playwright screenshots in landscape and portrait), and a Docker build
  job. Nothing is published.

### 12. Tests per layer

| Layer | Tests |
| --- | --- |
| Domain unit | Snapshot diff and events, position prediction and 2 s drift, quality labels, Hi-Res rule |
| Application, fake ports | Arbiter (claimed, unreachable, unclaimed, offline claims), PlayerMonitor emits snapshots and events |
| Adapter fixtures | BluOS parsing for Qobuz Connect, native Qobuz, Spotify Connect, Music Assistant stream, paused, stopped; `/Playlist`; personal-data scan |
| Port contract suite | Shared `DeviceMonitor` suite run against the BluOS adapter (local test server replaying fixtures) and the fake adapter: reconnect, offline, request spacing |
| HTTP | `/api/v1/player` shape, SSE first event and heartbeat, no vendor data, `/api/v1/art` 200/404/502, kiosk at `/` |
| Kiosk components | Vitest + Testing Library per screen state: playing, no position, no artwork, up next, Hi-Res, paused, idle, offline |
| Kiosk screenshots | Playwright against the binary with fake scenarios, 3840x2160 and 2160x3840, reduced motion |
| End to end | Manual on the reference host against the real player |

## Risks / Trade-offs

- [BluOS firmware changes response fields] → Tolerant flat parsing, fixture
  tests, and missing fields become absent values instead of errors.
- [Kiosk build must exist before `cargo build` embeds it] → `rust-embed`
  points at `web/kiosk/dist`; a build script creates an empty folder with a
  placeholder page when it is missing, so Rust-only builds and tests work.
- [In-memory artwork registry is lost on restart] → The kiosk refetches the
  snapshot after reconnecting and receives freshly issued ids.
- [Chrome on the reference host renders slowly at 4K] → Only opacity and
  transform animations, no CSS blur in this step; checked on the reference
  host.
- [Clock skew between the server and a browser on another device] → Clock
  offset from the `Date` header (decision 8).
- [Music Assistant streams show little until step 2] → Accepted; the screen
  shows what BluOS reports, labelled "Stream".

## Migration Plan

New project; nothing to migrate. Deployment on the reference host is
`docker compose up --build -d`; rollback is `docker compose down` and
starting the previous checkout.
