# DEV-3: Show Music Assistant playback as a session

Linear issue: DEV-3. Roadmap step 2 of 4 (Music Assistant session).
Delivered as a stack of pull requests, one Linear sub-issue per layer.

## Why

When Music Assistant drives the BluOS player (Navidrome or Qobuz through
Music Assistant), the player itself only knows a stream. The kiosk then shows
"Music Assistant" with no artwork, progress, quality or queue, for a large
part of what is played at home. Music Assistant knows all of it: the track,
the per-track position, the source format, the queue and colours taken from
the cover.

DEV-1 also left three gaps between the code and the OpenSpec artifacts,
which this change closes (see What Changes).

## What Changes

- New session provider for Music Assistant:
  - It claims the device's origin when the player streams from the Music
    Assistant server and the stream belongs to this player, decided from
    configuration alone.
  - While it drives the player, it supplies the track, artwork, per-track
    position, source quality (with conversion and loudness normalization
    noted), up next and the album's colours.
  - It names the music provider, so the session label reads for example
    "Music Assistant · Qobuz".
- The snapshot carries the album's colours (background and accent) when a
  source provides them, and the published API exposes them.
- The kiosk tints its background and accent colour with those colours.
- A session provider's own updates refresh the snapshot, even when the
  player reports nothing new (BluOS does not notice Music Assistant track
  changes).
- Configuration: `MA_URL` and `MA_TOKEN` from the environment; without them,
  Music Assistant streams keep showing as "Stream".
- Gaps left by DEV-1:
  - `published-api`: the requirement that live streams end on shutdown, which
    the code already implements and tests.
  - Design record of the Rust 1.93 pin for local builds, CI and Docker.
  - Correct font package name: `@fontsource-variable/inter`.

## Capabilities

### New Capabilities

- `session-music-assistant`: how Music Assistant is recognised as the
  session driving the player, and how its playback is translated into
  content.

### Modified Capabilities

- `playback-monitoring`: the player snapshot also carries the album's
  colours when known.
- `session-arbitration`: the session label names the music provider, and a
  provider's own updates refresh the snapshot.
- `published-api`: the snapshot JSON includes the colours; live streams end
  on shutdown.
- `kiosk-display`: the screen is tinted with the album's colours.

## Non-goals

- Storing the Music Assistant connection or token in the database, and the
  admin app (DEV-6).
- Push updates over Music Assistant's WebSocket (polling is used; push can
  replace it later without changing behaviour).
- Spotify through Music Assistant (needs Spotify Premium).
- Colours for sources other than Music Assistant (BluOS provides none).
- Display power (DEV-4), sharper CDN artwork (DEV-5).

## Impact

- New crate `crates/adapter-music-assistant` (HTTP client, polling, mapping),
  with recorded and scrubbed Music Assistant responses as test fixtures.
- `domain`: colours on the content; `application`: provider updates drive
  the player monitor, a shared session provider contract suite.
- `adapter-http`: `palette` in the snapshot JSON.
- `app`: `MA_URL` and `MA_TOKEN` configuration and wiring.
- `web/kiosk`: colour variables from the snapshot.
- Docs: README configuration table, `.env.example`.
- External system: reads from a Music Assistant server on the LAN with a
  long-lived token (a dedicated non-admin Music Assistant user is
  recommended). Nothing is written to it.
- Done when: the kiosk shows Music Assistant playback (Navidrome and Qobuz
  via Music Assistant) with track, artwork, progress, source quality, up next
  and colours; an unreachable Music Assistant degrades as specified; CI is
  green on every layer; acceptance against the real player is recorded on
  DEV-3.
