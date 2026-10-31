# DEV-4: Switch the screen on with playback and off after a grace period

## Why

The wall screen stays on all day, even when nothing plays: it wastes power,
wears the panel and lights the room at night. It should wake by itself when
music starts and go dark some time after it stops, without anyone touching
it. This is roadmap step 3 (display power), and the first time Jukelight
runs on the reference host (the ThinkCentre with the LG monitor).

## What Changes

- **Display Control** (new bounded context): a pure policy turns the
  player's playback state and connectivity into a display decision, on or
  off. Playing or buffering: on at once. Paused, stopped or offline: on for
  a grace period (default 10 minutes), then off. A display controller in
  the application layer applies the policy over time and publishes only
  changes.
- **Display API**: `GET /api/v1/display`, a Server-Sent Events stream that
  sends the current decision on connect, then every change, with
  heartbeats.
- **Configuration**: `DISPLAY_GRACE_MINUTES` (default 10) from the
  environment until the admin app (DEV-6) stores it.
- **Display agent** (new, runs on the host, not in Docker): a small script
  run as a systemd user service in the kiosk's sway session. It follows the
  display stream and runs `swaymsg "output * power on"` or `power off`,
  reconnects when the backend restarts, and switches the output off when
  the backend has been unreachable for a while (fail-safe).
- **Reference host**: install and check the agent on the ThinkCentre, fill
  in the "Tested versions" table of `docs/reference-setup.md`, and run the
  deferred DEV-1 acceptance there.

## Capabilities

### New Capabilities

- `display-control`: when the display should be on or off, from the
  playback state and connectivity, with the grace period.
- `display-agent`: how the host follows the display decisions and switches
  the monitor, including reconnection and the fail-safe.

### Modified Capabilities

- `published-api`: adds the display decision stream
  (`GET /api/v1/display`).

## Non-goals

- Changing the grace period in the admin app (DEV-6); here it comes from
  the environment.
- Monitor control over DDC/CI beyond the documented fallback in
  `docs/reference-setup.md`.
- Dimming, schedules (night mode) or presence detection.
- Kiosk visual changes; the kiosk page keeps running while the output is
  off and is current when it comes back.

## Impact

- **Code**: `domain` (display policy and decision), `application` (display
  controller), `adapter-http` (display stream), `app` (configuration and
  wiring), new `host/display-agent/` (script and systemd unit, outside the
  Rust workspace and the Docker image).
- **API**: new endpoint `GET /api/v1/display`; existing endpoints unchanged.
- **Configuration**: new optional `DISPLAY_GRACE_MINUTES`; README and
  `.env.example`.
- **Docs**: `docs/reference-setup.md` (agent installation, tested
  versions).
- **Host**: the reference host needs sway, `curl` and the agent's user
  service; the last stack layer needs the ThinkCentre for acceptance.
