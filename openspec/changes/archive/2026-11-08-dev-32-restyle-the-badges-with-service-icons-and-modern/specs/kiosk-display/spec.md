## ADDED Requirements

### Requirement: Source badge
The playing screen SHALL show the source in one dark badge: the source's
logo when it has a `logoUrl`, otherwise an icon for its brand (or a
generic one) and its name. A route SHALL appear inside the badge as "via"
and the route's logo (or its name). An unavailable route SHALL be shown as
a separate greyed, outlined badge before the source.

#### Scenario: Logo supplied
- **WHEN** the source is Qobuz with a `logoUrl`, through Music Assistant with a `logoUrl`
- **THEN** one badge shows the Qobuz logo, "via" and the Music Assistant logo

#### Scenario: No logos
- **WHEN** the source is Qobuz without a `logoUrl`, through Music Assistant without a `logoUrl`
- **THEN** one badge shows an icon, "Qobuz", "via" and "Music Assistant"

#### Scenario: Route unavailable
- **WHEN** the route "Music Assistant" is unavailable
- **THEN** a greyed, outlined badge marks Music Assistant as unavailable, followed by the source's badge

### Requirement: Quality on one line
The badge, the Hi-Res mark, the quality ("24 bit / 96 kHz FLAC", with
tabular figures) and its detail SHALL stay on one line, without wrapping
inside any part. When space runs out, the detail SHALL be shortened with
"…" first (or left out when hardly any of it fits), then the route's "via"
part SHALL be left out as a whole; the source, the Hi-Res mark and the
quality SHALL never be shortened.

#### Scenario: Everything fits
- **WHEN** the source is Qobuz through Music Assistant and the quality is Hi-Res "24 bit / 96 kHz FLAC" with the detail "Normalised" in landscape
- **THEN** all of it shows on one line

#### Scenario: Too long for portrait
- **WHEN** the screen is in portrait and the line would be wider than the column
- **THEN** the detail is shortened or left out first, and no part wraps

### Requirement: Device signature
Every screen SHALL show the player's maker and name in a quiet footer:
the maker's logo when `device.logoUrl` is present, otherwise a streamer
icon and the maker's name when the maker is a known brand; then the
player's name. With an unknown maker, only the player's name SHALL show.
The footer SHALL sit centred at the bottom in portrait and in the
bottom-right corner in landscape, keeping its place in the steady portrait
layout.

#### Scenario: Bluesound player with its logo
- **WHEN** the device has a `logoUrl` and the name "Example Player"
- **THEN** the footer shows the logo and "Example Player" on the playing, input and idle screens

#### Scenario: No logo
- **WHEN** the device's brand is bluesound, without a `logoUrl`
- **THEN** the footer shows a streamer icon, "Bluesound" and "Example Player"

#### Scenario: Unknown maker
- **WHEN** the device has no brand
- **THEN** the footer shows only "Example Player"

## MODIFIED Requirements

### Requirement: Playing screen
While a track is playing, the kiosk SHALL show the artwork, title, artists,
album, a progress bar with elapsed time and duration, the source badge, and
the quality with its detail on one line (see "Source badge" and "Quality on
one line"). Titles longer than two lines SHALL end with an ellipsis.

#### Scenario: Track with all data
- **WHEN** the snapshot is playing "Example Song" by "Example Artist" with artwork, position, the source "Qobuz" and the quality FLAC at 16 bits and 44 100 Hz
- **THEN** the screen shows the artwork, "Example Song", "Example Artist", the progress, a "Qobuz" source badge and "16 bit / 44.1 kHz FLAC"

#### Scenario: No position
- **WHEN** the snapshot has no position
- **THEN** the progress bar and times are hidden

### Requirement: Hi-Res badge
When the quality is Hi-Res, the kiosk SHALL show the Hi-Res mark next to
the quality: the Hi-Res logo when `quality.hiResLogoUrl` is present,
otherwise a gold "HI-RES" text tag.

#### Scenario: Hi-Res track
- **WHEN** the quality is "FLAC 24/96" and Hi-Res, without a Hi-Res logo
- **THEN** a "HI-RES" tag is shown

#### Scenario: Hi-Res track with the logo supplied
- **WHEN** the quality is Hi-Res and `quality.hiResLogoUrl` is present
- **THEN** the Hi-Res logo is shown instead of the tag

#### Scenario: CD-quality track
- **WHEN** the quality is "FLAC 16/44.1" and not Hi-Res
- **THEN** no Hi-Res mark is shown

### Requirement: Idle screen
When nothing is playing (stopped, or no track and no input), the kiosk
SHALL show the current time and "nothing playing", with the device
signature.

#### Scenario: Stopped
- **WHEN** the snapshot is stopped and the player name is "Example Player"
- **THEN** the screen shows the time, "nothing playing" and, in the footer, "Example Player"
