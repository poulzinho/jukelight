## Purpose

The kiosk screen: what it shows in each playback state, readable from across
the room on landscape and portrait screens.

## ADDED Requirements

### Requirement: Playing screen
While a track is playing, the kiosk SHALL show the artwork, title, artists,
album, a progress bar with elapsed time and duration, a session badge with
the session label, and a quality badge with the quality label and detail.
Titles longer than two lines SHALL end with an ellipsis.

#### Scenario: Track with all data
- **WHEN** the snapshot is playing "Example Song" by "Example Artist" with artwork, position, session label "Qobuz" and quality "FLAC 16/44.1"
- **THEN** the screen shows the artwork, "Example Song", "Example Artist", the progress, a "Qobuz" badge and a "FLAC 16/44.1" badge

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
When the quality is Hi-Res, the kiosk SHALL show a "HI-RES" text badge next
to the quality badge.

#### Scenario: Hi-Res track
- **WHEN** the quality is "FLAC 24/96" and Hi-Res
- **THEN** a "HI-RES" badge is shown

#### Scenario: CD-quality track
- **WHEN** the quality is "FLAC 16/44.1" and not Hi-Res
- **THEN** no "HI-RES" badge is shown

### Requirement: Paused screen
While paused, the kiosk SHALL dim the content and show a large pause symbol
over the artwork at full brightness, with the progress frozen.

#### Scenario: Pause
- **WHEN** the snapshot changes from playing to paused
- **THEN** the content is dimmed, the pause symbol is shown and the elapsed time stops advancing

### Requirement: Idle screen
When nothing is playing (stopped, or no track), the kiosk SHALL show the
current time and "<player name> · nothing playing".

#### Scenario: Stopped
- **WHEN** the snapshot is stopped and the player name is "Example Player"
- **THEN** the screen shows the time and "Example Player · nothing playing"

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
