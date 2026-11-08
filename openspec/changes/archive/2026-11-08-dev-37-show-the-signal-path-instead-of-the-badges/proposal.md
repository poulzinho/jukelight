# DEV-37: Show the signal path instead of the badges

Builds on DEV-32 (source brands, user-supplied logos, device signature),
whose stack is not merged yet: this stack starts on DEV-32's top layer and
moves onto `main` once DEV-32 lands.

## Why

The badge line packs the source, the route, the Hi-Res mark, the quality
and its detail into one line of text. In portrait it runs out of room and
gives parts away, and it can't show what happened to the music on the way:
a CD-quality track that Music Assistant upsamples to 24/96 reads like a
Hi-Res one. A small diagram (where the music comes from, the broker it
passes through, and the sound the player receives) says that at a glance,
from across the room. Part of roadmap step 4 (finished kiosk).

## What Changes

- **Signal path on the playing screen**, replacing the badge line: the
  source's logo (or icon and name), a wave, Music Assistant when it is the
  route, a second wave, and the **sound box**. The player is not a node: it
  is always the end and already signs the footer.
- **Waves that show the direction:** calm, tapered into a port at each end,
  with pulses of light running towards the end; the arrival port lights up
  as they arrive. Still with reduced motion and while paused.
- **One colour per path, from its quality class**, set by the lowest
  quality anywhere on the path: Hi-Res yellow, Lossless teal, Lossy violet,
  Unknown grey. The leg after Music Assistant uses a lighter shade of the
  same colour. **Upsampling never makes Hi-Res.**
- **The sound box:** the Hi-Res logo when the path is Hi-Res, the format
  the player receives ("24 bit / 96 kHz" or "320 kbps"), the codec, and
  what changed on the way ("untouched", "normalised (−6 dB)", "resampled
  from 16 / 44.1"). Quiet text; the coloured border carries the quality.
- **What Music Assistant did:** the provider reports the format it sends
  to this player and the changes it made: resampled, bit depth changed,
  normalised (with the applied gain), DSP.
- **The API** publishes a `signal` with the quality class, the source's
  and the delivered quality, the changes and the Hi-Res logo URL. The
  existing `quality` and `session` stay as they are.
- **Removed:** the source badge, the Hi-Res tag and the one-line quality
  from DEV-32. The input screen shows its session label with an input
  icon instead of a badge. The album's accent colour no longer tints the
  quality (it keeps the progress bar).

### Non-goals

- No signal path on the input screen: inputs report no quality.
- No stage for the player's own processing (BluOS reports none): the
  sound box is what the player receives.
- No admin setting to choose between the signal path and the badges.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-music-assistant`: the delivered quality and the changes Music
  Assistant made to the audio.
- `published-api`: the `signal` object.
- `kiosk-display`: the signal path replaces the source badge, the Hi-Res
  badge and the quality line; the input screen, album colours, update in
  place and the steady portrait layout follow.

## Impact

- `crates/domain`: quality class, delivered quality and changes on the
  content.
- `crates/adapter-music-assistant`: mapping of `audio_processing`
  (outputs, normalisation, DSP).
- `crates/adapter-http`: `SignalDto`.
- `crates/adapter-fake`: scenarios for each quality class and the
  upsampled path.
- `web/kiosk`: a `SignalPath` component in place of `Badges`; e2e
  screenshots in landscape and portrait.
- Kiosk CPU: the pulses animate; measured on the reference host during the
  acceptance, as for DEV-28.
