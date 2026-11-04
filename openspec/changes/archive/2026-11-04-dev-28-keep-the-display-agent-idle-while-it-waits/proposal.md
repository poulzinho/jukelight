# DEV-28: Keep the display agent idle while it waits

## Why

The display agent uses about 1.2 % of one CPU core all the time, even though
it mostly waits for the backend's decision stream: on the reference host
48 min of CPU in 2 days 20 hours, and 1.5 s in about 2 minutes. The likely
cause is its watchdog, which wakes up every second and starts three
processes each time (`sleep`, `cat` and `date`), about 260,000 a day. On a
small, always-on host this is wasted energy and noise in the process table.
Part of roadmap step 3 (display power).

## What Changes

- The watchdog sleeps until the silence limit could next be reached,
  instead of waking up every second. With the default limit (60 s) and the
  backend's heartbeat (every 15 s), it wakes up about once a minute.
- The agent notes the time of new data once per non-empty line, instead of
  twice per line including the blank lines that end each event.
- No change to what the agent does: the silence limit, the fail-safe, the
  reconnect and the switches behave as before.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `display-agent`: a new requirement that the agent stays idle while the
  connection is open and quiet.

## Non-goals

- Rewriting the agent in another language, or replacing `curl`.
- Changing the backend's heartbeat interval.
- The settings and their defaults (`SILENCE_SECONDS`, `RETRY_SECONDS`,
  `FAILSAFE_SECONDS`) stay as they are.

## Impact

- `host/display-agent/jukelight-display-agent` and its tests in
  `crates/app/tests/display_agent.rs`.
- Reference host: the agent script is reinstalled; no configuration change.
