## 1. Music Assistant's group sessions (stack layer 1, DEV-12: Music Assistant adapter, arbitration)

- [x] 1.1 Copy the recordings into `crates/adapter-music-assistant/tests/fixtures/raw/` (git-ignored) and commit scrubbed fixtures for a group playing, a group paused and a phone's AirPlay (example names, `192.168.1.x`, `aa:bb:cc:dd:ee:ff`-style ids only); verify the existing "no personal data in fixtures" test passes on them
- [x] 1.2 Write failing fixture tests "Music Assistant plays to a group", "A member of another player's group", "AirPlay from a phone" and "No answer yet", then claim AirPlay origins from the cached `players/get` answer (at most 5 s old); verify with `cargo test -p jukelight-adapter-music-assistant`
- [x] 1.3 Write failing provider tests "AirPlay session" (only `players/get` is polled) and "Claimed" for a group, then poll during unclaimed AirPlay sessions and tick `updates` when the answer changes; verify with the provider tests
- [x] 1.4 Write failing fixture tests "The player leads the group" and "The player is a member", then read the queue named by `active_source`; verify with the mapping tests
- [x] 1.5 Write the failing application test "AirPlay while the provider is down", then make it pass; verify with `cargo test -p jukelight-application`
- [ ] 1.6 Verify the layer: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, kiosk lint, typecheck and tests; open the pull request and verify CI is green

## 2. Speakers, outputs and names (stack layer 2, DEV-44: domain, adapters, application, API, configuration)

- [ ] 2.1 Write failing domain tests, then add `Speaker`, `SpeakerKind`, `Output`, `OutputKind`, `Content.speakers` and `DeviceSnapshot.outputs`; verify with `cargo test -p jukelight-domain`
- [ ] 2.2 Write failing fixture tests "Recorded group", "A speaker dropped out", "No format for a speaker" and "Playing alone", then report the group's speakers with their kinds and formats (`players/all`, `audio_processing.outputs`); verify with the mapping tests
- [ ] 2.3 Write failing BluOS fixture tests "Subwoofer on" and "Subwoofer off" (scrubbed `/SyncStatus` from the recordings), then report the subwoofer; verify with `cargo test -p jukelight-adapter-bluos`
- [ ] 2.4 Write failing application tests "Group with an output speaker and a subwoofer", "Nothing configured", "Renamed speaker", "Not renamed" and "Invalid names", then merge outputs and apply names; read `SPEAKER_NAMES` and `PLAYER_OUTPUT_NAME` in `app`; `.env.example`, `README.md`, `docs/reference-setup.md` (with `<maker>-mark` in the logos table); verify with the application and app tests
- [ ] 2.5 Write failing HTTP tests "Recorded group", "Playing alone", "Mark supplied" and "No mark", then publish `speakers`, `outputs` and `device.markUrl`; kiosk API types; fake scenarios for a group (with a dropped speaker, many speakers, long names), AirPlay, Bluetooth, radio and Music Assistant unavailable; verify with the HTTP tests and by reading `/api/v1/player` for each scenario
- [ ] 2.6 Verify the layer (as 1.6); open the pull request and verify CI is green

## 3. The diagram on the kiosk (stack layer 3, DEV-45: kiosk, reference host)

- [ ] 3.1 Write failing tests for the layout function (direction, rows, give-way order, fitting with measured text), then the layout; verify with the kiosk tests
- [ ] 3.2 Write failing component tests for every "Signal diagram", "Streamer card" and "Line colours" scenario, then `SignalDiagram` with its nodes, lines, ports, the streamer card and the PCM icon, replacing `SignalPath`; verify with the kiosk tests
- [ ] 3.3 Write failing tests for "Pulse along the diagram" (delays along the tree, nothing between pulses, reduced motion, paused), then the pulse per segment; verify with the kiosk tests and Playwright
- [ ] 3.4 Write failing tests for "Playing screen", "Up next" and "Steady portrait layout" (two columns, top-aligned, one-line titles), then the layout; verify with the kiosk tests
- [ ] 3.5 Write failing e2e checks for every fake scenario in landscape and portrait (nothing past the screen, under the footer, into the queue's column or over a logo) with screenshots, then fix what they find; review the screenshots against the mocks
- [ ] 3.6 Verify the layer (as 1.6); open the pull request and verify CI is green
- [ ] 3.7 Acceptance on the reference host: a Music Assistant group (playing, a member dropping out, paused), a phone's AirPlay, Qobuz Connect, Spotify, the optical input, the subwoofer off and on, your names in `.env`; portrait from across the room; an hour's CPU against about 62 s per hour; record the results on DEV-12 and DEV-13
