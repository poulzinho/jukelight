# DEV-43: Pulse the signal path only when something changes

## Why

A floor test on the reference host showed that the pulse is now most of
the kiosk's CPU use. With the stepped pulse every 8 s (DEV-42), Chrome
uses about 225 s of CPU per hour of playback. With no pulse at all
(Chrome with `--force-prefers-reduced-motion`), it uses about 66 s. So
the pulse costs about 160 s per hour, about 70 % of the total. Another
browser, a lighter framework, canvas or an animated image would leave
that cost as it is, because every pulse frame is a 4K composite whatever
draws it. The user chose to keep the pulse but run it only when something
happens. Part of roadmap step 4 (finished kiosk).

## What Changes

- **One pulse** (the relay from the source to the sound box, stepped as
  today) runs when:
  - a track starts, including when the screen first shows it;
  - the source or the route changes;
  - the quality changes;
  - playback resumes after a pause.
- **The rest of the time the waves stand still.** The brightening wave
  and the ports still show the direction.
- **Removed:** the 8-second timer.

### Non-goals

- A burst of several pulses per change.
- Changing the pulse's look, steps or the relay.
- Browser, framework or output-resolution changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kiosk-display`: "Wave motion" pulses on a change instead of every 8
  seconds.

## Impact

- `web/kiosk/src/components/SignalPath.tsx`: the pulse is keyed by what it
  is about instead of a timer.
- Component and e2e tests for the pulse's timing.
- Reference host: an hour's CPU measurement against about 225 s per hour
  (expected close to the 66 s floor); the screen judged from across the
  room.
