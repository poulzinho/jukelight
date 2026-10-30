## Context

DEV-1 built the device/session split with no session providers: the arbiter
uses a claiming provider's content, but nothing claims yet, and the player
monitor only rebuilds the snapshot when the device reports. See proposal.md
for the motivation.

Measured facts about Music Assistant (MA) 2.10, from the exploration and the
prototype, that shape the approach:

- Every API call needs a bearer token (401 otherwise). Commands go through
  `POST /api` (`{"command": …, "args": …}`) or the WebSocket `/ws`.
- MA's `player_id` for a Bluesound player is its MAC without separators, and
  the flow stream URL contains it.
- In flow mode BluOS sees one long stream: no etag change on track changes,
  and `secs` counts across tracks. Only MA knows when the track changes.
- The per-track position is `PlayerQueue.elapsed_time`;
  `Player.elapsed_time` is stream-wide and `current_media.elapsed_time` can
  be stale.
- Polling `players/get` and `player_queues/get` every 2 s worked reliably in
  the prototype; push events over `/ws` are untested.

## Goals / Non-Goals

**Goals:**

- A Music Assistant session provider behind the existing `SessionProvider`
  port, with no Music Assistant terms outside its crate.
- Providers can update the snapshot on their own.
- Each stack layer builds, passes CI and is reviewable on its own.

**Non-Goals:**

- WebSocket push, settings in the database, the admin app (see proposal).
- Colour transitions (DEV-7 crossfades colours with opacity layers).

## Decisions

### 1. Poll the HTTP API while claimed

The provider polls `players/get` and `player_queues/get` every 2 s while it
claims the device's origin, and not at all otherwise. Queue entries
(`player_queues/items`) are fetched only when the current item, index or
item count changes; provider names (`providers`) are cached and reloaded
when an unknown instance appears.

- Alternative: WebSocket push (`player_updated`, `queue_updated`,
  `queue_time_updated`). Faster track changes, but untested against this
  server, with reconnection and event ordering to handle. It can replace
  polling later behind the same port, without spec changes.
- Alternative: poll always. Rejected: needless load while other sessions
  play.

### 2. Providers follow the device and publish updates

The port gains two methods; `label`, `claims` and `content` stay:

```rust
pub trait SessionProvider: Send + Sync + 'static {
    fn label(&self) -> &str;
    fn claims(&self, device: &DeviceSnapshot) -> bool;
    fn content(&self) -> Option<Content>;
    /// The current device report, before every rebuild of the snapshot.
    fn follow(&self, device: &DeviceSnapshot);
    /// Ticks whenever `content()` changed.
    fn updates(&self) -> BoxStream<'static, ()>;
}
```

`PlayerMonitor::run` selects over the device stream and the merged provider
updates. On a device report it calls `follow` on every provider, then
rebuilds; on a provider update it rebuilds from the last device report.
`apply` (used by tests) keeps its meaning.

- Alternative: give providers a channel of device snapshots. Rejected: puts
  a tokio channel type into the port.
- Alternative: providers produce whole player snapshots. Rejected: moves
  arbitration out of the application layer.

### 3. Colours and service name in the content

`Content` gains `service: Option<String>` (e.g. "Qobuz") and
`palette: Option<Palette>` with `Palette { background: Rgb, accent: Rgb }`
and `Rgb(u8, u8, u8)`. The arbiter builds the session label as
`"<provider label> · <service>"` when a service is named. BluOS content has
neither, so nothing changes for device sessions.

- Alternative: a dynamic provider label. Rejected: the label identifies the
  provider; the service is per track.

### 4. The Music Assistant adapter

New crate `crates/adapter-music-assistant`:

- `reqwest` (rustls, as in the artwork adapter) and `serde` with typed,
  tolerant response structs (`#[serde(default)]`, unknown fields ignored).
- Claim rule from configuration only: the origin's host equals the
  configured MA URL's host and its path contains `/<player key>/`. The port
  is not compared: MA's stream server port is configurable.
- Mapping as specified: queue item first, current media as fallback; artwork
  from a remotely accessible `http(s)` path, otherwise the image proxy
  (`size=0`; `size=256` for up next); quality detail with conversion and
  "Normalised"; `OpenSubsonic Media Server Library` shortened to "Subsonic".
