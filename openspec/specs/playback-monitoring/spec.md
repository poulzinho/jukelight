# playback-monitoring Specification

## Purpose

Describes what a player is doing in Jukelight's own terms: one snapshot per
player, the events derived from changes between snapshots, the per-track
position, and the audio quality including the Hi-Res rule.

## Requirements

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

### Requirement: Queue view
The snapshot SHALL state the queue as either available (an ordered list of
upcoming entries, each with title, artists and an optional artwork
reference) or not applicable, so that "this session has no queue" is a
state rather than a missing field.

#### Scenario: Session with upcoming tracks
- **WHEN** the session reports three upcoming tracks
- **THEN** the queue view is available with those three entries in order

#### Scenario: Session without a queue
- **WHEN** the session keeps its queue outside the player (a cast session)
- **THEN** the queue view is not applicable

### Requirement: Per-track position
The position SHALL consist of the elapsed time within the current track and
the moment that elapsed time was observed, so that a client can predict the
current elapsed time without new updates. A position SHALL only be reported
when it is measured from the start of the current track.

#### Scenario: Predicting progress while playing
- **WHEN** the snapshot reports elapsed 60 s observed at time T and the state is playing
- **THEN** the predicted elapsed time at T + 5 s is 65 s

#### Scenario: Position does not advance while paused
- **WHEN** the snapshot reports elapsed 60 s observed at time T and the state is paused
- **THEN** the predicted elapsed time at T + 5 s is 60 s

#### Scenario: Elapsed time not measured per track
- **WHEN** the source only knows the time since its stream started, not since the current track started
- **THEN** the snapshot has no position

### Requirement: Change detection
The system SHALL compare each new snapshot with the previous one and derive
events: TrackChanged, PlaybackStateChanged, QueueChanged, PositionResynced,
PlayerWentOffline and PlayerCameOnline. A change of the elapsed time that
matches the predicted progress SHALL NOT produce an event. A deviation of
more than 2 seconds from the predicted elapsed time SHALL produce
PositionResynced.

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

### Requirement: Audio quality
The audio quality SHALL be described either as a format (codec, bit depth,
sample rate, and whether the codec is lossless) or, when no format is
known, as a bitrate. Its label SHALL be "<CODEC> <bit depth>/<sample rate in
kHz>" for a format and "<kbps> kbps" for a bitrate.

#### Scenario: Lossless format
- **WHEN** the source reports FLAC, 24 bit, 96 000 Hz
- **THEN** the quality label is "FLAC 24/96" and the quality is lossless

#### Scenario: Bitrate only
- **WHEN** the source reports only a bitrate of 160 000 bit/s
- **THEN** the quality label is "160 kbps" and the quality is lossy

### Requirement: Hi-Res rule
A track SHALL count as Hi-Res when its source format is lossless, has at
least 24 bits and at least 44.1 kHz. A bitrate-only quality SHALL never
count as Hi-Res.

#### Scenario: CD quality
- **WHEN** the source format is FLAC 16/44.1
- **THEN** the track is not Hi-Res

#### Scenario: 24-bit at 44.1 kHz
- **WHEN** the source format is FLAC 24/44.1
- **THEN** the track is Hi-Res

#### Scenario: High sample rate
- **WHEN** the source format is FLAC 24/192
- **THEN** the track is Hi-Res

#### Scenario: Lossy codec with 24 bits
- **WHEN** the source format is AAC 24/48
- **THEN** the track is not Hi-Res

#### Scenario: Bitrate only
- **WHEN** the quality is a bitrate of 320 kbps
- **THEN** the track is not Hi-Res
