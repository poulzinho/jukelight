## Context

Measured on the reference player (a Node 2i with a CD transport on its
optical input; details on DEV-20): while an input plays, `/Status` reports
`serviceType` AudioInputs, `inputTypeIndex` spdif-1, `title1` = the input's
name as set in the BluOS app, and `state`: `connecting` with `secs` 0
while it waits for a signal after being selected, then `stream` with `secs`
counting from the moment the signal started (measured: 107 s of
`connecting` until the CD started, then `stream` from 0). Once streaming,
it stays `stream` even when the CD transport is switched off: the player
notices when a signal starts, not when it stops. No track, no format, no
`serviceName`.

Today the adapter maps this to a service origin "Capture" with `title1` as a
track title and `secs` as a position, and reads `/Playlist` because
`is_native` treats every non-cast service as native. The kiosk's idle screen
already shows a large clock (`IdleScreen`). The display controller follows
the published player snapshots with a monotonic timer and has no clock.
See proposal.md for the motivation.

## Goals / Non-Goals

**Goals:**

- An input is a first-class session: kind, name, start of its signal; no fake
  track data anywhere.
- The screen does not stay on forever behind an input.
- Each stack layer builds and passes CI on its own; only the top layer
  needs the reference host.

**Non-Goals:** recognising the music (DEV-21), Bluetooth (unmeasured; may
carry track data), admin settings (DEV-6), volume display.

## Decisions

### 1. An input origin and an input on the snapshot

Domain: `Origin::Input(AudioInput)` with `AudioInput { kind: InputKind,
name: String, started_at: Option<Timestamp> }` (`None` while waiting for a
signal) and `InputKind { Optical, Analog,
HdmiArc, Other }`. The device reports no content for an input. The player
snapshot gains `input: Option<AudioInput>`, set by the arbiter for an input
origin; the session label is the kind's label ("Optical input", "Analogue
input", "HDMI ARC", "Input").

- Alternative: keep a service origin and fill `Content` with the input's
  name as the title. Rejected: that is the fake track data this change
  removes; every consumer would have to guess.
- Alternative: an input variant of `Content`. Rejected: content means "what
  is playing"; for an input nobody knows.

### 2. Start time from `secs`, with a tolerance

While `state` is `stream`, the adapter computes `started_at` as the
observation time minus `secs`; while `connecting`, there is none.
Each poll gives a slightly different value (rounding, request time), so
change detection treats moves of up to 2 s (`POSITION_TOLERANCE`) as equal
and emits `InputChanged` only beyond that, or when the kind, name or
presence changes.

- Alternative: remember the first `started_at` in the adapter and keep it
  while `secs` grows. Rejected: state in the adapter for something the
  domain's change detection already handles for positions.

### 3. Bluetooth stays a service origin

`inputTypeIndex` starting with `bluetooth` keeps today's mapping: a
Bluetooth source may send track data, and we have not measured it.

### 4. Input kind from `inputTypeIndex`

The prefix before the dash: `spdif` → optical (the Node 2i's digital input
is optical; BluOS's own icon says so), `analog` → analog, `arc` or `hdmi` →
HDMI ARC, anything else → other. Unknown values are not an error.

### 5. Display power: an input time limit

`DisplayPolicy` gains the input limit. The controller needs wall time to
compare with `started_at`, so it gets the `Clock` port. On each snapshot it
treats the player as active while online, playing or buffering, and for an
input only while its signal has started and its age is below the limit (an
input waiting for a signal counts as not playing: otherwise a selected
input with the CD transport off would keep the screen on, since `secs`
stays 0); it sleeps until whichever comes
first: the end of the grace period, or the moment the input passes its
limit (computed once per snapshot from the clock, then waited on with the
monotonic timer). When the limit passes, the grace period starts from that
moment, so the display goes off at limit plus grace.

`INPUT_DISPLAY_MINUTES` (whole minutes 0 to 1440, default 90) from the
environment, like `DISPLAY_GRACE_MINUTES`; 0 means inputs never count as
playing.

- Alternative: inputs always count as playing (today). Rejected: the screen
  stays on overnight behind a switched-off CD transport.
- Alternative: inputs never count as playing. Rejected: the input screen
  would vanish after the grace period during every CD.

### 6. API

`input`: `{ kind, name, startedAt }` (`startedAt` null while waiting for a
signal) or null, with `kind` one of
`optical`, `analog`, `hdmi-arc`, `other`. No vendor values: `Capture`,
`AudioInputs` and `inputTypeIndex` stay in the adapter.

### 7. Kiosk input screen

A new `InputScreen`, chosen when the snapshot has an input and is not
stopped. It reuses the idle screen's clock styling, adds a disc graphic
drawn in SVG with theme colours that turns with a CSS `transform`
animation (one turn in 24 s, linear, infinite; none with reduced motion),
the input's name, the session label as a badge, and "for N min" computed
from `startedAt` and the clock offset, updated every 30 s; "waiting for
signal" while `startedAt` is null. Sizes in the
theme's vmin tokens; landscape puts the disc beside the text, portrait
above.

- Alternative: the playing screen without track data. Rejected: an empty
  cover slot and nothing that changes.

### 8. Fake scenario

`optical-input`: an optical input named "Example Input", streaming since 23
minutes before the start, for the end-to-end screenshots.

### 9. Stack layers

| Layer | Content | Builds on | Needs the reference host |
| --- | --- | --- | --- |
| 1 | OpenSpec change; domain (input origin, snapshot field, change detection, display policy); application (arbiter label, display controller with clock) | `main` | no |
| 2 | BluOS adapter: input mapping, no `/Playlist` for inputs, recorded fixtures | 1 | to record the fixtures |
| 3 | API `input`; `INPUT_DISPLAY_MINUTES` and wiring; fake scenario; README and `.env.example` | 2 | no |
| 4 | Kiosk input screen; Playwright; acceptance on the reference host; sync and archive | 3 | yes |

Each layer is one branch, one pull request and one Linear sub-issue of
DEV-20.

### 10. Tests per layer

| Layer | Tests |
| --- | --- |
| Domain unit | Input in the snapshot; InputChanged and the 2 s tolerance; policy with the input limit |
| Application with fake ports | Arbiter label and input; display controller with tokio's paused time and a clock that follows it: within the limit, past limit plus grace, waiting for a signal, signal starts again, other sources |
| Adapter fixtures | Recorded optical input (connecting, streaming): kind, name, start time, no content, no `/Playlist` request; Bluetooth unchanged |
| HTTP | `input` present and null; label |
| App | `INPUT_DISPLAY_MINUTES` default, range, invalid values |
| Kiosk components and screenshots | Input screen content, "for N min", no progress or queue, reduced motion; Playwright landscape and portrait with the fake scenario |
| End to end | Reference host with the Leak CDT on the optical input |

## Risks / Trade-offs

- [A second CD more than 90 minutes after the signal started, without a new
  start: dark screen] → selecting the input again (a new start of the
  signal) wakes it; the screen shows no track
  data anyway; the limit is configurable.
- [Other BluOS players report other `inputTypeIndex` values] → unknown
  values map to "other" with the label "Input"; nothing breaks.
- [Clock skew between the player's `secs` and the host] → `started_at` is
  computed from the host's own observation time, so only `secs` matters.
- [Bluetooth left as is] → recorded as a non-goal; a later change can
  measure it.

## Migration Plan

No data. `/api/v1/player` gains a field; existing clients ignore it.
Rollback: deploy the previous image.
