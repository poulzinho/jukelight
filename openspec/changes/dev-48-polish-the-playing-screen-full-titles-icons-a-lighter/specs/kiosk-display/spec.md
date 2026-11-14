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

### Requirement: Signal diagram
The playing screen SHALL show how the music reaches every speaker as a
diagram of nodes joined by lines: the source (its logo, or its icon and
name) with what it streams on the same line: a gentle wave from the
logo's right edge to the format in its short form with the codec ("24/44.1
FLAC"), aligned with and set like the speakers' formats, in the source's
class colour (nothing when the quality is not reported; a logo too wide to
leave at least a short wave is shown smaller); Music Assistant's
logo when it is the route; one row per other speaker of the session
(`speakers`: an icon by kind, the name, and the format it receives with
its codec, or "not playing", dimmed, when it doesn't play); the player as
a streamer card; and the player's outputs (`outputs`: an icon by kind and
the name) hanging from the streamer. Lines SHALL run straight with right
angles and rounded corners, branch at junction dots, and end in a port.
The diagram SHALL start at the left edge of the progress bar. With a queue
the diagram SHALL run top to bottom; without a queue it SHALL run left to
right, the source on the left and the outputs on the right.

#### Scenario: A group with a queue
- **WHEN** Qobuz plays through Music Assistant to the player and the speakers "Kitchen" and "Office", the player has the outputs "Example Speaker" and a subwoofer, and the queue has entries
- **THEN** the diagram runs top to bottom: the Qobuz logo with its wave to the format, Music Assistant's logo, the rows "Kitchen" and "Office" with their formats, the streamer, and under it "Example Speaker" and "Subwoofer"

#### Scenario: What the source streams, as a wave
- **WHEN** Qobuz streams FLAC 24/44.1 through Music Assistant to the speaker "Office", which receives FLAC 24/44.1
- **THEN** a wave runs from the Qobuz logo to "24/44.1 FLAC" on the logo's line, and that format ends where Office's "24/44.1 FLAC" ends, set the same way

#### Scenario: A wide logo
- **WHEN** the source's logo is a wordmark so wide that less than a short wave would remain
- **THEN** the logo is shown smaller, and the wave keeps its minimum length

#### Scenario: Native service without a queue
- **WHEN** Spotify Connect plays at 320 kbps with no queue
- **THEN** the diagram runs left to right: the Spotify logo with "320 kbps lossy" under it, a line to the streamer, and the outputs to its right

#### Scenario: AirPlay from a phone
- **WHEN** the session is AirPlay with no quality reported
- **THEN** the source shows an AirPlay icon and "AirPlay", its line has no label, and the streamer's first line is "AirPlay"

#### Scenario: Music Assistant unavailable
- **WHEN** the route Music Assistant is unavailable
- **THEN** the source shows Music Assistant's logo greyed with "unavailable", and the streamer shows only the room

#### Scenario: A speaker dropped out
- **WHEN** a speaker of the session doesn't play
- **THEN** its row is dimmed and shows "not playing" instead of a format

### Requirement: Playing screen
While a track is playing, the kiosk SHALL show the artwork, title, artists,
album, a progress bar with elapsed time and duration, and the signal
diagram (see "Signal diagram"). The artists SHALL follow a dimmed icon of
one person, or of a group of people when the track has two or more
artists or its one artist's name contains a word for a group (orchestra,
philharmonic, ensemble, quartet, trio, choir or band, also in Spanish,
French, German and Italian: orquesta, orchestre, orchester, orchestra,
coro, chœur, chor, cuarteto, quatuor, quartett, quartetto); the album SHALL
follow a dimmed disc icon. In portrait everything below the cover (the
title, artists, album, progress bar, the diagram and "Up next") SHALL line
up with the cover's edges; the progress bar, with space below it,
separates the track from what plays it. Artists too long for one
line SHALL wrap to a second line, then end with "…". With a queue, the area below the
progress SHALL have two columns, the diagram on the left and "Up next" on
the right, both aligned at the top; without a queue the diagram SHALL take
the full width. The title SHALL take the largest size that fits its room
(one line at the largest size): one line at the two largest sizes, then
two lines at smaller sizes, then three lines at the smallest; only a title
that doesn't fit even then SHALL end with an ellipsis. The title's
letters SHALL never be clipped: each line SHALL be tall enough for the
tails of letters such as "g" and for accents.

