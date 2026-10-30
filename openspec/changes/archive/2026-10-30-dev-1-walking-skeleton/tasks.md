## 1. Workspace and tooling

- [x] 1.1 Create the Cargo workspace with the crates `domain`, `application`, `adapter-bluos`, `adapter-fake`, `adapter-artwork`, `adapter-http` and `app`, workspace lints (clippy pedantic where sensible, `unwrap_used` denied outside tests) and `rustfmt.toml`; verify `cargo build` and `cargo clippy --all-targets -- -D warnings` pass on the empty crates
- [x] 1.2 Add `deny.toml` (licences compatible with MIT, advisories, sources) and an architecture test that reads `cargo metadata` and fails when `domain` reaches an IO crate (`tokio`, `reqwest`, `hyper`, `axum`, `quick-xml`), when `application` depends on an adapter, or when adapters depend on each other; verify by temporarily adding `tokio` to `domain` and seeing the test fail
- [x] 1.3 Scaffold `web/kiosk` with Vite, React, TypeScript (strict), Tailwind CSS v4, Inter (`@fontsource-variable/inter`), oxlint (the Vite template's linter), Vitest and Testing Library; verify `npm run lint`, `npm run typecheck`, `npm test` and `npm run build` pass
- [x] 1.4 Add `.env.example` with example values only (`BLUOS_HOST=192.168.1.40`, `HTTP_PORT=8080`, `LOG_LEVEL=info`) and verify `.env` is git-ignored with `git check-ignore .env`

## 2. Domain: playback model

- [x] 2.1 Write failing tests for the player snapshot and queue view scenarios (playing track, source without duration, session with upcoming tracks, session without a queue), then implement the types (`PlayerSnapshot`, `PlaybackState`, `Track`, `Position`, `QueueView`, `Origin`, `Content`) until they pass
- [x] 2.2 Write failing tests for per-track position prediction (playing, paused, elapsed not measured per track), then implement `Position` prediction until they pass
- [x] 2.3 Write failing tests for audio quality labels (lossless format "FLAC 24/96", bitrate "160 kbps") and every Hi-Res rule scenario (16/44.1, 24/44.1, 24/192, AAC 24/48, bitrate only), then implement `AudioQuality` and the Hi-Res rule until they pass
- [x] 2.4 Write failing tests for change detection (new track, normal progress without event, seek over 2 s, pause, player goes offline, queue change, player back online), then implement the snapshot diff until they pass

## 3. Application: ports and services

- [x] 3.1 Define the ports `DeviceMonitor`, `SessionProvider` (`claims` + content), `ArtworkSource` and `Clock`, plus in-memory fakes for tests; verify the crate builds and `domain` remains free of IO dependencies (task 1.2 check)
- [x] 3.2 Write failing tests for every session-arbitration scenario (provider drives the device, state comes from the device, provider down, native service, unclaimed stream, claim while the provider is down) using fake providers, then implement `SessionArbiter` until they pass
- [x] 3.3 Write failing tests for `PlayerMonitor`: a new device snapshot produces an arbitrated player snapshot on the `watch` channel and the derived events on the `broadcast` channel, and late subscribers receive the current snapshot; then implement it until they pass

## 4. BluOS device monitor

- [x] 4.1 Record `/Status`, `/SyncStatus` and `/Playlist` from the real player into `fixtures/raw/` (git-ignored) for Qobuz Connect, native Qobuz with a queue, Spotify Connect, a Music Assistant stream, paused and stopped; replace personal values with example values and commit them to `crates/adapter-bluos/tests/fixtures/`; verify `git status` shows no file from `fixtures/raw/`
- [x] 4.2 Write the personal-data scan test (no IP outside 192.168.1.0/24, no MAC other than aa:bb:cc:dd:ee:ff in any fixture) and verify it passes on the committed fixtures and fails on a deliberately broken copy
- [x] 4.3 Write failing fixture tests for identity, playback state mapping, origin (stream, cast service, native service), the player's own content (full metadata, escaped characters, relative artwork path, bitrate instead of format) and position (native session vs stream origin), then implement the pure parser until they pass
- [x] 4.4 Write failing fixture tests for the native queue and for the stale playlist during a cast session, then implement playlist parsing and the native-only rule until they pass
- [x] 4.5 Write the shared `DeviceMonitor` contract suite (timeout without changes keeps the device online, request spacing of at least 1 s, offline on failure, back online after success, backoff capped at 30 s) and run it against the BluOS adapter using a local test server that replays fixtures; implement the long-poll loop until the suite passes

## 5. Fake device

- [x] 5.1 Implement `adapter-fake` with the scenarios playing-with-queue, cast-without-queue, long-title, no-artwork, hi-res, paused, stopped and offline, selectable by name; verify the shared `DeviceMonitor` contract suite passes against it

## 6. Artwork

- [x] 6.1 Write failing tests for opaque ids (same source gives the same id, the id reveals nothing of the source address), then implement the id registry until they pass
- [x] 6.2 Write failing tests against a local test server for the proxy: issued id returns the image with its content type, unknown id returns 404 without an outgoing request, a redirecting source returns the target image, a failing source returns 502, and a second request does not contact the source again; then implement the proxy and bounded cache until they pass

## 7. HTTP API

- [x] 7.1 Write failing tests for `GET /api/v1/player` (playing track, nothing playing, JSON field names and shapes from the published-api spec), then implement the DTO mapping and handler until they pass
- [x] 7.2 Write failing tests for `GET /api/v1/player/events` (first event is the current snapshot, a track change produces an event, normal progress produces none, a comment line arrives within 30 s), then implement the SSE handler until they pass
- [x] 7.3 Write a failing test that `/api/v1/player` responses contain neither the configured player address nor BluOS field names, then make it pass
- [x] 7.4 Wire `GET /api/v1/art/<id>` to the artwork proxy and verify the HTTP tests for 200, 404 and 502 pass
- [x] 7.5 Embed `web/kiosk/dist` with `rust-embed` (with a build script that creates a placeholder page when the folder is missing) and serve it at `/` with an `index.html` fallback; verify a test request to `/` returns the kiosk HTML

## 8. Composition and configuration

- [x] 8.1 Write failing tests for configuration parsing (`BLUOS_HOST` required for the BluOS device, defaults for `BLUOS_PORT`, `HTTP_PORT` and `LOG_LEVEL`, `JUKELIGHT_DEVICE=fake:<scenario>`, clear error on invalid values), then implement it until they pass
- [x] 8.2 Wire the `app` crate (device monitor, player monitor, artwork, HTTP server, `tracing` logs without addresses at `info` level) and verify `JUKELIGHT_DEVICE=fake:playing-with-queue cargo run -p jukelight` serves a snapshot at `http://localhost:8080/api/v1/player`

## 9. Kiosk screen

- [x] 9.1 Define the theme tokens (vmin type scale, spacing, colour variables) and the Inter font in the Tailwind theme; verify a token test or snapshot shows the expected CSS variables in the build
- [x] 9.2 Write failing component tests for the `usePlayer` hook (applies the first SSE snapshot, marks the connection lost on error, applies the current snapshot after reconnecting, corrects `observedAt` by the server clock offset), then implement it until they pass
- [x] 9.3 Write failing component tests for the playing screen (track with all data, no position hides progress, progress advances while playing and stops while paused, missing artwork shows initials), then implement `Artwork`, `TrackInfo` and `Progress` until they pass
- [x] 9.4 Write failing component tests for badges and up next (session and quality badges, "HI-RES" badge only for Hi-Res, up next only when available, 4 entries in landscape and 3 in portrait), then implement `Badges` and `UpNext` until they pass
- [x] 9.5 Write failing component tests for the paused, idle and offline screens (pause overlay and dimming, idle clock with "<player name> · nothing playing", corner indicator while disconnected or offline), then implement `PauseOverlay`, `IdleScreen` and `ConnectionIndicator` until they pass
- [x] 9.6 Implement landscape and portrait layouts and reduced motion (opacity fades and `transform` only); verify with the Playwright screenshots in task 10.2

## 10. Delivery

- [x] 10.1 Write the multi-stage `Dockerfile` (Node build of the kiosk, Rust build with `cargo-chef`, slim non-root runtime with a health check on `/api/v1/player`) and `docker-compose.yml` (build from source, port 8080, `.env`); verify `docker compose up --build` with `JUKELIGHT_DEVICE=fake:playing-with-queue` serves the kiosk at `http://localhost:8080`
- [x] 10.2 Add Playwright screenshot tests that start the binary with fake scenarios (playing with queue, cast without queue, long title, no artwork, hi-res, paused, stopped, offline) at 3840x2160 and 2160x3840 with reduced motion; verify they pass and the screenshots match the kiosk-display scenarios
- [x] 10.3 Add the GitHub Actions workflow with the Rust job (fmt, clippy `-D warnings`, test, `cargo deny check`), web job (lint, typecheck, Vitest), end-to-end job (task 10.2) and Docker build job, publishing nothing; verify all jobs pass on the pull request
- [x] 10.4 Add a README quickstart (clone, copy `.env.example` to `.env`, `docker compose up --build`) and link `docs/reference-setup.md`; verify the steps work on a clean checkout

## 11. Acceptance on the reference host

- [x] 11.1 On a development machine (the reference host follows with DEV-3, display power), run `docker compose up --build -d` with the real player's address in `.env`, open the kiosk in a browser, and verify playing, paused, stopped and offline states for native Qobuz, Qobuz Connect, Spotify Connect and a Music Assistant stream (shown as "Stream"); record the results on DEV-1
- [x] 11.2 Verify that `git status` on that machine shows neither `.env` nor any raw recording, then run `openspec validate dev-1-walking-skeleton --strict`
