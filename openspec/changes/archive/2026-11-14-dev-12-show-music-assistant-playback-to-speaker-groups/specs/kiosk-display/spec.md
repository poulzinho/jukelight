## ADDED Requirements

### Requirement: Signal diagram
The playing screen SHALL show how the music reaches every speaker as a
diagram of nodes joined by lines: the source (its logo, or its icon and
name) with what it streams written under or beside it ("streams 16 bit /
44.1 kHz · FLAC", nothing when the quality is not reported); Music
Assistant's logo when it is the route; one row per other speaker of the
session (`speakers`: an icon by kind, the name, and the format it receives
with its codec, or "not playing", dimmed, when it doesn't play); the player
as a streamer card; and the player's outputs (`outputs`: an icon by kind
and the name) hanging from the streamer. Lines SHALL run straight with
right angles and rounded corners, branch at junction dots, and end in a
port. The diagram SHALL start at the left edge of the progress bar. With a
queue the diagram SHALL run top to bottom; without a queue it
SHALL run left to right, the source on the left and the outputs on the
right.

#### Scenario: A group with a queue
- **WHEN** Qobuz plays through Music Assistant to the player and the speakers "Kitchen" and "Office", the player has the outputs "Example Speaker" and a subwoofer, and the queue has entries
- **THEN** the diagram runs top to bottom: the Qobuz logo with "streams …", Music Assistant's logo, the rows "Kitchen" and "Office" with their formats, the streamer, and under it "Example Speaker" and "Subwoofer"

#### Scenario: Native service without a queue
- **WHEN** Spotify Connect plays at 320 kbps with no queue
- **THEN** the diagram runs left to right: the Spotify logo with "streams 320 kbps lossy" under it, a line to the streamer, and the outputs to its right

#### Scenario: AirPlay from a phone
- **WHEN** the session is AirPlay with no quality reported
- **THEN** the source shows an AirPlay icon and "AirPlay", its line has no label, and the streamer's first line is "AirPlay"

#### Scenario: Music Assistant unavailable
- **WHEN** the route Music Assistant is unavailable
- **THEN** the source shows Music Assistant's logo greyed with "unavailable", and the streamer shows only the room

#### Scenario: A speaker dropped out
- **WHEN** a speaker of the session doesn't play
- **THEN** its row is dimmed and shows "not playing" instead of a format

### Requirement: Streamer card
The player SHALL be drawn as a streamer: a low card with a lid, a status
light and feet. Inside it SHALL show the format the player receives ("24
bit / 96 kHz" or "320 kbps"; or what is known, such as "AirPlay", when no
quality is reported), then its codec with what changed on the way ("FLAC ·
resampled from 16 / 44.1", "PCM · decoded from 320 kbps"; "lossy" for a
bitrate without a codec), then the room. A PCM codec SHALL have a small
icon of raw samples. When the whole path is Hi-Res, the card SHALL show the
Hi-Res logo (or the "HI-RES" tag) on its left. The maker's mark
(`device.markUrl`) SHALL sit in the card's top-right corner, when supplied.

#### Scenario: Hi-Res with both logos
- **WHEN** the class is Hi-Res, the Hi-Res logo and the maker's mark are supplied, and the player receives FLAC 24/96
- **THEN** the card shows the Hi-Res logo, "24 bit / 96 kHz", "FLAC", the room, and the mark in its corner

#### Scenario: Group member receiving PCM
- **WHEN** the player receives PCM 16/44.1 in a group
- **THEN** the card shows "16 bit / 44.1 kHz" and the PCM icon with "PCM"

#### Scenario: Lossy source decoded
- **WHEN** the source is 320 kbps and the player receives PCM 16/44.1
- **THEN** the card's second line is "PCM · decoded from 320 kbps"

### Requirement: Line colours
Each line, its ports and the card it leads to SHALL take the colour of the
quality class of what flows on it: yellow for Hi-Res, teal for lossless,
violet for lossy and grey for unknown. Lines after the route SHALL use a
lighter shade, and each speaker's line its own class from the API (the
lower of the source's and what it receives). Lines SHALL brighten gently towards their end. The
album's colours SHALL NOT change them.

#### Scenario: One lossy speaker in a lossless group
- **WHEN** the group is lossless and the speaker "Garden" receives AAC at 256 kbps
- **THEN** the line, icon and format of "Garden" are violet and the others teal

#### Scenario: Upsampled CD quality
- **WHEN** the class is lossless and the player receives 24/96
- **THEN** the lines and the streamer are teal and show no Hi-Res logo

### Requirement: Pulse along the diagram
A pulse of light SHALL run once along the diagram when something changes:
when a track starts (also when the screen first shows it), when the source,
the route or the speakers change, when the quality changes, and when
playback resumes after a pause. It SHALL run from the source to Music
Assistant, then down the line to the streamer, each speaker's branch
starting as the pulse passes its junction, then on from the streamer to
its outputs; each port SHALL light
up as the pulse arrives. The pulse SHALL move in about 15 steps a second,
using only transform and opacity. Between pulses, with reduced motion,
while paused and while the route is unavailable, no animation SHALL run in
the diagram.

#### Scenario: Track starts in a group
- **WHEN** a track starts playing to a group of three speakers
- **THEN** one pulse runs from the source, reaches Music Assistant, then each of the three branches in turn as it passes their junctions, then the streamer and its outputs

#### Scenario: Nothing changes
- **WHEN** a track keeps playing and nothing about it changes for a minute after its pulse
- **THEN** no further pulse runs and no animation runs in the diagram

#### Scenario: Reduced motion
- **WHEN** the browser prefers reduced motion
- **THEN** no pulse moves

### Requirement: Diagram fits
No part of the diagram SHALL leave its area, come within 3vmin of the
footer, run into the queue's column, or run over a logo. Names, the codec line and the room
SHALL be shortened with "…" to the room they have; the format on the
streamer SHALL never be shortened (the maker's mark gives way first). When
the diagram runs top to bottom and is too tall for the space above the
footer, it SHALL give way in this order: speakers fold into a "+ N more"
row, from the last; the source's line gets shorter and its label drops
"streams"; the outputs move onto one line as icons; last, the outputs are
left out. Without a queue the diagram SHALL run left to right only when
every part fits in full, the outputs' names included; otherwise it SHALL
run top to bottom.

#### Scenario: Many speakers in portrait
- **WHEN** the session plays to five other speakers in portrait
- **THEN** as many rows as fit show, followed by "+ N more" for the rest, and nothing comes within 3vmin of the footer

#### Scenario: Long names
- **WHEN** a speaker is named "Upstairs Bathroom Speaker" and the output "Example Speaker With A Very Long Name"
- **THEN** both are shortened with "…" and nothing overlaps the formats or the queue

#### Scenario: A wide logo without a queue
- **WHEN** there is no queue and the source's logo, the line's label, the streamer and the outputs don't fit across the width
- **THEN** the diagram runs top to bottom

## MODIFIED Requirements

### Requirement: Playing screen
While a track is playing, the kiosk SHALL show the artwork, title, artists,
album, a progress bar with elapsed time and duration, and the signal
diagram (see "Signal diagram"). With a queue, the area below the progress
SHALL have two columns, the diagram on the left and "Up next" on the right,
both aligned at the top; without a queue the diagram SHALL take the full
width. Titles longer than two lines SHALL end with an ellipsis.

#### Scenario: Track with all data
- **WHEN** the snapshot is playing "Example Song" by "Example Artist" with artwork, position, the source "Qobuz" and the quality FLAC at 16 bits and 44 100 Hz
- **THEN** the screen shows the artwork, "Example Song", "Example Artist", the progress, and a diagram from Qobuz to the streamer with "16 bit / 44.1 kHz"

#### Scenario: No position
- **WHEN** the snapshot has no position
- **THEN** the progress bar and times are hidden

#### Scenario: Queue beside the diagram
- **WHEN** the snapshot has a queue
- **THEN** "Up next" sits in the right column, its heading level with the top of the diagram

### Requirement: Up next
The kiosk SHALL show an "Up next" list only when the queue is available and
not empty, in the right column beside the diagram: as many entries as fit
above the footer, at most 5. Titles and artists SHALL stay on one line,
ending with "…" when too long.

#### Scenario: Queue available
- **WHEN** the queue holds 5 upcoming entries and the screen is landscape
- **THEN** "Up next" shows the first 4 entries

#### Scenario: No queue
- **WHEN** the queue is not applicable
- **THEN** no "Up next" list is shown

#### Scenario: Long title
- **WHEN** an entry's title is wider than the column
- **THEN** it ends with "…" on one line

### Requirement: Steady portrait layout
In portrait, the cover, the title (room for two lines), the artists (one
line), the album (one line), the progress, and the area below it (the
diagram and "Up next") SHALL each keep a fixed place on the screen, so
changing content never moves the other parts.

#### Scenario: No queue
- **WHEN** the screen is in portrait and one track has three "Up next" entries and another has none
- **THEN** the cover and the title are in the same place for both

#### Scenario: Long title
- **WHEN** the screen is in portrait and one track's title fits on one line and another's needs two
- **THEN** the cover, the artists and "Up next" are in the same place for both

#### Scenario: Missing album
- **WHEN** the screen is in portrait and one track has an album and another has none
- **THEN** the progress and "Up next" are in the same place for both

### Requirement: Update only what changed
On a track change, the kiosk SHALL change only the track's title, artists
and album: the old text fades out, then the new text fades in, however
often further snapshots arrive meanwhile. The cover and the backdrop SHALL
stay when the new track has the same album and artists. A new cover SHALL
replace the old one only once it has loaded, with a cross-fade, so the
cover is never blank. The signal diagram, progress and "Up next" SHALL
update in place, without fading.

#### Scenario: Next track on the same album
- **WHEN** the track changes to another track with the same album and artists
- **THEN** the cover is still the same element and the title fades in

#### Scenario: Old text fades out first
- **WHEN** the track changes
- **THEN** the old title is still shown, fading out, and the new title replaces it once the fade-out has finished

#### Scenario: Snapshots keep arriving during the change
- **WHEN** the track changes and further snapshots with the same track arrive every 200 ms
- **THEN** the new title replaces the old one once the fade-out has finished, as without them

#### Scenario: Track with a new cover
- **WHEN** the track changes to a track with a different album and its cover hasn't loaded yet
- **THEN** the old cover is still shown, and the new one replaces it once it has loaded

#### Scenario: Quality changes
- **WHEN** only the quality of the current track changes
- **THEN** only the signal diagram's colours and texts change, its lines are the same elements, and nothing fades

## REMOVED Requirements

### Requirement: Signal path
**Reason**: Replaced by "Signal diagram", which shows every speaker of the session.
**Migration**: None; the kiosk is the only consumer.

### Requirement: Signal colours
**Reason**: Replaced by "Line colours", per line and per speaker.
**Migration**: None.

### Requirement: Sound box
**Reason**: Replaced by "Streamer card".
**Migration**: None.

### Requirement: Wave motion
**Reason**: Replaced by "Pulse along the diagram": straight lines instead of waves, the pulse along every branch.
**Migration**: None.

### Requirement: Signal path fits
**Reason**: Replaced by "Diagram fits".
**Migration**: None.