#### Scenario: Track with all data
- **WHEN** the snapshot is playing "Example Song" by "Example Artist" with artwork, position, the source "Qobuz" and the quality FLAC at 16 bits and 44 100 Hz
- **THEN** the screen shows the artwork, "Example Song", "Example Artist", the progress, and a diagram from Qobuz to the streamer with "16 bit / 44.1 kHz"

#### Scenario: No position
- **WHEN** the snapshot has no position
- **THEN** the progress bar and times are hidden

#### Scenario: Queue beside the diagram
- **WHEN** the snapshot has a queue
- **THEN** "Up next" sits in the right column, its heading level with the top of the diagram

#### Scenario: Aligned with the cover in portrait
- **WHEN** the screen is in portrait
- **THEN** the title, artists, album, progress bar, the diagram and "Up next" start and end at the cover's left and right edges

#### Scenario: Short title
- **WHEN** the title is "Example Song"
- **THEN** it is shown at the largest size, on one line

#### Scenario: Medium title
- **WHEN** the title is "Example Song Title (Remastered 2024)" in portrait and doesn't fit one line at the two largest sizes
- **THEN** it is shown on two lines at a smaller size

#### Scenario: Two artists on two lines
- **WHEN** the artists "Example Violinist" and "Example Chamber Orchestra" don't fit one line
- **THEN** they are shown in full on two lines, and the album and progress move down by one artist line

#### Scenario: Long classical title
- **WHEN** the title is "J.S. Bach: Violin Concerto No. 1 in A Minor, BWV 1041: I. Allegro moderato" in portrait
- **THEN** it is shown in full, smaller, without an ellipsis

#### Scenario: Title too long for any size
- **WHEN** the title is longer than three lines at the smallest size
- **THEN** it is shown on three lines at the smallest size, ending with "…"

#### Scenario: Letters with tails and accents
- **WHEN** the title is "Example Song" or a long classical title, at any size
- **THEN** the tails of its "g", "p" and "y" and its accents show in full, inside the title's room

#### Scenario: Artists and album with icons
- **WHEN** the snapshot names the artist "Example Artist" and the album "Example Album"
- **THEN** a one-person icon precedes "Example Artist" and a disc icon precedes "Example Album"

#### Scenario: Several artists
- **WHEN** the snapshot names the artists "Example Singer" and "Guest Artist"
- **THEN** a group icon precedes "Example Singer, Guest Artist"

#### Scenario: A group's name
- **WHEN** the snapshot names the one artist "Example Chamber Orchestra"
- **THEN** a group icon precedes it

#### Scenario: A name without a group word
- **WHEN** the snapshot names the one artist "The Example Collective"
- **THEN** a one-person icon precedes it

### Requirement: Up next
The kiosk SHALL show an "Up next" list only when the queue is available and
not empty, in the right column beside the diagram: as many entries as fit
above the page's bottom margin, at most 5. Entries SHALL be grouped:
- **By work:** the part of a title before its last ": " or " - " is a work
  when two or more consecutive entries share it, or the first entry shares
  it with the current track. Consecutive entries of one work SHALL share
  the work as a heading and show only the rest of their titles; the first
  entry of the group SHALL show its artists, later ones only when the
  artists change.
- **By artist:** otherwise, two or more consecutive entries with the same
  main (first) artist SHALL share that artist as a heading and show only
  their titles, with any other artists as "with …".
- **Single entries** SHALL show their title and artists.

A group SHALL start again after any entry in between. Headings and
artists SHALL stay on one line and titles on at most two, ending with "…"
when too long. A heading SHALL never be shown without its first entry
below it.

