## ADDED Requirements

### Requirement: Signal path
The playing screen SHALL show how the music reaches the player in one row:
the source node (the source's logo, or its icon and name), a wave, then,
when the source has a route, the route's node (its logo, or its name) and
a second wave, and finally the sound box. Above each wave, a quiet label
SHALL give that leg's quality and codec ("24/96 FLAC", or "320 kbps" for a
lossy stream, whose codec isn't published); above the second wave, the first change ("normalised",
"resampled") replaces the codec. The player SHALL NOT be a node.

#### Scenario: Through Music Assistant
- **WHEN** the source is Qobuz through Music Assistant
- **THEN** the row shows the Qobuz node, a wave, the Music Assistant node, a wave and the sound box, in that order

#### Scenario: Native service
- **WHEN** the source is Spotify without a route
- **THEN** the row shows the Spotify node, one wave and the sound box

#### Scenario: No logos
- **WHEN** neither the source nor the route has a `logoUrl`
- **THEN** the nodes show an icon and "Qobuz", and "Music Assistant"

#### Scenario: Route unavailable
- **WHEN** the route "Music Assistant" is unavailable
- **THEN** the Music Assistant node is greyed with "unavailable" under it, and both waves are grey and still

### Requirement: Signal colours
The waves, the ports and the sound box's border SHALL take one colour from
`signal.class`: yellow for Hi-Res, teal for lossless, violet for lossy and
grey for unknown. The wave after the route SHALL use a lighter shade of
the same colour. The album's colours SHALL NOT change them.

#### Scenario: Hi-Res through Music Assistant
- **WHEN** `signal.class` is "hi-res" and the source has a route
- **THEN** the first wave is yellow and the second a lighter yellow

#### Scenario: Upsampled CD quality
- **WHEN** `signal.class` is "lossless" and `delivered` is 24/96
- **THEN** the waves and the sound box are teal

### Requirement: Sound box
The sound box SHALL show the quality the player receives (`delivered`, or
else `source`): the bit depth and sample rate ("24 bit / 96 kHz") or the
bitrate ("320 kbps") with tabular figures, the codec on the next line
("lossy" for a bitrate without a codec), and
a third, quieter line with the changes ("normalised (−6 dB)", "resampled
from 16 / 44.1", "24 bit from 16", "DSP", joined with " · "), or
"untouched" without changes. It SHALL show the Hi-Res logo before the
lines when the class is Hi-Res: the Hi-Res logo when
`signal.hiResLogoUrl` is present, otherwise a yellow "HI-RES" text tag. It
SHALL show "quality not reported" when the class is unknown.

#### Scenario: Untouched Hi-Res
- **WHEN** `signal.class` is "hi-res", `source` is FLAC 24/96, there is no route and the Hi-Res logo is supplied
- **THEN** the box shows the Hi-Res logo, "24 bit / 96 kHz", "FLAC" and "untouched"

#### Scenario: Upsampled CD quality
- **WHEN** `source` is FLAC 16/44.1, `delivered` is FLAC 24/96 and the only change is "resampled"
- **THEN** the box shows "24 bit / 96 kHz", "FLAC" and "resampled from 16 / 44.1", and no Hi-Res logo or tag

#### Scenario: Hi-Res without the logo
- **WHEN** `signal.class` is "hi-res" and `signal.hiResLogoUrl` is `null`
- **THEN** the box shows a "HI-RES" tag before the lines

#### Scenario: Lossy
- **WHEN** `source` is 320 kbps, without a codec
- **THEN** the box shows "320 kbps", "lossy" and "untouched"

#### Scenario: Unknown
- **WHEN** `signal.class` is "unknown"
- **THEN** the box shows "quality not reported"

### Requirement: Wave motion
A pulse of light SHALL run towards the end every 4 seconds, and each
arrival port SHALL light up as the pulse arrives. With a route, the pulse
SHALL run along the first wave and, as it arrives, go on along the second,
so the light travels from the source to the sound box.
Between pulses no animation SHALL run in the signal path, so the browser
can stop drawing frames. The motion SHALL use only transform and opacity. With reduced motion, while paused, and on an
unavailable route, the waves SHALL stand still, with no pulses.

#### Scenario: Playing
- **WHEN** a track plays
- **THEN** the pulses move towards the sound box

#### Scenario: From one end to the other
- **WHEN** a track plays through Music Assistant
- **THEN** the pulse on the second wave starts as the pulse on the first wave arrives

#### Scenario: Between pulses
- **WHEN** a pulse has reached the end and the next one is not due yet
- **THEN** no animation runs in the signal path

#### Scenario: Reduced motion
- **WHEN** the browser prefers reduced motion
- **THEN** no pulse moves

#### Scenario: Paused
- **WHEN** the player pauses
- **THEN** no pulse moves

### Requirement: Signal path fits
The signal path SHALL stay on one row, with no part wrapping. The source
node and the sound box's first two lines SHALL never shrink. When space
runs out, the waves SHALL get shorter down to a minimum length, then a
wave's label SHALL be left out when it is wider than its wave, then the
sound box's third line SHALL be shortened with "…", and last the route's
name SHALL be left out, leaving its icon.

#### Scenario: Landscape
- **WHEN** the screen is in landscape and the source is Qobuz through Music Assistant with "normalised (−6 dB)"
- **THEN** every part of the row shows in full

#### Scenario: Route without a logo in landscape
- **WHEN** the screen is in landscape, neither the source nor the route has a logo, and the changes are "resampled", "normalised (−7.4 dB)" and "DSP"
- **THEN** the row doesn't overflow, and the route keeps at least its icon

#### Scenario: Narrow portrait
- **WHEN** the screen is in portrait and the row would be wider than the column
- **THEN** the labels or the third line give way first, and no part wraps or overflows

## MODIFIED Requirements

### Requirement: Playing screen
While a track is playing, the kiosk SHALL show the artwork, title, artists,
album, a progress bar with elapsed time and duration, and the signal path
(see "Signal path"). Titles longer than two lines SHALL end with an
ellipsis.

#### Scenario: Track with all data
- **WHEN** the snapshot is playing "Example Song" by "Example Artist" with artwork, position, the source "Qobuz" and the quality FLAC at 16 bits and 44 100 Hz
- **THEN** the screen shows the artwork, "Example Song", "Example Artist", the progress, and a signal path from Qobuz to a sound box with "16 bit / 44.1 kHz"

#### Scenario: No position
- **WHEN** the snapshot has no position
- **THEN** the progress bar and times are hidden

### Requirement: Album colours
When the snapshot has album colours, the kiosk SHALL tint its background
with the background colour and use the accent colour for the progress bar.
Text SHALL stay near-white for contrast. Without colours, the kiosk SHALL
use its default dark theme.

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
in 20 to 30 seconds), the input's name, the session label with an input
icon, and "for N min" since the input's signal started, in whole minutes,
or "waiting for signal" while there is none yet. It SHALL show no artwork,
track time, progress bar, queue or signal path. With reduced motion the
disc SHALL stand still.

