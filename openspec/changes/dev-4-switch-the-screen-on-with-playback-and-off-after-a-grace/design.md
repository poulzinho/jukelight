## Context

DEV-1 named Display Control as a bounded context but gave it no code. The
player monitor already publishes a `watch` channel of player snapshots that
changes only on material changes (state, connectivity, track, queue), which
is exactly what the display needs. `docs/reference-setup.md` already plans
the host side: sway as compositor, `swaymsg "output * power on|off"` to
switch the monitor, `curl` for a display agent run as a systemd user service
of the kiosk user, and no idle daemon. See proposal.md for the motivation.

## Goals / Non-Goals

**Goals:**

- A display decision that follows the playback with a grace period, tested
  without real time.
- A host agent that is small enough to read in one sitting, needs nothing
  beyond what the reference host already installs, and is tested in CI.
- Each stack layer builds, passes CI and is reviewable on its own; only the
  top layer needs the reference host.

**Non-Goals:**

- Admin settings for the grace period (DEV-6), DDC/CI control, dimming,
  schedules, presence detection (see proposal).

## Decisions

### 1. A pure policy in the domain, timing in the application

The domain gets `DisplayPower` (`On`, `Off`) and a `DisplayPolicy` with the
grace period: given whether the player is active (online and playing or
buffering) and for how long it has been inactive, it returns the decision.
It knows no clock and no timer.

A `DisplayController` in the application layer follows the player
snapshots, remembers when the player stopped being active (a monotonic
`tokio::time::Instant`; at start, the start time), asks the policy, and
sleeps until the grace period ends when the decision is still on. It
publishes the decision in a `watch` channel, which only notifies on change.

- Alternative: the policy reads a clock. Rejected: the policy would need a
  clock port and tests would need to drive it; a duration in, a decision
  out is simpler to test exhaustively.
- Alternative: use the `Clock` port's wall time for the grace period.
  Rejected: wall-clock jumps (NTP, DST) would switch the screen at the wrong
  time; a monotonic instant cannot jump, and tokio's paused time drives it
  in tests.

### 2. The controller follows published snapshots, not events

The controller subscribes to the published snapshot channel and reads
`online` and `state`. Late subscribers get the current snapshot at once,
and only material changes wake it.

- Alternative: the `PlayerEvent` broadcast channel. Rejected: a lagging
  receiver can drop events, and the controller would have to rebuild state
  from events; the latest snapshot is all it needs.

### 3. Display stream in the HTTP adapter

`GET /api/v1/display` is an SSE stream like the player events: the current
decision on connect, every change after, keep-alive comments (15 s by
default), and the stream ends on shutdown (same `router_with_shutdown`
signal). Events are named `display` with the data `on` or `off`, so a shell
client can read them line by line.

- Alternative: JSON data (`{"power":"on"}`). Rejected for now: the agent
  would need a JSON parser; a named event can grow other event names later
  without breaking it.
- Alternative: the agent polls `GET /api/v1/display` as JSON. Rejected:
  polling adds delay to waking the screen, and the spec wants waking in
  seconds.

### 4. Configuration

`DISPLAY_GRACE_MINUTES` (whole minutes from 0 to 1440, default 10) from the
environment; invalid values stop the process with a message naming the
variable, like the other settings. DEV-6 moves it to the database.

### 5. The agent is a POSIX shell script with curl

`host/display-agent/jukelight-display-agent` (POSIX `sh`, `curl`,
`swaymsg`), installed to `~/.local/bin` of the kiosk user, with a systemd
user unit `jukelight-display-agent.service` (`Restart=always`). It reads
the stream with `curl --no-buffer` through a named pipe, applies the first
decision of every connection, and keeps the time of the backend's last data
for the fail-safe. A watchdog subshell ends the connection once nothing
(not even a heartbeat) has arrived for the silence limit. After a
connection that delivered data and lasted at least the retry interval, the
agent reconnects at once; otherwise it waits the retry interval, so a
backend or proxy that closes every connection at once is not hammered.

