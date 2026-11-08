# kiosk-display Specification

## Purpose

The kiosk screen: what it shows in each playback state, readable from across
the room on landscape and portrait screens.

## Requirements

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

### Requirement: Progress between updates
The kiosk SHALL advance the progress bar and elapsed time by itself from the
snapshot's position while the state is playing, and SHALL keep them still
while paused.

#### Scenario: Progress without updates
- **WHEN** the snapshot reports elapsed 60 s of 200 s while playing and no update arrives for 10 seconds
- **THEN** the screen shows about 1:10 elapsed

### Requirement: Missing artwork
When the snapshot has no artwork, the kiosk SHALL show a placeholder with
up to two initials of the album, or of the artists when there is no album.

#### Scenario: No artwork
- **WHEN** the snapshot has album "Example Album" and no artwork
- **THEN** the placeholder shows "EA"

### Requirement: Up next
The kiosk SHALL show an "Up next" list only when the queue is available and
not empty: 4 entries in landscape and 3 in portrait.

#### Scenario: Queue available
- **WHEN** the queue holds 5 upcoming entries and the screen is landscape
- **THEN** "Up next" shows the first 4 entries

#### Scenario: No queue
- **WHEN** the queue is not applicable
- **THEN** no "Up next" list is shown

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

### Requirement: Paused screen
While paused, the kiosk SHALL dim the content and show a large pause symbol
over the artwork at full brightness, with the progress frozen.

#### Scenario: Pause
- **WHEN** the snapshot changes from playing to paused
- **THEN** the content is dimmed, the pause symbol is shown and the elapsed time stops advancing

### Requirement: Idle screen
When nothing is playing (stopped, or no track and no input), the kiosk
SHALL show the current time and "nothing playing", with the device
signature.

#### Scenario: Stopped
- **WHEN** the snapshot is stopped and the player name is "Example Player"
- **THEN** the screen shows the time, "nothing playing" and, in the footer, "Example Player"

### Requirement: Offline and reconnecting
When the player is offline or the connection to the backend is lost, the
kiosk SHALL keep the last screen and show a small indicator in a corner.
After reconnecting it SHALL show the current snapshot again.

#### Scenario: Backend restarts
- **WHEN** the live update stream drops and comes back
- **THEN** the indicator shows while disconnected and disappears once the current snapshot arrives

#### Scenario: Player offline
- **WHEN** the snapshot reports the player as offline
- **THEN** the last screen stays and the indicator is shown

### Requirement: Landscape and portrait
The kiosk SHALL follow the screen orientation without a setting: artwork and
information side by side in landscape, artwork above the information in
portrait. Text and artwork SHALL be sized relative to the shorter side of
the screen.

#### Scenario: Portrait screen
- **WHEN** the screen is 2160 by 3840 pixels
- **THEN** the artwork is shown above the title and information

#### Scenario: Landscape screen
- **WHEN** the screen is 3840 by 2160 pixels
- **THEN** the artwork is shown to the left of the title and information

### Requirement: Reduced motion
When the browser requests reduced motion, the kiosk SHALL switch states
without transitions.

#### Scenario: Reduced motion
- **WHEN** reduced motion is requested and the track changes
- **THEN** the new track is shown at once, without a fade

### Requirement: Album colours
When the snapshot has album colours, the kiosk SHALL tint its background
with the background colour and use the accent colour for the progress bar
and the quality badge. Text SHALL stay near-white for contrast. Without
colours, the kiosk SHALL use its default dark theme.

#### Scenario: Colours known
- **WHEN** the snapshot has the background colour (73, 51, 38) and the accent colour (209, 147, 108)
- **THEN** the page background is rgb(73, 51, 38) and the progress bar fill is rgb(209, 147, 108)

#### Scenario: Colours unknown
- **WHEN** the snapshot has no colours
- **THEN** the page uses the default background rgb(18, 18, 22)

#### Scenario: Colours change with the track
- **WHEN** the next track has different colours
- **THEN** the page uses the new colours

### Requirement: Input screen
When an audio input plays (the snapshot has an input and is not stopped),
the kiosk SHALL show the current time, a disc that turns slowly (one turn
in 20 to 30 seconds), the input's name, the session label as a badge, and
"for N min" since the input's signal started, in whole minutes, or
"waiting for signal" while there is none yet. It SHALL show no artwork,
track time, progress bar or queue. With reduced motion the disc SHALL stand
still.

#### Scenario: Optical input
- **WHEN** the snapshot has the optical input named "Example Input", whose signal started 23 minutes ago, with the session label "Optical input"
- **THEN** the screen shows the time, the turning disc, "Example Input", the badge "Optical input" and "for 23 min", and no progress bar or "Up next"

#### Scenario: Just started
- **WHEN** the input's signal started 20 seconds ago
- **THEN** the screen shows "for 0 min"

#### Scenario: Waiting for a signal
- **WHEN** the input waits for a signal
- **THEN** the screen shows "waiting for signal" instead of the time it plays

#### Scenario: Reduced motion
- **WHEN** the browser prefers reduced motion and an input plays
- **THEN** the disc does not turn

### Requirement: No translation prompt
The kiosk page SHALL tell browsers not to translate it, so a browser set to
another language than the page's does not show a translation prompt on the
screen.

#### Scenario: Browser in another language
- **WHEN** the kiosk page is loaded
- **THEN** its `html` element has `translate="no"` and the page has the meta tag `google` with the content `notranslate`

### Requirement: Reload after an update
When the player event stream announces a build that differs from the one in
the kiosk's own page, the kiosk SHALL reload the page, at most once per
announced build. When the builds match, or the page carries no build (for
example during development), it SHALL NOT reload.

#### Scenario: New build
- **WHEN** the page carries build `a1b2c3` and the stream announces build `d4e5f6`
- **THEN** the kiosk reloads the page

#### Scenario: Same build
- **WHEN** the page carries build `a1b2c3` and the stream announces build `a1b2c3`
- **THEN** the kiosk doesn't reload

#### Scenario: Still different after reloading
- **WHEN** the kiosk has reloaded for build `d4e5f6` and the reloaded page still carries build `a1b2c3`
- **THEN** the kiosk doesn't reload again for build `d4e5f6`

#### Scenario: No build in the page
- **WHEN** the page carries no build and the stream announces build `d4e5f6`
- **THEN** the kiosk doesn't reload

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
