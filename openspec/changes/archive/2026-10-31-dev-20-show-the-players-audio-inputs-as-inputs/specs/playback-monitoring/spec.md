## MODIFIED Requirements

### Requirement: Player snapshot
The system SHALL describe a player's current activity as a single snapshot
containing: the player name, its connectivity (online or offline), its
playback state (playing, paused, stopped or buffering), the session label,
the track (title, artists, album), an optional artwork reference, the
position, the audio quality, the queue view, the album's colours (a
background and an accent colour) when the source knows them, and the audio
input (its kind, name and, once its signal started, since when it plays)
when an input plays. Every
field that the current source does not provide SHALL be explicitly absent
rather than filled with a placeholder value.

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

#### Scenario: Input plays
- **WHEN** the player plays its optical input named "Example Input", whose signal started 23 minutes ago
- **THEN** the snapshot carries the input (optical, "Example Input", started 23 minutes ago) and has no track

### Requirement: Change detection
The system SHALL compare each new snapshot with the previous one and derive
events: TrackChanged, PlaybackStateChanged, QueueChanged, PositionResynced,
InputChanged, PlayerWentOffline and PlayerCameOnline. A change of the
elapsed time that matches the predicted progress SHALL NOT produce an event.
A deviation of more than 2 seconds from the predicted elapsed time SHALL
produce PositionResynced. A change of the input's kind or name, an input
appearing or going away, its signal starting, or its start time moving by
more than 2 seconds SHALL produce InputChanged; smaller moves of the start
time SHALL NOT produce an event.

#### Scenario: New track
- **WHEN** the previous snapshot has title "Song A" and the new snapshot has title "Song B"
- **THEN** a TrackChanged event is produced

#### Scenario: Normal progress produces no event
- **WHEN** the previous snapshot reports elapsed 60 s at time T, the state is playing, and the new snapshot reports elapsed 90 s at T + 30 s with all other fields equal
- **THEN** no event is produced

#### Scenario: Seek
- **WHEN** the previous snapshot reports elapsed 60 s at time T, the state is playing, and the new snapshot reports elapsed 150 s at T + 10 s
- **THEN** a PositionResynced event is produced

#### Scenario: Pause
- **WHEN** the previous snapshot is playing and the new snapshot is paused
- **THEN** a PlaybackStateChanged event is produced

#### Scenario: Player goes offline
- **WHEN** the previous snapshot is online and the new snapshot is offline
- **THEN** a PlayerWentOffline event is produced

#### Scenario: Signal starts again
- **WHEN** the previous snapshot has the input's signal started at time T and the new snapshot has it started at T + 30 minutes
- **THEN** an InputChanged event is produced

#### Scenario: Signal arrives
- **WHEN** the previous snapshot has the input waiting for a signal and the new snapshot has its signal started
- **THEN** an InputChanged event is produced

#### Scenario: Start time jitter
- **WHEN** the previous snapshot has the input's signal started at time T and the new snapshot has it started at T + 1 s, all else equal
- **THEN** no event is produced
