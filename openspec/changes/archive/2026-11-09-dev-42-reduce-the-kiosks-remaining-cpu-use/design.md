## Context

DEV-37 animates the pulse with two one-off CSS animations per wave:
`signal-pulse` (`transform` of a bright band, 2.2 s, `linear`) and
`signal-arrive` (`opacity` of the port's core, 2.2 s, `linear`). A timer
remounts them every 8 s, and the second wave starts 2 s later. Chrome runs
both on the compositor, which produces a frame on every vsync while they
run: 60 a second.

Measured in Chromium at 2160 × 3840 with the `music-assistant` scenario
(16 s traces, `DrawFrame` events):

| Pulse | Frames per second, on average |
| --- | --- |
| Smooth (`linear`) | 28.6 |
| `steps(33)` per 2.2 s wave (about 15 a second) | 12.9 |
| `steps(24)` | 11.6 |
| No pulses (reduced motion) | 1.1 |

## Goals / Non-Goals

**Goals:**
- About 15 frames a second while a pulse runs, with the same path and
  timing.

**Non-Goals:**
- Changing the pulse's timing, how often it runs or its look.

## Decisions

### 1. `steps()` on both animations

Both theme animations use `steps(33)` instead of `linear`: 2.2 s × 15 =
33 steps. The compositor only produces a frame when a step changes the
value. The relay delay (2 s) and the 8 s timer stay as they are.

- *Alternative:* moving the band from JavaScript at 15 frames a second.
  Rejected: it puts the main thread to work on every step and makes the
  timing harder to test; the compositor does the same with one keyword.
- *Alternative:* fewer steps (`steps(24)`, about 11 a second). Rejected
  for now: only about 10 % fewer frames than 33 steps, for a visibly
  choppier pulse. The acceptance can still lower it.

### Tests per layer

- **Kiosk components:** none new. jsdom doesn't apply the theme's
  animations, so the stepping is checked in the browser.
- **e2e:** "Stepped pulse": the running pulse and port animations use
  `steps(33)`. The existing checks still hold: transform and opacity
  only, the relay delays, nothing between pulses.
- **Reference host:** an hour of playback against about 296 s per hour;
  the pulse judged from across the room.

## Risks / Trade-offs

- [The pulse looks choppy] → 15 steps a second over a few centimetres of
  wave; judged in the acceptance, and the step count is one number to
  change.
- [The ThinkCentre's saving differs from the trace's frame count] → the
  hour's measurement decides; the trace is Chromium on a Mac, without the
  ThinkCentre's GPU.
