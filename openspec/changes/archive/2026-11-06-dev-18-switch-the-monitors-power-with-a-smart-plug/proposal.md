# DEV-18: Switch the monitor's power with a smart plug

## Why

On the reference host the monitor is reached through a wireless HDMI
extender. When the display agent switches the output off, the extender's
receiver shows its own logo instead of going quiet, so the monitor never
goes into standby (measured: logo still there after 10 minutes). The screen
should really switch off a grace period after the music stops, and back on
when it starts, also on setups like this. Part of roadmap step 3 (display
power).

## What Changes

- **Display agent:** two optional settings, `POWER_ON_COMMAND` and
  `POWER_OFF_COMMAND`. When set, the agent runs them together with the
  output switch: to switch off, the outputs first, then the command; to
  switch on, the outputs first, then the command, so the picture is ready
  when the monitor wakes. The fail-safe runs the off command too. Without
  them, the agent behaves as before.
- **Failing command:** the agent logs it and retries at the retry interval
  until the command succeeds or the decision changes. It doesn't count
  towards "Stop after repeated failed switches", which is for a stale sway
  session.
- **Agent settings for the kiosk user:** the user service also reads an
  optional settings file in the kiosk user's home, readable only by that
  user, for settings that must stay private (such as a plug's address and
  password).
- **Reference setup:** a Shelly Plug M Gen3 for the monitor, switched over
  the local network (digest authentication, no cloud, no Matter); its setup,
  the agent's commands and a troubleshooting entry in
  `docs/reference-setup.md`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `display-agent`: run the configured power commands with every switch and
  with the fail-safe, and retry a failing command.

## Non-goals

- A built-in smart plug client in Jukelight, or plug settings in the admin
  app (could come with DEV-6).
- HDMI-CEC and DDC/CI (neither reaches the monitor through this extender).
- Support for specific plug brands in the agent: the commands are plain
  shell commands, so any plug or relay with a command-line interface works.

## Impact

- `host/display-agent/jukelight-display-agent`, its unit
  `jukelight-display-agent.service` and its tests in
  `crates/app/tests/display_agent.rs`.
- `docs/reference-setup.md`.
- Reference host: the plug, a settings file and a credentials file for the
  kiosk user; the agent and its unit are reinstalled.
- No change to the backend, the API or the kiosk page.
