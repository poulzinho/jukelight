## MODIFIED Requirements

### Requirement: Claiming the session
The Music Assistant provider SHALL claim the device's origin when the origin
is a stream whose host is the host of the configured Music Assistant URL and
whose path contains `/<player key>/`, the player key being the device's MAC
address in lowercase without separators (Music Assistant's `player_id` for
Bluesound players). That claim SHALL be decided without contacting Music
Assistant. The provider SHALL also claim an AirPlay origin when Music
Assistant's latest answer for this player (`players/get`, at most 5 seconds
old) has an `active_source` that is one of Music Assistant's queues, which
is how Music Assistant plays to a group of speakers, and either Music
Assistant started naming its queue after the player's AirPlay title last
changed to another sender's, or the player's AirPlay title shows that
Music Assistant sends it: the title is
`cliairplay` (Music Assistant's AirPlay sender) or the title of Music
Assistant's current track, the track before it or the next one (around a
track change, the player and Music Assistant's answer may differ by a
track for a moment). Music Assistant can take longer than 5 seconds
to notice that another sender, such as a phone, took the player over; the
title tells them apart at once.

#### Scenario: Stream from Music Assistant for this player
- **WHEN** Music Assistant is configured at `http://192.168.1.50:8095` and the device with player key `aabbccddeeff` reports the stream origin `http://192.168.1.50:8097/flow/session1/aabbccddeeff/00000000000000000000000000000001/aabbccddeeff.flac`
- **THEN** the provider claims the origin

#### Scenario: Stream for another player
- **WHEN** the stream origin is on the Music Assistant host but its path contains the player key `112233445566`, not `aabbccddeeff`
- **THEN** the provider does not claim the origin

#### Scenario: Stream from another host
- **WHEN** the stream origin is `http://192.168.1.60:8097/flow/session1/aabbccddeeff/x.flac`
- **THEN** the provider does not claim the origin

#### Scenario: Service session
- **WHEN** the device reports the service origin "Qobuz"
- **THEN** the provider does not claim the origin

#### Scenario: Music Assistant plays to a group
- **WHEN** the device reports an AirPlay origin titled `cliairplay` and Music Assistant reports this player's `active_source` as the queue `aabbccddeeff`
- **THEN** the provider claims the origin

#### Scenario: Music Assistant sends its track's title
- **WHEN** the device reports an AirPlay origin titled with Music Assistant's current track, and Music Assistant reports the queue `aabbccddeeff`
- **THEN** the provider claims the origin

#### Scenario: Around a track change
- **WHEN** Music Assistant reports the queue `aabbccddeeff` and the device's AirPlay origin is titled with the next track in that queue, or with the track that played before
- **THEN** the provider claims the origin

#### Scenario: A group takes over from a phone
- **WHEN** the device's AirPlay origin has the title "Phone Song" and Music Assistant then starts reporting the queue `aabbccddeeff`, while the title stays "Phone Song"
- **THEN** the provider claims the origin

#### Scenario: A phone takes over from a group
- **WHEN** Music Assistant still reports the queue `aabbccddeeff`, but the device's AirPlay origin has no title, or the title "Phone Song"
- **THEN** the provider does not claim the origin

#### Scenario: A member of another player's group
- **WHEN** the device reports an AirPlay origin titled `cliairplay` and Music Assistant reports this player's `active_source` as the queue of the group leader `112233445566`
- **THEN** the provider claims the origin

#### Scenario: AirPlay from a phone
- **WHEN** the device reports an AirPlay origin and Music Assistant reports this player's `active_source` as `AirPlay`
- **THEN** the provider does not claim the origin

#### Scenario: No answer yet
- **WHEN** the device reports an AirPlay origin and Music Assistant has not answered within the last 5 seconds
- **THEN** the provider does not claim the origin

