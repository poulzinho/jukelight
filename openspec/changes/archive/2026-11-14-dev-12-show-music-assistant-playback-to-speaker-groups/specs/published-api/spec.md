## ADDED Requirements

### Requirement: Speakers and outputs
The snapshot SHALL contain `speakers`, the other speakers playing the same
session, each with `name`, `kind` (`speaker` or `computer`), `playing`,
and `quality` (the format it receives, or `null`) with `class`, the lower
of the source's class and the class of what it receives, so a speaker fed
upsampled CD quality is not Hi-Res; and
`outputs`, the player's own outputs, each with `name` and `kind`
(`speaker` or `subwoofer`). Both SHALL be empty lists when there are none.

#### Scenario: Recorded group
- **WHEN** the session plays to two other speakers, "Kitchen" receiving PCM 16/44.1 and "Office" FLAC 24/44.1, and the player has the output speaker "Example Speaker" and a subwoofer
- **THEN** `speakers` has "Kitchen" (`kind` `speaker`, `quality.bitDepth` 16, `quality.codec` "PCM") and "Office" (`kind` `computer`), and `outputs` is `[{"name": "Example Speaker", "kind": "speaker"}, {"name": "Subwoofer", "kind": "subwoofer"}]`

#### Scenario: Padded CD quality is not Hi-Res
- **WHEN** the source is FLAC 16/44.1 and the speaker "Office" receives FLAC 24/44.1
- **THEN** its `quality.class` is "lossless"

#### Scenario: Playing alone
- **WHEN** the player plays alone with no outputs configured or reported
- **THEN** `speakers` and `outputs` are `[]`

### Requirement: The maker's mark
The snapshot's `device` SHALL contain `markUrl`, the URL of the maker's mark
(`<maker>-mark.svg` or `.png` in the brand assets folder), or `null` when
none is supplied; served like the other brand logos.

#### Scenario: Mark supplied
- **WHEN** the device's brand is bluesound and `bluesound-mark.svg` is in the brand assets folder
- **THEN** `device.markUrl` is `/api/v1/logos/bluesound-mark?v=<version>`

#### Scenario: No mark
- **WHEN** no mark is supplied
- **THEN** `device.markUrl` is `null`
