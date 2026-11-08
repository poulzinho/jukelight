## Context

- `AudioQuality` (domain) holds one measure, the format or the bitrate,
  plus a free-text `detail`. It knows `is_lossless` and `is_hi_res` (at
  least 24 bit and 44.1 kHz, lossless).
- The Music Assistant (MA) mapping already reads the source format, the
  output format for this player and the normalisation mode, but it folds
  the output and normalisation into the `detail` text ("Lossless · → 24/44.1
  to player · Normalised"). The recorded `streamdetails` also carry
  `queue_processing.normalization.applied_gain_db`,
  `volume_normalization_gain_correct` and each output's `dsp.state`.
- BluOS reports a quality only for its own service sessions. For a stream
  it describes the stream that reaches it, not the source, so the mapping
  leaves it out.
- The kiosk draws the badge line in `Badges.tsx` (playing and input
  screens) and fits it with a `ResizeObserver`. DEV-32 brings the source,
  route, logos and device signature that this change reuses.

## Goals / Non-Goals

**Goals:**
- Model the delivered quality and the changes as data, not text, so the
  API and the kiosk don't parse strings.
- Animate the waves without layout or paint work on each frame.

**Non-Goals:**
- Changing the Hi-Res rule, the `quality` object or the session label.
- A signal path for BluOS streams that no provider claims (their quality
  stays unknown).

## Decisions

### 1. Domain: `Delivery` next to the source quality

`Content` gains `delivery: Option<Delivery>`, where `Delivery { quality:
Option<AudioQuality>, changes: Vec<Change> }` and `Change` is
`Resampled`, `BitDepth`, `Normalised { gain_db: Option<f32> }` or `Dsp`.
`QualityClass { Unknown, Lossy, Lossless, HiRes }` is ordered, so the
class of a path is the `min` of the source's class and the delivered one.
`AudioQuality::class()` and `signal_class(source, delivery)` live in the
domain, beside the Hi-Res rule.

- *Alternative:* a list of legs (`Vec<Leg>`) in the domain. Rejected: there
  is at most one route today, and the nodes already come from `Source`; a
  list adds shape without a second use.
- *Alternative:* compute the class in the kiosk. Rejected: the Hi-Res rule
  is in the domain, and the kiosk should not repeat it.

The MA `detail` text stays as it is, so `quality.detail` doesn't change
for API clients.

### 2. Music Assistant mapping

`delivery()` reads the output that names the player (or the only one), as
`output_format()` does today. `Resampled` when the sample rates differ,
else `BitDepth` when only the depths differ; `Normalised` with
`applied_gain_db`, falling back to `volume_normalization_gain_correct`;
`Dsp` when `dsp.state == "enabled"`. New `serde` fields with defaults, so
older MA versions still parse. Fixtures: the existing Qobuz and
OpenSubsonic recordings, plus small edited copies for the upsampled and
DSP cases (example values only).

### 3. API: one `signal` object

`SignalDto { class, source, delivered, changes, hiResLogoUrl }`, built in
`dto.rs` from `content.quality`, `content.delivery` and the logos port.
`source` and `delivered` reuse `QualityDto`. `signal` is `null` without
content (idle, input). Changes serialise as tagged objects
(`{"kind": "normalised", "gainDb": -6.0}`) so a new kind can be added
without breaking the shape.

- *Alternative:* add `delivered` and `changes` to `quality`. Rejected:
  `quality` describes one format; mixing in the path makes `hiRes` ambiguous
  (source or path?).

### 4. Kiosk: `SignalPath` replaces `Badges`

`SignalPath` renders `Node` (logo, or icon and name, reusing `Logo` and
`SourceIcon` from `Badges.tsx`, moved to `Brand.tsx`), `Wave` and
`SoundBox`. The colours are theme tokens (`--color-signal-hi-res`, `-lossless`,
`-lossy`, `-unknown`, each with a `-soft` shade for the leg after the
route). Sizes in vmin. `Badges.tsx` and its fitting code are removed; the
input screen shows `SourceIcon` and the session label.

### 5. Waves: a masked band moved by `transform`

Each wave is a `div` whose CSS `mask-image` is the tapered sine (an SVG
data URL, amplitude about 0.4 vmin, generated once per width). Inside it:
the base gradient (faint at the start, full at the end) and a bright band
that moves with `transform: translateX` from start to end. The arrival
port's core animates `opacity`. Only transform and opacity change, so the
compositor runs the animation without repainting.

- *Alternative:* animate `stroke-dashoffset` on an SVG path (the mock).
  Rejected: it repaints the wave on every frame and breaks the kiosk's rule
  to animate only opacity and transform.
- *Alternative:* slide the whole wave by one period. Kept as the fallback
  if the masked band costs too much on the reference host.

Paused, reduced motion, or an unavailable route: the band is not rendered
and the port core is static.

**Pulses with a rest (acceptance finding).** On the reference host the
badges' build already costs about 500 s of CPU per hour, mostly in
Chrome's GPU and browser processes: drawing frames, which a never-ending
animation keeps at about 60 a second (DEV-41 calms the progress bar). So
a pulse is a one-off animation (1.6 s), started by a timer every 4 s: the
band and the port's core remount with a new key. Between pulses no
animation runs, and Chrome can stop drawing.

A pulse takes 1.3 s per wave. On the second wave it starts 1.2 s later,
as the first one arrives (an `animation-delay`), so one pulse travels the
whole path in 2.5 s and rests for 1.5 s (acceptance finding: the waves
must not light up at the same time).

- *Alternative:* an infinite animation whose keyframes hold still for the
  rest. Rejected: Chrome keeps drawing while any animation is active, even
  when its values don't change.

### 6. Fitting

Waves are `flex: 1` with a minimum of 6 vmin. Each label measures itself
against its wave (one `ResizeObserver` for the row) and hides when wider.
The sound box's third line has `text-overflow: ellipsis`; the nodes and
the first two lines are `shrink-0`.

### Tests per layer

- **Domain unit:** `class()` for each format; `signal_class` takes the
  lowest (upsampled CD stays lossless, downsampled Hi-Res becomes lossless).
- **MA adapter fixtures:** each "Delivered quality" scenario.
- **HTTP:** each "Signal" scenario, with fake logos for `hiResLogoUrl`.
- **Kiosk components:** node order, colours per class, sound box lines,
  the HI-RES tag fallback, no pulses when paused or with reduced motion,
  label hiding.
- **e2e screenshots, landscape and portrait:** fake scenarios
  `music-assistant` (Hi-Res, normalised), `upsampled` (new), `long-badges`
  renamed `long-signal`, Spotify lossy, and unknown quality.

## Risks / Trade-offs

- [Animated mask costs CPU on the reference host] → measure before and
  after for an hour, as for DEV-28; fall back to the sliding wave
  (decision 5) or fewer pulses.
- [MA renames `audio_processing` fields] → `serde` defaults: the delivery
  becomes unknown, and the path shows the source quality only.
- [The stack sits on the unmerged DEV-32 stack] → rebase onto `main` with
  `gh stack rebase` once DEV-32 is merged; DEV-32's requirements this
  change removes are only in its delta until DEV-32 is archived.

## Migration Plan

Nothing to migrate: the API only gains `signal`. Rollback is a revert of
the stack.
