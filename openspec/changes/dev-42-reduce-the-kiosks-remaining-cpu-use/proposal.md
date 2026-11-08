# DEV-42: Reduce the kiosk's remaining CPU use

## Why

After DEV-41, Chrome still uses about 296 s of CPU per hour of playback on
the reference host (about 8 % of one core). Traces of the kiosk in
Chromium, in portrait at 4K with the fake Music Assistant scenario, show
where it goes. With the signal path's pulse running smoothly, Chrome
draws **28.6 frames a second** on average: 60 a second while a pulse
runs (4.2 s of every 8 s through Music Assistant). Without pulses it
draws **1.1 a second**, just the second's tick. The page's own repaints
are small: about 2 a second and 17 ms of rasterising in 16 s. Each frame
makes Chrome composite the whole 4K screen, which on the reference host
is what its GPU and browser processes spend their CPU on. Part of roadmap
step 4 (finished kiosk).

## What Changes

- The pulse, and the arrival port lighting up with it, move in **about 15
  steps a second** instead of smoothly. Chrome draws a frame only when
  something changes, so a pulse takes about 15 frames a second instead of
  60. Measured in the trace: 12.9 frames a second on average instead of
  28.6.
- Unchanged: the pulse's path, its timing (2.2 s per wave, every 8 s, the
  relay from the first wave to the second), its look, and its rest.

### Non-goals

- Fewer or no pulses; pulses only on a change. The user chose to keep the
  pulse every 8 s, only stepped.
- Chrome flags, the output resolution, the backdrop and compositor layers
  for the second's tick: the traces show they are not where the frames
  come from.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kiosk-display`: "Wave motion" moves the pulse in about 15 steps a
  second.

## Impact

- `web/kiosk/src/index.css`: the pulse's and the port's timing function.
- Component and e2e tests for the stepped pulse.
- Reference host: an hour's CPU measurement against about 296 s per hour,
  and the pulse judged from across the room.