### Requirement: Following the player
While it claims the device's origin, the provider SHALL fetch the player's
state from Music Assistant (`players/get`, and `player_queues/get` for the
queue the player plays from) at least every 2 seconds and publish an update
whenever the resulting content changes; it SHALL do the same while it holds
a paused group session. While the device reports an AirPlay origin that it
does not claim, it SHALL fetch only `players/get`, at least every 2
seconds, to learn whether Music Assistant drives the player. Otherwise it
SHALL make no requests.

#### Scenario: Claimed
- **WHEN** the provider claims the origin for 5 seconds
- **THEN** it has requested the player and queue state at least twice

#### Scenario: Not claimed
- **WHEN** the device reports a service origin for 5 seconds
- **THEN** the provider has made no request to Music Assistant

#### Scenario: Track change
- **WHEN** Music Assistant's current queue item changes from "Song A" to "Song B"
- **THEN** the provider publishes an update whose content has the title "Song B"

#### Scenario: AirPlay session
- **WHEN** the device reports an AirPlay origin that the provider does not claim, for 5 seconds
- **THEN** the provider has requested only the player state, at least twice

## ADDED Requirements

### Requirement: The group's queue
When it claims a group session, the provider SHALL take the track, artwork,
position, quality, queue and colours from the queue named by the player's
`active_source`: the player's own queue when it leads the group, the
leader's queue when it is a member.

#### Scenario: The player leads the group
- **WHEN** this player's `active_source` is its own queue `aabbccddeeff`, which plays "Example Song"
- **THEN** the content's title is "Example Song"

#### Scenario: The player is a member
- **WHEN** this player's `active_source` is the queue `112233445566`, which plays "Other Song"
- **THEN** the provider reads the queue `112233445566` and the content's title is "Other Song"

### Requirement: The group's speakers
While it claims the session, the provider SHALL report the other speakers
playing the same queue: each player whose `synced_to` names the leader, or
which the leader lists in `group_members`, except this player. For each it
SHALL report its name (`display_name`, or else `name`), its kind (a
computer for Music Assistant's web player, otherwise a speaker), whether it
plays (`playback_state` is `playing`), and the format it receives: the
`output_format` of the `audio_processing.outputs` entry whose `player_ids`
contain it, as a quality. Without a group, the list SHALL be empty.

#### Scenario: Recorded group
- **WHEN** this player leads a group with two other players, one receiving PCM 16/44.1 and the other FLAC 24/44.1
- **THEN** the content lists two speakers with those formats, and this player is not among them

#### Scenario: A speaker dropped out
- **WHEN** a group member's `playback_state` is `idle` while the group plays
- **THEN** that speaker is listed as not playing

#### Scenario: No format for a speaker
- **WHEN** no output entry names a group member
- **THEN** that speaker is listed without a format

#### Scenario: Playing alone
- **WHEN** this player has no `group_members` and no `synced_to`
- **THEN** the list of speakers is empty

### Requirement: A paused group
After the provider claimed a group session, when the device reports
stopped and Music Assistant's queue for the player is not playing but
still holds the same current item, the provider SHALL hold the session:
it SHALL keep claiming the device and report that item with its position,
"Up next" and the speakers, marked as held. The hold SHALL end when the
device reports anything other than stopped, or when the queue's current
item changes or clears.

#### Scenario: Group paused
- **WHEN** a group session was claimed with "Example Song", then the device reports stopped with its own previous queue and Music Assistant's queue is idle with "Example Song" at 121 s
- **THEN** the provider claims the device and reports "Example Song" at 121 s, held

#### Scenario: Group resumed
- **WHEN** the provider holds a paused group session and the device reports an AirPlay session that Music Assistant drives
- **THEN** the provider claims it as a group session, not held

#### Scenario: Something else plays
- **WHEN** the provider holds a paused group session and the device reports playing its own Qobuz queue
- **THEN** the provider no longer claims the device

#### Scenario: Queue cleared
- **WHEN** the provider holds a paused group session and Music Assistant's queue no longer has a current item
- **THEN** the provider no longer claims the device