#### Scenario: Optical input
- **WHEN** the snapshot has the optical input named "Example Input", whose signal started 23 minutes ago, with the session label "Optical input"
- **THEN** the screen shows the time, the turning disc, "Example Input", an input icon with "Optical input" and "for 23 min", and no progress bar, "Up next" or signal path

#### Scenario: Just started
- **WHEN** the input's signal started 20 seconds ago
- **THEN** the screen shows "for 0 min"

#### Scenario: Waiting for a signal
- **WHEN** the input waits for a signal
- **THEN** the screen shows "waiting for signal" instead of the time it plays

#### Scenario: Reduced motion
- **WHEN** the browser prefers reduced motion and an input plays
- **THEN** the disc does not turn

### Requirement: Update only what changed
On a track change, the kiosk SHALL change only the track's title, artists
and album: the old text fades out, then the new text fades in. The cover
and the backdrop SHALL stay when the new track has the
same album and artists. A new cover SHALL replace the old one only once it
has loaded, with a cross-fade, so the cover is never blank. The signal
path, progress and "Up next" SHALL update in place, without fading; the
waves keep running.

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
- **THEN** only the signal path's colours and texts change, the waves are the same elements, and nothing fades

### Requirement: Steady portrait layout
In portrait, the cover, the title (room for two lines), the artists (one
line), the album (one line), the progress, the signal path (one row) and
"Up next" SHALL each keep a fixed place on the screen, so changing content
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

## REMOVED Requirements

### Requirement: Source badge
**Reason**: The signal path's source and route nodes replace the badge.
**Migration**: None; the kiosk is the only consumer.

### Requirement: Quality on one line
**Reason**: The sound box and the wave labels replace the quality line.
**Migration**: None; see "Signal path fits".

### Requirement: Hi-Res badge
**Reason**: The sound box shows the Hi-Res logo or tag instead.
**Migration**: None; see "Sound box" and "Signal colours".
