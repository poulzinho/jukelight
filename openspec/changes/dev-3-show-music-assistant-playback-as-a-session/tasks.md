## 1. Layer 1 (DEV-8): domain and application

- [x] 1.1 Write failing domain tests for content with album colours and a music service (snapshot scenarios "Source with album colours" and "Source without album colours"), then add `Palette`, `Rgb`, `Content::palette` and `Content::service` until they pass; update the BluOS and fake adapters to set both to `None`
- [x] 1.2 Write a failing arbiter test for "Provider names the music service" (label "Example Provider · Example Service") and keep the existing arbiter tests passing; implement the label rule
- [x] 1.3 Extend the `SessionProvider` port with `follow` and `updates` and the fake provider with controllable updates; write failing monitor tests for "Track change seen only by the provider" and "Provider told about the device", then implement the provider-driven rebuild in `PlayerMonitor::run`
- [x] 1.4 Add the shared session provider contract suite to `application::testing::contract` (claims decided offline, no content while unreachable, content after recovery, an update tick when content changes) and verify it passes against the fake provider
- [x] 1.5 Verify the layer: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, the architecture test, and kiosk lint, typecheck and tests; open the layer's pull request and verify CI is green

## 2. Layer 2 (DEV-9): Music Assistant adapter

- [x] 2.1 Record `players/get`, `player_queues/get`, `player_queues/items` and `providers` from the real Music Assistant into `crates/adapter-music-assistant/tests/fixtures/raw/` (git-ignored) while playing a Navidrome track and a Qobuz track through Music Assistant; commit scrubbed copies and a personal-data scan test (no IP outside 192.168.1.0/24, no MAC other than aa:bb:cc:dd:ee:ff, no token) that fails on a deliberately broken copy
- [x] 2.2 Write failing tests for every "Claiming the session" scenario, then implement the offline claim rule
- [x] 2.3 Write failing fixture tests for "Track and artwork", "Per-track position" (queue position, clock skew), "Source quality", "Music provider name" and "Album colours", then implement the pure mapping until they pass
- [x] 2.4 Write failing tests against a local test server replaying the fixtures for "Following the player" (claimed, not claimed, track change), "Up next" (following tracks, unchanged queue fetched once), "Unreachable Music Assistant" (server down, wrong token, back) and "Token handling", then implement the polling task and `updates()` until they pass
- [x] 2.5 Run the session provider contract suite against the Music Assistant adapter and verify it passes; verify the architecture test still passes (no adapter depends on another)
- [x] 2.6 Verify the layer as in 1.5; open the layer's pull request on top of layer 1 and verify CI is green

## 3. Layer 3 (DEV-10): API, configuration and wiring

- [ ] 3.1 Write failing HTTP tests for "Album colours" and "No album colours" in `/api/v1/player`, then add `palette` to the DTO until they pass; confirm the existing shutdown tests cover "Live streams end on shutdown"
- [ ] 3.2 Write failing configuration tests (`MA_URL` and `MA_TOKEN` both or neither, `MA_URL` must be http(s), no provider without them), then implement them and wire the provider (`follow`, `updates`, its polling task) into `app`; verify with the real Music Assistant that `/api/v1/player` shows the Music Assistant track
- [ ] 3.3 Add a fake scenario with album colours, update `.env.example` and the README configuration table with `MA_URL` and `MA_TOKEN` (example values only, recommend a non-admin Music Assistant user)
- [ ] 3.4 Verify the layer as in 1.5; open the layer's pull request on top of layer 2 and verify CI is green

## 4. Layer 4 (DEV-11): kiosk colours and acceptance

- [ ] 4.1 Write failing component tests for "Colours known", "Colours unknown" and "Colours change with the track", then set `--bg` and `--accent` from `palette` until they pass
- [ ] 4.2 Extend the Playwright suite with the colours scenario (landscape and portrait), checking the page background and the progress fill colour; review the screenshots
- [ ] 4.3 Acceptance against the real player with `docker compose up --build -d` and the Music Assistant settings in `.env`: Navidrome and Qobuz via Music Assistant (playing, paused, track change, up next, quality detail, colours, label "Music Assistant · …"), Music Assistant unreachable (degraded label), and a non-Music-Assistant session unchanged; record the results on DEV-3
- [ ] 4.4 Verify the layer as in 1.5; open the layer's pull request on top of layer 3 and verify CI is green