#### Scenario: Queue available
- **WHEN** the queue holds 5 upcoming entries by five different artists and the screen is landscape
- **THEN** "Up next" shows the first 4 entries

#### Scenario: No queue
- **WHEN** the queue is not applicable
- **THEN** no "Up next" list is shown

#### Scenario: Movements of one work
- **WHEN** the current track is "Example Composer: Example Concerto, Op. 1: I. Allegro" by "Example Violinist" and the next entries are its "II. Andante" and "III. Presto" by the same violinist
- **THEN** "Up next" shows the heading "Example Composer: Example Concerto, Op. 1", then "II. Andante" with "Example Violinist", then "III. Presto" without an artist

#### Scenario: Performer changes within a work
- **WHEN** consecutive entries of one work are by "Example Pianist", "Another Example Pianist" and "Another Example Pianist"
- **THEN** the artist shows on the first and second entries only

#### Scenario: A work without the composer in the title
- **WHEN** the next entries are "Symphony No. 5 in C Minor, Op. 67: II. Andante con moto" and "Symphony No. 5 in C Minor, Op. 67: III. Allegro"
- **THEN** they share the heading "Symphony No. 5 in C Minor, Op. 67"

#### Scenario: Dash as the separator
- **WHEN** the next entries are "Piano Sonata No. 14 - II. Allegretto" and "Piano Sonata No. 14 - III. Presto agitato"
- **THEN** they share the heading "Piano Sonata No. 14"

#### Scenario: A lone title with a separator
- **WHEN** an entry titled "Interlude: The Long Way Home" shares its beginning with no neighbour and not with the current track
- **THEN** it is shown whole, as a single entry with its artist

#### Scenario: Album by one artist
- **WHEN** the next entries are "Example Song A", "Example Song B" and "Example Song C", all by "Example Singer"
- **THEN** "Up next" shows the heading "Example Singer" and the three titles without an artist line

#### Scenario: Featured artist
- **WHEN** consecutive entries by "Example Singer" include one by "Example Singer" and "Guest Artist"
- **THEN** that entry stays in the group and shows "with Guest Artist"

#### Scenario: An artist interrupted
- **WHEN** two entries by "Example Singer" are followed by one by "Example Band" and then two by "Example Singer"
- **THEN** the "Example Singer" heading is shown before both pairs, and the "Example Band" entry shows its title and artist

#### Scenario: No artist
- **WHEN** two consecutive entries have no artists
- **THEN** they are not grouped and show only their titles

#### Scenario: Long title
- **WHEN** an entry's title is wider than two lines of the column
- **THEN** it ends with "…" on its second line

#### Scenario: Heading at the bottom
- **WHEN** in portrait a group's heading would fit above the page's bottom margin but its first entry would not
- **THEN** neither is shown

### Requirement: Steady portrait layout
In portrait, the cover, the title (room for one line at the largest size,
whatever size the title takes), the artists (one line), the album (one
line), the progress, and the area below it (the diagram and "Up next")
SHALL each keep a fixed place on the screen, so changing content never
moves the other parts. The one exception: artists that need a second line
move the album, the progress and the area below down by that line, for
that track only.

#### Scenario: No queue
- **WHEN** the screen is in portrait and one track has three "Up next" entries and another has none
- **THEN** the cover and the title are in the same place for both

#### Scenario: Long title
- **WHEN** the screen is in portrait and one track's title fits on one line and another's needs two
- **THEN** the cover, the artists and "Up next" are in the same place for both

#### Scenario: Title stepped down to two lines
- **WHEN** the screen is in portrait and one track's title fits on one line and another's is shown on two smaller lines
- **THEN** the artists, the progress and "Up next" are in the same place for both

#### Scenario: Back from the idle screen
- **WHEN** playback stops, the idle screen shows, and then the same track plays again
- **THEN** from its first frame the playing screen shows the diagram and "Up next" at their final size and place, without a jump

