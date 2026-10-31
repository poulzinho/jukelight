# DEV-20: Show the player's audio inputs as inputs

## Why

When the player plays one of its audio inputs (on the reference setup a CD
transport on the optical input), the kiosk treats it like a track: the badge
"Capture" (BluOS's internal id), BluOS's small icon blown up to cover size,
the time since the signal started shown as a track time, and the
player's old play queue as "Up next". And because the player cannot tell
whether a signal comes in (it reports "playing" even with the CD transport
switched off), the screen never switches off while the input is selected.
The player does notice when a signal starts (measured: `secs` restarts and
`state` goes from connecting to stream when the CD starts).
Recognising the music itself was studied and not pursued (DEV-21). This
change belongs to roadmap step 4 (finished kiosk), with a correction to step
3 (display power).

## What Changes

- **Audio inputs as their own origin:** the BluOS adapter recognises an
  audio input (optical, analog, HDMI ARC; Bluetooth stays as today) and
  reports its kind, its name (as set in the BluOS app) and when its signal
  started (none while it waits for a signal), with no track data and no
  queue.
- **Snapshot and API:** the snapshot carries the input (kind, name, start of
  its signal); the session label is the input's kind ("Optical input");
  `/api/v1/player` gains `input`.
- **Bug fix:** the player's play queue is no longer read or shown for
  inputs.
- **Display power:** an input counts as playing only once its signal
  started, and then for a limited time (`INPUT_DISPLAY_MINUTES`, default
  90, covering a full CD); after that, or while it waits for a signal, the
  normal grace period applies. A new start of the signal starts the time
  again.
- **Kiosk input screen:** the clock, a slowly turning disc, the input's
  name, the kind as the badge, and "for N min" since the signal started
  ("waiting for signal" before); no cover, track time, progress or queue.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `device-bluos`: audio inputs as an origin; no queue for inputs.
- `session-arbitration`: the session label of an input origin.
- `playback-monitoring`: the input in the snapshot; change detection for
  inputs.
- `display-control`: the time limit for inputs.
- `published-api`: `input` in the current snapshot.
- `kiosk-display`: the input screen.

## Non-goals

- Recognising what plays on an input (DEV-21, not pursued).
- Bluetooth: it may carry track information; it stays as today until
  measured.
- Setting the time limit in the admin app (DEV-6); here it comes from the
  environment.
- Volume display, input switching from the kiosk.

## Impact

- **Code:** `domain` (input origin and snapshot field, change detection,
  display policy), `application` (arbiter label, display controller with a
  clock), `adapter-bluos` (input mapping, no `/Playlist` for inputs),
  `adapter-fake` (an input scenario), `adapter-http` (`input` in the DTO),
  `app` (`INPUT_DISPLAY_MINUTES`), kiosk (input screen).
- **API:** `/api/v1/player` gains `input` (null for other sources); existing
  fields unchanged.
- **Configuration:** new optional `INPUT_DISPLAY_MINUTES`; README and
  `.env.example`.
- **Fixtures:** recorded `/Status` of the optical input (waiting for a
  signal and streaming), scrubbed.
