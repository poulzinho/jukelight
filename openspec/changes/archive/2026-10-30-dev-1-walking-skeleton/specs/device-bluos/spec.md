## Purpose

Follows a BluOS player through the BluOS Custom Integration API and
translates what it reports into a device snapshot: identity, playback state,
origin of the audio, and the content the player itself knows.

## ADDED Requirements

### Requirement: Player identity
The device monitor SHALL read the player's name and MAC address from
`/SyncStatus` when it connects. The player key SHALL be the MAC address in
lowercase without separators.

#### Scenario: Identity from SyncStatus
- **WHEN** `/SyncStatus` reports `name="Example Player"` and `mac="AA:BB:CC:DD:EE:FF"`
- **THEN** the device name is "Example Player" and the player key is "aabbccddeeff"

### Requirement: Long-polling
The device monitor SHALL follow `/Status` by long-polling with the `etag` of
the previous response and a timeout of at most 100 seconds. A poll that
returns at its timeout without changes SHALL be treated as normal, not as an
error. The monitor SHALL wait at least 1 second between two requests.

#### Scenario: Timeout without changes
- **WHEN** a long-poll returns after its timeout with the same `etag`
- **THEN** the monitor reports the player as online and starts the next long-poll

#### Scenario: Request spacing
- **WHEN** a long-poll returns immediately
- **THEN** the next request starts no earlier than 1 second after the previous one started

### Requirement: Connectivity and reconnection
When a request fails, the device monitor SHALL report the player as offline
and retry with an increasing delay of at most 30 seconds. After a successful
request it SHALL report the player as online again.

#### Scenario: Player unreachable
- **WHEN** a request to the player fails
- **THEN** the device snapshot is offline and a new attempt follows after a delay

#### Scenario: Player back
- **WHEN** a request succeeds after the player was offline
- **THEN** the device snapshot is online

### Requirement: Playback state mapping
The device monitor SHALL map the BluOS `state` to the playback state:
`play` and `stream` to playing, `pause` to paused, `connecting` to
buffering, and every other value to stopped.

#### Scenario: Cast session reports stream
- **WHEN** `/Status` reports `<state>stream</state>` with `<service>QobuzConnect</service>`
- **THEN** the playback state is playing

#### Scenario: Paused
- **WHEN** `/Status` reports `<state>pause</state>`
- **THEN** the playback state is paused

### Requirement: Origin
The device monitor SHALL report where the audio comes from as an origin:
a stream origin with its URL when `streamUrl` is an `http` or `https` URL;
otherwise a service origin with the `service` id and `serviceName`, marked as
a cast session when `serviceType` is `Endpoint`; otherwise no origin.

#### Scenario: Music Assistant stream
- **WHEN** `/Status` reports `<streamUrl>http://192.168.1.50:8097/flow/abc/aabbccddeeff/def/aabbccddeeff.flac</streamUrl>`
- **THEN** the origin is a stream origin with that URL

#### Scenario: Cast service
- **WHEN** `/Status` reports `<service>Spotify</service>`, `<serviceName>Spotify</serviceName>` and `<serviceType>Endpoint</serviceType>`
- **THEN** the origin is a service origin "Spotify" marked as a cast session

#### Scenario: Native service
- **WHEN** `/Status` reports `<service>Qobuz</service>` and `<serviceType>CloudService</serviceType>`
- **THEN** the origin is a service origin "Qobuz" not marked as a cast session

### Requirement: The player's own content
The device monitor SHALL report the content the player knows: title from
`title1`, artist from `artist` or else `title2`, album from `album` or else
`title3`, artwork from `image` or else `currentImage` (relative paths
resolved against the player's address), duration from `totlen`, and quality
from `streamFormat` or, when absent, from a numeric `quality` as a bitrate.
Text SHALL be decoded from XML entities.

#### Scenario: Full metadata
- **WHEN** `/Status` reports `title1` "Example Song", `artist` "Example Artist", `album` "Example Album", `totlen` 214 and `streamFormat` "FLAC 16/44.1"
- **THEN** the content has that title, artist and album, duration 214 s and quality "FLAC 16/44.1"

#### Scenario: Escaped characters
- **WHEN** `/Status` reports `<title1>Me &amp; You</title1>`
- **THEN** the title is "Me & You"

#### Scenario: Relative artwork path
- **WHEN** `/Status` reports `<image>/Artwork?service=Qobuz&amp;songid=Qobuz%3A1</image>` for the player at 192.168.1.40
- **THEN** the artwork source is `http://192.168.1.40:11000/Artwork?service=Qobuz&songid=Qobuz%3A1`

#### Scenario: Bitrate instead of format
- **WHEN** `/Status` has no `streamFormat` and reports `<quality>160000</quality>`
- **THEN** the quality is a bitrate of 160 kbps

### Requirement: Position from the player
The device monitor SHALL report `secs` as the per-track elapsed time only for
service origins. For a stream origin it SHALL report no position, because
the player then counts from the start of the stream rather than the track.

#### Scenario: Native session
- **WHEN** the origin is a service origin and `/Status` reports `<secs>42</secs>`
- **THEN** the position has elapsed 42 s

#### Scenario: Stream origin
- **WHEN** the origin is a stream origin and `/Status` reports `<secs>437</secs>`
- **THEN** the content has no position

### Requirement: Quality only for service sessions
The device monitor SHALL report quality only for service origins. For a
stream origin it SHALL report no quality, because `streamFormat` then
describes the stream sent to the player (which another system may have
converted, for example from 16 to 24 bits), not the source.

#### Scenario: Converted stream
- **WHEN** the origin is a stream origin and `/Status` reports `<streamFormat>FLAC 24/44.1</streamFormat>`
- **THEN** the content has no quality, so no Hi-Res label is shown

#### Scenario: Native session
- **WHEN** the origin is the native service "Qobuz" and `/Status` reports `<streamFormat>FLAC 24/96</streamFormat>`
- **THEN** the quality is "FLAC 24/96"

### Requirement: Native queue
For a service origin that is not a cast session, the device monitor SHALL
read the queue from `/Playlist`, reload it whenever the playlist id (`pid`)
changes, and report the entries after the current index (`song`) as the
upcoming entries. For cast sessions, stream origins and no origin, it SHALL
report the queue as not applicable, even when `/Playlist` contains entries.

#### Scenario: Native queue
- **WHEN** the origin is the native service "Qobuz", `song` is 0 and `/Playlist` holds 12 entries
- **THEN** the queue view is available with entries 2 to 12 in order

#### Scenario: Stale playlist during a cast session
- **WHEN** the origin is the cast service "Spotify" and `/Playlist` still holds 12 entries from an earlier session
- **THEN** the queue view is not applicable

### Requirement: Recorded responses as tests
Every requirement of this capability SHALL be covered by tests against
recorded `/Status`, `/SyncStatus` and `/Playlist` responses for at least:
Qobuz Connect, native Qobuz, Spotify Connect, a Music Assistant stream,
paused and stopped. Recorded responses SHALL contain example values only.

#### Scenario: No personal data in fixtures
- **WHEN** the test fixtures are checked
- **THEN** they contain no IP address outside 192.168.1.0/24 and no MAC address other than aa:bb:cc:dd:ee:ff
