## 1. Layer 1 (DEV-22): input model and display time limit

- [x] 1.1 Write failing domain tests for the snapshot's input ("Input plays") and change detection ("Input selected again", "Selection time jitter", kind or name changes, input appearing and going away), then add `AudioInput`, `InputKind`, `Origin::Input`, `PlayerSnapshot::input` and `InputChanged` until they pass; existing adapters set `input` to `None`
- [x] 1.2 Write failing domain tests for the display policy's input time limit ("Within the input time limit", "Past the limit and the grace period", a limit of 0), then extend `DisplayPolicy` until they pass
- [x] 1.3 Write a failing arbiter test for "Optical input" (label "Optical input", input carried, no track) and the other kind labels, then implement them
- [x] 1.4 Write failing display controller tests with tokio's paused time and a fixed clock ("Within the input time limit", "Past the limit and the grace period", "Input selected again", "Another source"), then give the controller the `Clock` port and the input limit until they pass; keep the existing controller tests passing
- [ ] 1.5 Verify the layer: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, the architecture test, and kiosk lint, typecheck and tests; open the layer's pull request and verify CI is green
- [x] 1.6 Finding from recording the reference player (task 2.1): `secs` counts from when the signal starts, and `connecting` means waiting for a signal. Rename the input's selection time to `started_at` (optional, none while waiting), count a waiting input as not playing, and adapt the tests ("Signal arrives", "Start time jitter", "Waiting for a signal", "Signal starts again")
- [x] 1.7 Review feedback (PR #14): British spelling for the analogue input's label ("Analogue input"); the identifiers (`InputKind::Analog`, API `analog`) stay
- [x] 1.8 Review feedback (PR #14): the `InputChanged` doc comment and task 2.2 still said "selection time"; they now say the start of the input's signal

## 2. Layer 2 (DEV-23): BluOS audio inputs

- [x] 2.1 Record `/Status` and `/Playlist` from the reference player while the optical input waits for a signal (`connecting`) and while it streams, into `crates/adapter-bluos/tests/fixtures/raw/` (git-ignored); commit scrubbed copies (input name changed to an example name) and keep the personal-data scan passing
- [x] 2.2 Write failing parsing and mapping tests for "Optical input", "Bluetooth input", "Named optical input", "Waiting for a signal" and "Unknown input kind", then implement the input origin, kind, name and the start of its signal, with no content
- [x] 2.3 Write a failing monitor test against the local test player for "Stale playlist while an input plays" (no `/Playlist` request, queue not applicable), then exclude inputs from `is_native`
- [ ] 2.4 Verify the layer as in 1.5; open the layer's pull request on top of layer 1 and verify CI is green

## 3. Layer 3 (DEV-24): API, configuration and fake scenario

- [x] 3.1 Write failing HTTP tests for "Input plays" and "No input", then add `input` to the player DTO until they pass
- [ ] 3.2 Write failing configuration tests for `INPUT_DISPLAY_MINUTES` (default 90, 0 to 1440, invalid values name the variable), then implement it and wire the clock and the input limit into the display controller in `app`
- [ ] 3.3 Write a failing fake device test, then add the `optical-input` scenario (optical, "Example Input", streaming since 23 minutes before the start)
- [ ] 3.4 Add `INPUT_DISPLAY_MINUTES` to `.env.example` and the README configuration table, and the new scenario to the README's list
- [ ] 3.5 Verify the layer as in 1.5; open the layer's pull request on top of layer 2 and verify CI is green

## 4. Layer 4 (DEV-25): kiosk input screen and acceptance

- [ ] 4.1 Write failing component tests for "Optical input" (time, disc, name, badge, "for 23 min", no progress bar or "Up next"), "Just started" ("for 0 min"), "Waiting for a signal" and the idle screen still shown when stopped, then add the input screen until they pass
- [ ] 4.2 Write a failing test for "Reduced motion" (the disc's animation is none), then implement it
- [ ] 4.3 Extend the Playwright suite with the `optical-input` scenario in landscape and portrait; review the screenshots
- [ ] 4.4 Acceptance on the reference host with the CD transport on the optical input, and record it on DEV-20: the input screen (name from BluOS, badge, "for N min", turning disc, no queue); with a short `INPUT_DISPLAY_MINUTES` the screen goes off after the limit plus the grace period, a selected input without a signal goes off after the grace period, and starting the CD again wakes it; streaming sources unchanged
- [ ] 4.5 Verify the layer as in 1.5; open the layer's pull request on top of layer 3; after approval, sync the delta specs, archive the change on this layer and verify CI is green before merging