- Colours: `background_dark` and `on_dark`. Music Assistant derives them
  from the cover as a contrast-checked pair (WCAG AA 4.5:1, Sendspin
  `color@v1`); `primary` and `accent` describe the cover and are not
  adjusted, so either can disappear against the background (seen in the
  recordings: `primary` equals `background_dark` on one cover, `accent` is
  nearly equal on another).
- Position from the queue; when `elapsed_time_last_updated` is more than
  10 s away from the local clock, the observation time is now.
- A background task (`run()`, spawned by `app`) does the polling, with a
  configurable interval for tests. Content changes tick a `watch` channel
  that backs `updates()`.
- The token lives in a newtype whose `Debug` prints `[redacted]`; request
  errors drop their URL (`without_url()`), like the BluOS adapter.

### 5. Configuration

`MA_URL` and `MA_TOKEN` from the environment, both or neither (one without
the other stops the process with a clear message). `MA_URL` must be an
`http(s)` URL. Without them no provider is registered and Music Assistant
streams keep showing as "Stream". The admin app and encrypted storage come
with DEV-6.

### 6. Kiosk colours

The kiosk sets `--bg` and `--accent` on the document from `palette`
(defaults when null). Text keeps `--text`. Colours switch instantly: the
design rule "animate only opacity and transform" excludes colour
transitions; DEV-7 adds a crossfade through opacity layers.

### 7. Rust toolchain pin (record of a DEV-1 decision)

`rust-toolchain.toml` pins Rust 1.93 (rustfmt, clippy) for local builds; CI
installs it with `actions-rust-lang/setup-rust-toolchain`; the Docker build
uses `rust:1.93-slim-trixie`. Reason: CI on the latest stable failed on a
clippy lint that the local toolchain did not have.

- Alternative: always latest stable. Rejected: new lints break unrelated
  pull requests at random.
- Alternative: pin only in CI. Rejected: local and CI results would differ.

Upgrades are deliberate chores that update all three places.

### 8. Font package (correction of DEV-1 artifacts)

The kiosk bundles `@fontsource-variable/inter`, not `@fontsource/inter` as
the DEV-1 proposal and design said. Nothing changes in code.

### 9. Stack layers

| Layer | Content | Builds on |
| --- | --- | --- |
| 1 | OpenSpec change; domain colours and service; arbiter label; provider updates in the monitor; session provider contract suite | `main` |
| 2 | Music Assistant adapter with recorded fixtures | 1 |
| 3 | API `palette`; `MA_URL`/`MA_TOKEN` configuration and wiring; fake scenario with colours; docs | 2 |
| 4 | Kiosk colours; end-to-end screenshots; acceptance | 3 |

Each layer is one branch, one pull request and one Linear sub-issue of
DEV-3.

### 10. Tests per layer

| Layer | Tests |
| --- | --- |
| Domain unit | Colours and service on content; arbiter label with service |
| Application, fake ports | Monitor rebuilds on provider updates; `follow` before rebuild; session provider contract suite (claims offline, no content while unreachable, content after recovery, update tick on change) |
| Adapter fixtures | Local test server replaying recorded MA responses: claim rule, track and artwork, queue position trap, quality detail, up next caching, provider names, palette, 401 and server down, token not in logs |
| Port contract suite | The shared session provider suite against the MA adapter and a fake provider |
| HTTP | `palette` in `/api/v1/player` (present and null); shutdown requirement (existing tests) |
| Kiosk components and screenshots | Colour variables from the snapshot and defaults; Playwright with a fake scenario that has colours, landscape and portrait |
| End to end | Manual acceptance against the real player and Music Assistant (Navidrome and Qobuz via MA) |

## Risks / Trade-offs

- [MA API changes between versions] → tolerant structs, recorded fixtures,
  degraded "Playing from Music Assistant" instead of errors.
- [Track changes appear up to 2 s late] → acceptable for a wall display;
  WebSocket push can remove it later.
- [Clock skew between MA and Jukelight] → observation time falls back to
  now beyond 10 s; the kiosk already corrects for its own clock.
- [Token in an environment variable until DEV-6] → never logged or
  returned; documentation recommends a dedicated non-admin MA user.
- [MA stream on another port or proxy] → the claim compares host and player
  key, not the port.

## Migration Plan

No data to migrate. Existing deployments without `MA_URL`/`MA_TOKEN` behave
as before. Rollback: deploy the previous image.
