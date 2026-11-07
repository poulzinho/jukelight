# DEV-26: Reload the kiosk when Jukelight is updated

## Why

After an update (`docker compose up --build -d`), the kiosk page in Chrome
keeps running the old version: its live stream reconnects to the new
backend, but the page itself is never reloaded. During the DEV-20
acceptance the new input screen didn't appear until the kiosk was restarted
by hand (`sudo systemctl restart jukelight-kiosk`), a step that is easy to
forget and needs SSH. Part of roadmap step 4 (finished kiosk).

## What Changes

- **Backend:** it identifies the kiosk build it serves by a short hash of
  the embedded `index.html` (which names every built asset by its content
  hash, so it changes with any kiosk change). It writes the identifier
  into the page it serves, as `<meta name="jukelight-build">`, and sends it
  as a `build` event on the player event stream, right after the first
  snapshot of every connection.
- **Kiosk:** when the announced build differs from the one in its own page,
  it reloads once. A same build changes nothing. It reloads at most once
  per announced build, so a stale cache can't make it reload in a loop.
- **Reference setup:** the manual kiosk restart after an update goes; it
  stays in Troubleshooting as a fallback.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-api`: the kiosk page carries its build identifier, and the
  player event stream announces the backend's.
- `kiosk-display`: the kiosk reloads itself when the announced build is
  different.

## Non-goals

- Caching headers: the page is already `no-cache`, the hashed assets
  immutable.
- Scheduled reloads, or reloads for any reason other than a new build.
- Reloading the kiosk when only the backend changes (the page is the same,
  so there is nothing to reload).

## Impact

- `crates/adapter-http` (the page and the event stream) and its HTTP tests.
- `web/kiosk/src/usePlayer.ts` and its tests.
- `docs/reference-setup.md`.
- No change to the display agent, the device adapters or the domain.
- The display stream is unchanged, so the display agent needs nothing.
