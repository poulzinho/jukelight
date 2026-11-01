# DEV-27: Restart the display agent with every new sway session

## Why

The display agent keeps the sway socket of the session it was started in.
The kiosk's sway configuration starts the agent with `systemctl --user
start`, which does nothing when the agent already runs; after sway restarts
(`systemctl restart jukelight-kiosk`, a crash, a configuration change) the
agent kept the old socket, and every switch failed silently for hours
(found during the DEV-20 acceptance: `swaymsg failed to switch the display`
from 18:20 to 21:00, while the backend's decisions were right). This fixes
roadmap step 3 (display power).

## What Changes

- The kiosk's sway configuration restarts the agent instead of starting
  it, so every new sway session gets an agent with its socket
  (`docs/reference-setup.md`).
- The agent stops after 3 switches in a row have failed, so its service
  manager (`Restart=always`) starts it again with the current environment:
  a stale socket can no longer persist unnoticed.
- `docs/reference-setup.md`: a troubleshooting entry for `swaymsg failed`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `display-agent`: the agent stops after repeated failed switches.

## Non-goals

- Smart plug commands (DEV-18).
- Changing how the agent finds sway (it keeps using `SWAYSOCK` from the
  user session).

## Impact

- `host/display-agent/jukelight-display-agent`, its tests in
  `crates/app/tests/display_agent.rs`, `docs/reference-setup.md`.
- Reference host: one line of the kiosk's sway configuration changes
  (`start` → `restart`); the agent script is reinstalled.