#### Scenario: Missing album
- **WHEN** the screen is in portrait and one track has an album and another has none
- **THEN** the progress and "Up next" are in the same place for both

### Requirement: Streamer card
The player SHALL be drawn as a streamer: a low card with a lid, a status
light and feet. The lid SHALL hold the maker's mark (`device.markUrl`, the
icon only), when supplied, followed by the room, set like a speaker's name
(the same size, weight and colour): the streamer matters as much as the
speakers. The card's body SHALL
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

#### Scenario: Room set like a speaker
- **WHEN** the diagram shows the speaker "Kitchen" and the room "Living Room"
- **THEN** "Living Room" has the same size, weight and colour as "Kitchen"

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
- **THEN** its lines, ports and junction dots are partly transparent and thinner than the streamer card's border, and the card's border is as thick as the speakers' icons

### Requirement: Pulse along the diagram
A pulse of light SHALL run once along the diagram when something changes:
when a track starts (also when the screen first shows it), when the source,
the route or the speakers change, when the quality changes, and when
playback resumes after a pause. As it starts, the source's wave and its
format SHALL light up at once and settle back; the wave itself carries no
moving light. The pulse SHALL run from the source to Music Assistant, then
down the line to the streamer, each speaker's branch starting as the pulse
passes its junction, then on from the streamer to its outputs; each port
SHALL light up as the pulse arrives. The pulse SHALL move in about 15
steps a second, using only transform and opacity. Between pulses, with
reduced motion, while paused and while the route is unavailable, no
animation SHALL run in the diagram.

#### Scenario: Track starts in a group
- **WHEN** a track starts playing to a group of three speakers
- **THEN** the wave and its format light up at once, and one pulse runs from the source, reaches Music Assistant, then each of the three branches in turn as it passes their junctions, then the streamer and its outputs

#### Scenario: Nothing changes
- **WHEN** a track keeps playing and nothing about it changes for a minute after its pulse
- **THEN** no further pulse runs and no animation runs in the diagram

#### Scenario: Reduced motion
- **WHEN** the browser prefers reduced motion
- **THEN** no pulse moves and the wave doesn't light up

### Requirement: Device signature
Every screen except the playing screen with its signal diagram SHALL show
the player's maker and name in a quiet footer: the maker's logo when
`device.logoUrl` is present, otherwise a streamer icon and the maker's
name when the maker is a known brand; then the player's name. With an
unknown maker, only the player's name SHALL show. On the playing screen
the streamer card names the player (its mark and room), so the footer
SHALL NOT show, and its space goes to the page. The footer SHALL sit
centred at the bottom in portrait and in the bottom-right corner in
landscape.

#### Scenario: Bluesound player with its logo
- **WHEN** the device has a `logoUrl` and the name "Example Player"
- **THEN** the footer shows the logo and "Example Player" on the input and idle screens, and no footer shows on the playing screen

#### Scenario: No logo
- **WHEN** the device's brand is bluesound, without a `logoUrl`
- **THEN** the footer shows a streamer icon, "Bluesound" and "Example Player"

#### Scenario: Unknown maker
- **WHEN** the device has no brand
- **THEN** the footer shows only "Example Player"

### Requirement: Diagram fits
No part of the diagram SHALL leave its area, run past the page's bottom
margin, run into the queue's column, or run over a logo. Names, the codec
line and the room SHALL be shortened with "…" to the room they have; the
format on the streamer SHALL never be shortened. When the diagram runs top
to bottom and is too tall for the space above the bottom margin, it SHALL
give way in this order: speakers fold into a "+ N more" row, from the
last; the source's line gets shorter; the outputs move
onto one line as icons; last, the outputs are left out. Without a queue the
diagram SHALL run left to right only when every part fits in full, the
outputs' names included; otherwise it SHALL run top to bottom.

#### Scenario: Many speakers in portrait
- **WHEN** the session plays to five other speakers in portrait
- **THEN** as many rows as fit show, followed by "+ N more" for the rest, and nothing runs past the bottom margin

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
