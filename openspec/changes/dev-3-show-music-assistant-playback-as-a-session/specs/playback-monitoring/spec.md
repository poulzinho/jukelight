## MODIFIED Requirements

### Requirement: Player snapshot
The system SHALL describe a player's current activity as a single snapshot
containing: the player name, its connectivity (online or offline), its
playback state (playing, paused, stopped or buffering), the session label,
the track (title, artists, album), an optional artwork reference, the
position, the audio quality, the queue view, and the album's colours (a
background and an accent colour) when the source knows them. Every field
that the current source does not provide SHALL be explicitly absent rather
than filled with a placeholder value.

#### Scenario: Snapshot of a playing track
- **WHEN** the player plays the track "Example Song" by "Example Artist" from the album "Example Album"
- **THEN** the snapshot has playback state playing, title "Example Song", artists ["Example Artist"] and album "Example Album"

#### Scenario: Source without duration
- **WHEN** the source reports a playing track but no duration
- **THEN** the snapshot's position has no duration and the track is still shown

#### Scenario: Source with album colours
- **WHEN** the source reports the background colour (73, 51, 38) and the accent colour (209, 147, 108)
- **THEN** the snapshot carries both colours

#### Scenario: Source without album colours
- **WHEN** the source reports no colours
- **THEN** the snapshot has no colours
