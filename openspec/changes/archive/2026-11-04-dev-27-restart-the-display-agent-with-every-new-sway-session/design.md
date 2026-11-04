## Context

The agent is a systemd user service of the kiosk user (`Restart=always`,
`RestartSec=5`). The sway configuration imports `SWAYSOCK` and
`WAYLAND_DISPLAY` into the user manager and starts the agent; `swaymsg`
reads `SWAYSOCK` from the agent's environment, fixed when the agent
started. See proposal.md for the failure seen on the reference host.

## Goals / Non-Goals

**Goals:** a new sway session always gets a working agent; a stale socket
cannot persist unnoticed.

**Non-Goals:** smart plug commands (DEV-18); discovering sway's socket by
other means.

## Decisions

### 1. `restart` in the sway configuration

`exec "systemctl --user import-environment SWAYSOCK WAYLAND_DISPLAY;
systemctl --user restart jukelight-display-agent"`. A restart stops a
running agent and starts it with the newly imported environment; for a
stopped agent it is the same as `start`.

- Alternative: the agent searches `/run/user/<uid>/sway-ipc.*.sock`
  itself. Rejected (as in DEV-4): guessing breaks with several sessions.

### 2. Stop after 3 failed switches in a row

The agent counts consecutive failed `swaymsg` calls; after the third it
logs the reason and exits with status 1; systemd starts it again after 5 s
with the user manager's current environment. A successful call resets the
count. Three, not one: a single failure (sway busy, an output briefly
gone) should not restart the agent.

- Alternative: only the configuration fix. Rejected: a sway restart by
  other means (crash, manual start) would leave the agent stale again,
  silently.

## Risks / Trade-offs

- [sway not running at all: the agent restarts every few switches] →
  systemd's restart limit and the log make it visible; the screen cannot
  be switched without sway anyway.

## Tests

Agent tests in `crates/app/tests/display_agent.rs` with a fake `swaymsg`
that fails: three failed decisions end the agent with status 1; a failure
followed by a success keeps it running.
