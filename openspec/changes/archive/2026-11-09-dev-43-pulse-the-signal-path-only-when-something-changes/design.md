## Context

DEV-37 and DEV-42 start a pulse every 8 s: `useBeat` counts with
`setInterval`, and the count is the `key` of each wave's pulse band and
arrival-port core. A new key remounts them, which starts their one-off,
stepped CSS animations again (2.2 s per wave, the second wave 2 s later).
Paused, reduced motion or an unavailable route: the band isn't rendered.

## Goals / Non-Goals

**Goals:**
- A pulse only when what the signal path shows changes, or when playback
  resumes; no timer.

**Non-Goals:**
- Changing the pulse's animation.

## Decisions

### 1. Key the pulse by what it is about

The pulse's key becomes a string made from what the pulse is about: the
track's title and artists, the source's brand and name, the route's brand,
name and availability, the signal's class, and the sound box's text. A
new track, source, route or quality gives a new key, which remounts the
band and the port, so their one-off animation runs once. Nothing else
changes the key, so no further pulse runs.

- **Resuming** needs no extra logic: while paused the band isn't rendered,
  and when playback resumes it mounts again, which runs the animation.
- **The first time the screen shows a track** works the same way, through
  the first mount.
- *Alternative:* keep a counter and bump it from an effect when the inputs
  change. Rejected: more state, and the kiosk's lint rejects setting state
  from an effect; the key does it without state.
- *Alternative:* a burst of three pulses per change. Rejected for now:
  one pulse is enough to show the direction, and every pulse costs frames.

### 2. Remove the timer

`useBeat` and `PULSE_EVERY_MS` go. `RELAY_MS` and the stepped animations
stay.

### Tests per layer

- **Kiosk components** (fake timers):
  - "Playing": the band mounts;
  - "Nothing changes": after a minute the band is still the same element;
  - "Quality changes": a new band element;
  - "Playback resumes": paused has no band, and resuming mounts one;
  - "Paused" and "Reduced motion" are unchanged.
- **e2e:**
  - "Between pulses": once the first pulse has run, no animation runs for
    the following 10 s;
  - the existing checks ("From one end to the other", "Stepped pulse",
    transform and opacity only) still see the first pulse.
- **Reference host:** an hour of playback against about 225 s per hour.

## Risks / Trade-offs

- [The screen looks too still] → the direction still shows in the
  brightening wave and the ports; judged in the acceptance from across the
  room. A burst or an occasional pulse can come back later.
- [A source that rewrites its quality text often makes many pulses] →
  each change is one pulse; the hour's measurement shows it, and the key
  can leave out the third line of the sound box if needed.
