## ADDED Requirements

### Requirement: Backdrop
On the playing screen, the kiosk SHALL show the backdrop image stretched
over the whole screen, behind a dark veil that keeps the text readable,
and SHALL cross-fade to a new backdrop when the cover changes. Without a
backdrop, or when it fails to load, the screen SHALL show the plain
background colour. The kiosk SHALL NOT blur with CSS filters.

#### Scenario: Track with a backdrop
- **WHEN** the snapshot has a `backdropUrl`
- **THEN** the playing screen shows that image over the whole screen, behind the veil

#### Scenario: No backdrop
- **WHEN** the snapshot has no `backdropUrl`
- **THEN** the playing screen shows the plain background colour

### Requirement: Update only what changed
On a track change, the kiosk SHALL change only the track's title, artists
and album: the old text fades out, then the new text fades in. The cover
and the backdrop SHALL stay when the new track has the
same album and artists. A new cover SHALL replace the old one only once it
has loaded, with a cross-fade, so the cover is never blank. Badges,
progress and "Up next" SHALL update in place, without fading.

#### Scenario: Next track on the same album
- **WHEN** the track changes to another track with the same album and artists
- **THEN** the cover is still the same element and the title fades in

#### Scenario: Old text fades out first
- **WHEN** the track changes
- **THEN** the old title is still shown, fading out, and the new title replaces it once the fade-out has finished

#### Scenario: Track with a new cover
- **WHEN** the track changes to a track with a different album and its cover hasn't loaded yet
- **THEN** the old cover is still shown, and the new one replaces it once it has loaded

#### Scenario: Quality changes
- **WHEN** only the quality of the current track changes
- **THEN** only the badges change, and nothing fades

### Requirement: Steady portrait layout
In portrait, the cover, the title (room for two lines), the artists (one
line), the album (one line), the progress, the badges (one row) and "Up
next" SHALL each keep a fixed place on the screen, so changing content
never moves the other parts.

#### Scenario: No queue
- **WHEN** the screen is in portrait and one track has three "Up next" entries and another has none
- **THEN** the cover and the title are in the same place for both

#### Scenario: Long title
- **WHEN** the screen is in portrait and one track's title fits on one line and another's needs two
- **THEN** the cover, the artists and "Up next" are in the same place for both

#### Scenario: Missing album
- **WHEN** the screen is in portrait and one track has an album and another has none
- **THEN** the progress and "Up next" are in the same place for both