- Alternative: curl's low-speed limit (`--speed-limit 1 --speed-time N`).
  Rejected: curl averages the speed over several seconds, so a 2-second
  limit took 8 seconds in a measurement; too coarse for a specified limit.

Settings from the environment, with defaults: `JUKELIGHT_URL`
(`http://localhost:8080`), `RETRY_SECONDS` (5), `SILENCE_SECONDS` (60),
`FAILSAFE_SECONDS` (600).

- Alternative: a Rust binary in the workspace. Rejected: it must be built
  for and copied to the host outside Docker; the reference host already has
  `curl` and `sh`, and the logic is a loop of a few dozen lines.
- Alternative: run the agent inside the Docker container. Rejected: it
  needs the kiosk user's sway socket and session; mounting it into the
  container couples the container to one host layout.
- Alternative: let the kiosk page blank itself (black page). Rejected: the
  monitor keeps its backlight on; only switching the output lets it sleep.

### 6. Reaching sway from a user service

A user service does not inherit `SWAYSOCK`. The sway config imports it
into the systemd user environment and starts the agent:
`exec "systemctl --user import-environment SWAYSOCK WAYLAND_DISPLAY; systemctl --user start jukelight-display-agent"`.

- Alternative: start the agent with sway's `exec` directly. Rejected: no
  restart on failure and no journal.
- Alternative: search `/run/user/<uid>/sway-ipc.*.sock` in the script.
  Rejected: guessing breaks with several sessions; importing is explicit.

### 7. Testing the agent in CI

A Rust integration test in the `app` crate starts a local SSE test server,
runs the script with `sh`, short timings and a fake `swaymsg` first in
`PATH` that records its arguments, and checks the recorded calls for every
display-agent scenario. CI also runs `shellcheck` on the script.

- Alternative: bats or a shell test framework. Rejected: one more tool;
  the Rust test runner already starts servers and processes in CI.

### 8. Stack layers

| Layer | Content | Builds on | Needs the reference host |
| --- | --- | --- | --- |
| 1 | OpenSpec change; display policy (domain) and display controller (application) | `main` | no |
| 2 | Display stream, `DISPLAY_GRACE_MINUTES`, wiring, README and `.env.example` | 1 | no |
| 3 | Display agent script and unit, agent tests, `shellcheck` in CI, installation steps in `docs/reference-setup.md` | 2 | no |
| 4 | Installation and acceptance on the ThinkCentre, tested versions, deferred DEV-1 acceptance, sync and archive | 3 | yes |

Each layer is one branch, one pull request and one Linear sub-issue of
DEV-4.

### 9. Tests per layer

| Layer | Tests |
| --- | --- |
| Domain unit | Policy: active, inactive within and past the grace period, offline, zero grace period |
| Application with fake ports | Controller with tokio's paused time: start, play, pause past the grace period, resume within it, offline, only changes published |
| HTTP | Display stream: decision on connect, change, heartbeat, end on shutdown |
| App | Configuration of `DISPLAY_GRACE_MINUTES`; agent scenarios against a test server with a fake `swaymsg` |
| End to end | Manual acceptance on the reference host: wake under 5 s, off after the grace period, agent survives backend restarts, page current after waking |

No kiosk change, so no new kiosk component tests or screenshots.

## Risks / Trade-offs

- [The monitor drops the DisplayPort link in deep sleep and does not come
  back] → the troubleshooting order in `docs/reference-setup.md` (Deep
  Sleep off, re-enable the output, HDMI); the acceptance records which one
  works.
- [Screen off while someone looks at it after pausing] → grace period of 10
  minutes by default, configurable; playback wakes it at once.
- [Agent and backend disagree after a network glitch] → the agent applies
  the first decision of every connection.
- [Backend down for long] → fail-safe off after 10 minutes; the screen
  stays dark until the backend is back.
- [Layer 4 waits for the reference host] → layers 1 to 3 need no host; the
  stack is merged only after layer 4, which carries the sync and archive.

## Migration Plan

No data. Deployments without the agent behave as before: the monitor stays
on, and the new endpoint is unused. Rollback: stop the agent's service and
deploy the previous image.
