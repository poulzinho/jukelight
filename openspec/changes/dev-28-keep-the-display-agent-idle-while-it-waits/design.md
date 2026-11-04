## Context

The agent (`host/display-agent/jukelight-display-agent`, POSIX sh) reads
the backend's decision stream through `curl` and a FIFO. A watchdog in a
background subshell enforces the silence limit: it reads the time of the
last data from a file and kills `curl` when the limit has passed. It runs
as `while sleep 1; do last=$(cat …); … $(date +%s) …`: three new processes
every second. Each line from the stream also calls `stamp` twice (`date`
and `mv` each), including the blank line that ends every event; the
backend sends a heartbeat every 15 s. On Ubuntu `/bin/sh` is dash, which
has neither `read -t` nor a built-in clock, so waiting and reading the time
both need external commands.

## Goals / Non-Goals

**Goals:** clearly under 10 s of CPU per hour on the reference host (now
about 45 s); the same behaviour for the silence limit, fail-safe, reconnect
and switches; still POSIX sh, still passing under bash and dash.

**Non-Goals:** another language or HTTP client; a different heartbeat.

## Decisions

### 1. The watchdog sleeps until the next possible deadline

Each round, the watchdog reads the last data time, computes how long until
`last + silence`, and kills `curl` if that is zero or less; otherwise it
sleeps exactly that long (at least 1 s). Data that arrives meanwhile moves
the deadline, which the next round sees. The limit is enforced as
precisely as now, with one wake-up per heartbeat-free stretch instead of
one per second: with the defaults about one per 45 to 60 s.

The sleep runs in the background and the watchdog waits for it, with a
trap that stops it when the watchdog is stopped. Otherwise each ended
connection would leave a sleep of up to the silence limit behind; while
the backend is unreachable that is a new one at every retry (found during
apply: 8 left over after 8 s with a retry of 1 s). Once a sleep has ended,
the watchdog forgets its process id, so the trap never kills a process
that has since been given the same id (from the review).

- Alternative: keep the loop, but check every `silence / 4` seconds.
  Rejected: still periodic, and up to a quarter of the limit late.
- Alternative: let `curl` end a silent transfer (`--speed-limit 1
  --speed-time <silence>`). Rejected: curl compares the average rate with
  the limit in bytes per second, and a heartbeat of a few bytes every 15 s
  is under 1 byte per second, so curl would end healthy connections.
- Alternative: `read -t` in the main loop instead of a watchdog. Rejected:
  not POSIX; dash doesn't have it.

### 2. One time stamp per non-empty line

`follow` skips blank lines (they only end an event) and, for each other
line, reads the time once and writes both files (`contact` and `seen`)
from that value, each through a rename as before. That halves the
processes per heartbeat and drops those for blank lines.

- Alternative: a single file for both times. Rejected: `seen` restarts with
  each connection and `contact` does not; merging them changes the
  fail-safe.

## Risks / Trade-offs

- [The deadline arithmetic is off by a second] → the existing scenario
  "Silent connection" (limit 2 s, killed within 3 s) and the new
  "Silence limit still applies" catch it.
- [An empty read of the time file during a rename] → as now, the watchdog
  retries after 1 s.

- [The agent tests time out on macOS when run in parallel] → macOS scans
  each new executable on its first run (12 new scripts: 1.1 s, then
  0.02 s). The counting `sleep` wrapper is installed only in the test that
  needs it. "Silent connection" already failed about 1 run in 4 in
  parallel on macOS before this change; CI (Linux) is the reference.

## Tests

Agent tests in `crates/app/tests/display_agent.rs` (the agent is a host
adapter, tested as a process against a fake backend):

- "Quiet connection": a counting wrapper for `sleep` on `PATH`, ahead of
  the real one; with `SILENCE_SECONDS=60` and an open, silent connection,
  the agent starts `sleep` at most twice in 5 s (now about five times).
- "Silence limit still applies": the existing silent-connection test,
  unchanged.
- All other agent tests unchanged, under bash and dash.
- On the reference host: CPU time from
  `systemctl --user show jukelight-display-agent -p CPUUsageNSec` over at
  least an hour, before and after.
