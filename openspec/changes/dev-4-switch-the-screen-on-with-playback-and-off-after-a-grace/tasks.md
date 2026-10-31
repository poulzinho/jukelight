## 1. Layer 1 (DEV-14): display policy and controller

- [x] 1.1 Write failing domain tests for the policy ("Playback starts", "Buffering", "Paused within the grace period", "Paused past the grace period", "Stopped", "Player offline", "No grace period"), then add `DisplayPower` and `DisplayPolicy` until they pass
- [x] 1.2 Write failing application tests for the display controller with tokio's paused time and the fake device ("Playback resumes within the grace period", "Start with nothing playing", "Start while music plays", "Track change while playing", "Pause and resume within the grace period"), then implement `DisplayController` following the published player snapshots until they pass
- [x] 1.3 Verify the layer: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, the architecture test, and kiosk lint, typecheck and tests; open the layer's pull request and verify CI is green
- [x] 1.4 Review feedback (PR #8): word the display controller test helper's comment for any grace period, not a fixed 10 minutes

## 2. Layer 2 (DEV-15): display stream and configuration

- [x] 2.1 Write failing HTTP tests for the display stream ("Connect", "Decision changes", "Heartbeat", "Shutdown"), then add `GET /api/v1/display` until they pass
- [x] 2.2 Write failing configuration tests for `DISPLAY_GRACE_MINUTES` (default 10, 0 to 1440, invalid values name the variable), then implement it and wire the display controller into `app`; check with the fake device that `/api/v1/display` sends `on` and, with a grace period of 0 and the `paused` scenario, `off`
- [x] 2.3 Add `DISPLAY_GRACE_MINUTES` to `.env.example` and the README configuration table
- [x] 2.4 Verify the layer as in 1.3; open the layer's pull request on top of layer 1 and verify CI is green

## 3. Layer 3 (DEV-16): display agent

- [x] 3.1 Write failing agent tests in `crates/app/tests/` that run the script with `sh` against a local SSE test server, short timings and a fake `swaymsg` that records its arguments, for every display-agent scenario ("Decision on", "Decision off", "Reconnect with the same decision", "Backend restarts", "Silent connection", "Backend unreachable", "Backend back after the fail-safe")
- [x] 3.2 Write `host/display-agent/jukelight-display-agent` (POSIX `sh`, `curl`, `swaymsg`; `JUKELIGHT_URL`, `RETRY_SECONDS`, `SILENCE_SECONDS`, `FAILSAFE_SECONDS`) until the tests pass, and the systemd user unit `jukelight-display-agent.service`
- [x] 3.3 Add `shellcheck` of the script to CI
- [x] 3.4 Replace the placeholder in `docs/reference-setup.md` ("5. Display agent") with the installation steps: copy the script and unit, the sway `exec` line that imports `SWAYSOCK` and starts the service, and how to check it (`systemctl --user status`, `journalctl --user -u jukelight-display-agent`)
- [x] 3.5 Verify the layer as in 1.3 plus `shellcheck`; open the layer's pull request on top of layer 2 and verify CI is green
- [x] 3.6 Review feedback (PR #11): write the timestamp files through a rename and let the watchdog skip an empty read, so a concurrent truncated write cannot end the watchdog under dash
- [x] 3.7 Review feedback (PR #11): give the right reason for the timestamp files (the background watchdog reads them), not a subshell around the stream reader
- [x] 3.8 CI failure (PR #11): the agent tests hung on GitHub's runner while stopping the agent; stop its whole process group with SIGKILL (`kill -KILL -- -<pgid>`) and wait at most 5 seconds

## 4. Layer 4 (DEV-17): reference host acceptance

- [x] 4.1 On the reference host, follow `docs/reference-setup.md` (Docker, sway, Chrome kiosk, agent); record findings that change the document and update it: the kiosk session as `jukelight-kiosk.service` instead of the getty auto-login (design 7a), who logs in where, the SSH forms of the `swaymsg` and `systemctl --user` checks, Chrome starting before Jukelight answers, DisplayPort or HDMI, Deep Sleep Mode, outputs reappearing
- [x] 4.2 Acceptance on the reference host and record it on DEV-4: playback switches the output on (picture back; measure the time); pause and stop switch it off after the grace period (checked with a short grace period and with the default); the agent reconnects after `docker compose restart` and applies the current decision; the fail-safe switches the output off when the container is stopped; the page shows the current track at once after waking. The monitor itself does not go into standby behind the reference host's wireless extender (its receiver shows a logo): record that as a finding; DEV-18 switches the monitor's power with a smart plug
- [ ] 4.3 Run the deferred DEV-1 acceptance on the reference host (playing, paused, stopped, offline, cast session, 4K landscape and portrait) and record it on DEV-1
- [x] 4.4 Fill in "Tested versions" in `docs/reference-setup.md` (kernel, sway, Chrome, Docker, Mesa, player firmware)
- [ ] 4.5 Verify the layer as in 1.3; open the layer's pull request on top of layer 3; after approval, sync the delta specs, archive the change on this layer and verify CI is green before merging
