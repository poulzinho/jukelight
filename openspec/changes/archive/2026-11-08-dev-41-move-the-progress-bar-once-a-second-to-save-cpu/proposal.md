# DEV-41: Move the progress bar once a second to save CPU

## Why

On the reference host, Chrome uses about 500 s of CPU per hour of playback
(about 14 % of one core), mostly in its GPU process (about 61 %) and its
browser process: the cost of drawing frames, not of the page's own work.
The progress bar keeps it drawing. It re-renders every 250 ms, and each
step starts a 300 ms `transform` transition. The steps overlap, so the
bar is always moving, and Chrome draws about 60 frames a second at 4K for
the whole song. Found while measuring DEV-37. Part of roadmap step 4
(finished kiosk).

## What Changes

- The progress bar and the elapsed time move **once a second, together**,
  when the elapsed time reaches the next whole second, **without a
  transition**. Between those moments nothing in the progress animates.
- The e2e motion check no longer expects a 0.3 s transition on the bar.

### Non-goals

- Other animations (the cover and track text fades, the disc, DEV-37's
  pulses): they are one-off or already rest.
- The clock on the idle and input screens: it already changes once a
  second without a transition.
- Lowering the output resolution or Chrome's frame rate.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kiosk-display`: "Progress between updates" moves once a second, without
  a transition.

## Impact

- `web/kiosk/src/components/Progress.tsx`, a small hook for the ticks.
- Component tests and `e2e/kiosk.spec.ts` (the motion check).
- Reference host: an hour's CPU measurement before and after, with the
  same command as for DEV-37.
