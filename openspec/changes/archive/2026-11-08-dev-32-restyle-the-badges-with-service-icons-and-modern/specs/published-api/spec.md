## ADDED Requirements

### Requirement: Source and device
The snapshot SHALL contain `source` with `brand` (a short identifier such
as `qobuz`, or `null`), `name`, and `via` (the route: `name` and
`available`, or `null`), and `device` with `brand` (or `null`) and `name`
(the player's name). The existing `session` label SHALL stay.

#### Scenario: Qobuz through Music Assistant
- **WHEN** the snapshot's source is Qobuz through Music Assistant on a Bluesound player named "Example Player"
- **THEN** `source` is `{"brand": "qobuz", "name": "Qobuz", "via": {"name": "Music Assistant", "available": true, …}}` and `device` is `{"brand": "bluesound", "name": "Example Player", …}`

### Requirement: Brand logos
The backend SHALL serve logos that the user puts into the brand assets
folder (`brand-assets/`, configurable), named `<brand>.svg` or
`<brand>.png`, at `/api/v1/logos/{brand}`, and SHALL notice added or
removed files without a restart. A brand's `logoUrl` (in `source`, in
`source.via`, in `device`, and for Hi-Res in `quality.hiResLogoUrl`, under
the brand `hi-res`) SHALL be present only when its logo exists, and
`null` otherwise. The repository SHALL contain no brand logos.

#### Scenario: Logo supplied
- **WHEN** the folder contains `qobuz.svg` and the source's brand is qobuz
- **THEN** `source.logoUrl` points to `/api/v1/logos/qobuz`, which serves the SVG

#### Scenario: No logo
- **WHEN** the folder contains no logo for the source's brand
- **THEN** `source.logoUrl` is `null`

#### Scenario: Logo added later
- **WHEN** the backend runs and `spotify.png` is added to the folder
- **THEN** the next snapshot with the brand spotify has a `logoUrl`, without a restart

#### Scenario: Unknown brand requested
- **WHEN** a client requests `/api/v1/logos/example` and no such logo exists
- **THEN** the response is 404

### Requirement: Quality details
Next to its existing `label`, `detail` and `hiRes`, the snapshot's
`quality` SHALL contain `codec` (in capitals, such as "FLAC"), `bitDepth`
and `sampleRateHz` for a lossless format, or `bitrate` (bits per second) for a lossy one; the
fields that don't apply SHALL be `null`.

#### Scenario: Lossless
- **WHEN** the quality is FLAC at 24 bits and 96 000 Hz
- **THEN** `quality` has `codec` "FLAC", `bitDepth` 24, `sampleRateHz` 96000 and `bitrate` `null`

#### Scenario: Lossy
- **WHEN** the quality is 160 kbps
- **THEN** `quality` has `bitrate` 160000 and `codec`, `bitDepth` and `sampleRateHz` `null`
