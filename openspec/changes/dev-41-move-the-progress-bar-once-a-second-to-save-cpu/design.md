## Context

`Progress` takes `useNow(250)` and draws the fill with
`transform: scaleX(fraction)` and a 300 ms linear transition, so every
quarter second starts a new transition before the last one ends. Chrome
keeps drawing frames while any transition or animation runs. The elapsed
time is shown in whole seconds (`formatTime`), so between ticks only the
bar changes.

## Goals / Non-Goals

**Goals:**
- At most about one frame a second from the progress while playing.
- The bar and the time change at the same moment.

**Non-Goals:**
- A smoother bar: at 4K the fill moves a few pixels a second, which looks
  the same from across the room.

## Decisions

### 1. Ticks at whole elapsed seconds

A hook, `useSecondTicks(position, state, clockOffsetMs)`, returns the
current time and schedules its next update with `setTimeout` for the
moment the predicted elapsed time reaches the next whole second. While
paused or without a position it schedules nothing.

- *Alternative:* `useNow(1000)`. Rejected: its ticks don't line up with
  the elapsed seconds, so the time could change up to a second after the
  bar, and after each snapshot the phase drifts.

### 2. No transition on the fill

The fill keeps `transform: scaleX(...)` (only transform changes, as the
kiosk rule asks) and loses its `transition`. One step a second is one
frame a second.

- *Alternative:* one CSS animation from the current fraction to 1 over the
  remaining duration. Rejected: it runs continuously, which is what keeps
  Chrome drawing.

### Tests per layer

- **Kiosk components** (fake timers): "Once a second" (nothing changes
  before the next whole second, then the bar and the time change
  together), paused schedules nothing, and the fill has no transition
  class.
- **e2e:** the motion check expects no transition on the fill, in both
  projects; "No transition" checks that no animation runs in the progress.
- **Reference host:** an hour of playback, compared with the 500 s per
  hour measured for DEV-37's baseline.

## Risks / Trade-offs

- [The bar moves in visible steps] → a few pixels a second at 4K; judged
  in the acceptance from across the room.
- [Timers drift or a tab sleeps] → every tick recomputes from the
  snapshot's position and the clock, so no error builds up.
