## ADDED Requirements

### Requirement: Signal
The snapshot SHALL contain a `signal` describing how the music reaches the
player, or `null` when nothing plays from a session (idle, or an input):
- `class`: the quality class of the whole path, the lowest of the source's
  and the delivered quality: `"hi-res"` (the Hi-Res rule), `"lossless"`
  (any other lossless format), `"lossy"`, or `"unknown"` when the source's
  quality is not reported;
- `source`: the quality as the source delivers it (the same fields as
  `quality`), or `null`;
- `delivered`: the quality the route sends to the player, or `null` when
  there is no route or it is not reported;
- `changes`: what the route changed, each an object with a `kind` of
  `"resampled"`, `"bitDepth"`, `"normalised"` (with `gainDb`, or `null`)
  or `"dsp"`, in that order;
- `hiResLogoUrl`: the Hi-Res logo's URL when `class` is `"hi-res"` and the
  logo is supplied, otherwise `null`.

#### Scenario: Hi-Res through Music Assistant
- **WHEN** Qobuz delivers FLAC 24/96 and Music Assistant sends FLAC 24/96, normalised by −6.0 dB, with the Hi-Res logo supplied
- **THEN** `signal.class` is "hi-res", `delivered.sampleRateHz` is 96000, `changes` is `[{"kind": "normalised", "gainDb": -6.0}]` and `hiResLogoUrl` is set

#### Scenario: Upsampling stays lossless
- **WHEN** the source delivers FLAC 16/44.1 and Music Assistant sends FLAC 24/96
- **THEN** `signal.class` is "lossless", `hiResLogoUrl` is `null` and `changes` starts with `{"kind": "resampled"}`

#### Scenario: Downsampled Hi-Res
- **WHEN** the source delivers FLAC 24/192 and Music Assistant sends FLAC 16/48
- **THEN** `signal.class` is "lossless"

#### Scenario: Native service
- **WHEN** the player plays Spotify at 320 kbps without a route
- **THEN** `signal.class` is "lossy", `source.bitrate` is 320000, `delivered` is `null` and `changes` is empty

#### Scenario: Quality not reported
- **WHEN** a session plays without a reported quality
- **THEN** `signal.class` is "unknown" and `source` is `null`

#### Scenario: Input
- **WHEN** an input plays
- **THEN** `signal` is `null`
