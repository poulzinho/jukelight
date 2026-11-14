## ADDED Requirements

### Requirement: Typeface
The kiosk SHALL set all its text, the diagram's included, in Figtree, a
font bundled with the kiosk under a licence that allows redistribution
(SIL Open Font License); it SHALL load no font from the internet. Text the
kiosk measures to fit (the diagram's names and formats, the title) SHALL be
measured again once the font has loaded.

#### Scenario: Bundled font
- **WHEN** the playing screen is shown
- **THEN** its text is set in Figtree, served by Jukelight itself

#### Scenario: Measured after loading
- **WHEN** the diagram is first laid out before the font has loaded
- **THEN** it is laid out again once the font has loaded, and nothing runs into the formats or past its area

## MODIFIED Requirements

### Requirement: Playing screen
While a track is playing, the kiosk SHALL show the artwork, title, artists,
album, a progress bar with elapsed time and duration, and the signal
diagram (see "Signal diagram"). The artists SHALL follow a microphone icon
and the album a disc icon, both dimmed. With a queue, the area below the
progress SHALL have two columns, the diagram on the left and "Up next" on
the right, both aligned at the top; without a queue the diagram SHALL take
the full width. A title too long for two lines at the largest size SHALL
step down in size until it fits the same room: two lines at smaller
sizes, then three lines at the smallest; only a title that doesn't fit
even then SHALL end with an ellipsis.

#### Scenario: Track with all data
- **WHEN** the snapshot is playing "Example Song" by "Example Artist" with artwork, position, the source "Qobuz" and the quality FLAC at 16 bits and 44 100 Hz
- **THEN** the screen shows the artwork, "Example Song", "Example Artist", the progress, and a diagram from Qobuz to the streamer with "16 bit / 44.1 kHz"

#### Scenario: No position
- **WHEN** the snapshot has no position
- **THEN** the progress bar and times are hidden

#### Scenario: Queue beside the diagram
- **WHEN** the snapshot has a queue
- **THEN** "Up next" sits in the right column, its heading level with the top of the diagram

#### Scenario: Short title
- **WHEN** the title is "Example Song"
- **THEN** it is shown at the largest size, in full

#### Scenario: Long classical title
- **WHEN** the title is "J.S. Bach: Violin Concerto No. 1 in A Minor, BWV 1041: I. Allegro moderato" in portrait
- **THEN** it is shown in full, smaller, without an ellipsis

#### Scenario: Title too long for any size
- **WHEN** the title is longer than three lines at the smallest size
- **THEN** it is shown on three lines at the smallest size, ending with "…"

#### Scenario: Artists and album with icons
- **WHEN** the snapshot names the artist "Example Artist" and the album "Example Album"
- **THEN** a microphone icon precedes "Example Artist" and a disc icon precedes "Example Album"

### Requirement: Steady portrait layout
In portrait, the cover, the title (room for two lines at the largest
size, whatever size the title takes), the artists (one line), the album
(one line), the progress, and the area below it (the diagram and "Up
next") SHALL each keep a fixed place on the screen, so changing content
never moves the other parts.

#### Scenario: No queue
- **WHEN** the screen is in portrait and one track has three "Up next" entries and another has none
- **THEN** the cover and the title are in the same place for both

#### Scenario: Long title
- **WHEN** the screen is in portrait and one track's title fits on one line and another's needs two
- **THEN** the cover, the artists and "Up next" are in the same place for both

#### Scenario: Title stepped down to three lines
- **WHEN** the screen is in portrait and one track's title fits on one line and another's is shown on three smaller lines
- **THEN** the artists, the progress and "Up next" are in the same place for both

#### Scenario: Missing album
- **WHEN** the screen is in portrait and one track has an album and another has none
- **THEN** the progress and "Up next" are in the same place for both

### Requirement: Streamer card
The player SHALL be drawn as a streamer: a low card with a lid, a status
light and feet. The lid SHALL hold the maker's mark (`device.markUrl`, the
icon only), when supplied, followed by the room. The card's body SHALL
show the format the player receives ("24 bit / 96 kHz" or "320 kbps"; or
what is known, such as "AirPlay", when no quality is reported), then its
codec with what changed on the way ("FLAC · resampled from 16 / 44.1",
"PCM · decoded from 320 kbps"; "lossy" for a bitrate without a codec). A
PCM codec SHALL have a small icon of raw samples. When the whole path is
Hi-Res, the card SHALL show the Hi-Res logo (or the "HI-RES" tag) on its
left.

#### Scenario: Hi-Res with both logos
- **WHEN** the class is Hi-Res, the Hi-Res logo and the maker's mark are supplied, and the player receives FLAC 24/96
- **THEN** the card shows the mark and the room in its lid, and the Hi-Res logo, "24 bit / 96 kHz" and "FLAC" in its body

#### Scenario: No mark
- **WHEN** no maker's mark is supplied
- **THEN** the lid shows the room alone

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
lower of the source's and what it receives). Lines SHALL brighten gently
towards their end. Lines, ports and junction dots SHALL be drawn thin and
partly transparent, so the nodes and the formats lead. The album's colours
SHALL NOT change them.

#### Scenario: One lossy speaker in a lossless group
- **WHEN** the group is lossless and the speaker "Garden" receives AAC at 256 kbps
- **THEN** the line, icon and format of "Garden" are violet and the others teal

#### Scenario: Upsampled CD quality
- **WHEN** the class is lossless and the player receives 24/96
- **THEN** the lines and the streamer are teal and show no Hi-Res logo

#### Scenario: Lines stay in the background
- **WHEN** the diagram is drawn
- **THEN** its lines, ports and junction dots are partly transparent and thinner than the streamer card's border

### Requirement: Diagram fits
No part of the diagram SHALL leave its area, come within 3vmin of the
footer, run into the queue's column, or run over a logo. Names, the codec
line and the room SHALL be shortened with "…" to the room they have; the
format on the streamer SHALL never be shortened. When the diagram runs top
to bottom and is too tall for the space above the footer, it SHALL give
way in this order: speakers fold into a "+ N more" row, from the last; the
source's line gets shorter and its label drops "streams"; the outputs move
onto one line as icons; last, the outputs are left out. Without a queue the
diagram SHALL run left to right only when every part fits in full, the
outputs' names included; otherwise it SHALL run top to bottom.

#### Scenario: Many speakers in portrait
- **WHEN** the session plays to five other speakers in portrait
- **THEN** as many rows as fit show, followed by "+ N more" for the rest, and nothing comes within 3vmin of the footer

#### Scenario: Two speakers in portrait
- **WHEN** the session plays to two other speakers in portrait, with a queue
- **THEN** both rows show, without "+ N more"

#### Scenario: Long names
- **WHEN** a speaker is named "Upstairs Bathroom Speaker" and the output "Example Speaker With A Very Long Name"
- **THEN** both are shortened with "…" and nothing overlaps the formats or the queue

#### Scenario: Long room name
- **WHEN** the room is "Example Very Long Room Name Upstairs" and the maker's mark is supplied
- **THEN** the room is shortened with "…" in the lid, after the mark

#### Scenario: A wide logo without a queue
- **WHEN** there is no queue and the source's logo, the line's label, the streamer and the outputs don't fit across the width
- **THEN** the diagram runs top to bottom
