## MODIFIED Requirements

### Requirement: Origin
The device monitor SHALL report where the audio comes from as an origin:
a stream origin with its URL when `streamUrl` is an `http` or `https` URL;
an input origin when `serviceType` is `AudioInputs` and the input is not
Bluetooth; otherwise a service origin with the `service` id and
`serviceName`, marked as a cast session when `serviceType` is `Endpoint`;
otherwise no origin.

#### Scenario: Music Assistant stream
- **WHEN** `/Status` reports `<streamUrl>http://192.168.1.50:8097/flow/abc/aabbccddeeff/def/aabbccddeeff.flac</streamUrl>`
- **THEN** the origin is a stream origin with that URL

#### Scenario: Cast service
- **WHEN** `/Status` reports `<service>Spotify</service>`, `<serviceName>Spotify</serviceName>` and `<serviceType>Endpoint</serviceType>`
- **THEN** the origin is a service origin "Spotify" marked as a cast session

#### Scenario: Native service
- **WHEN** `/Status` reports `<service>Qobuz</service>` and `<serviceType>CloudService</serviceType>`
- **THEN** the origin is a service origin "Qobuz" not marked as a cast session

#### Scenario: Optical input
- **WHEN** `/Status` reports `<service>Capture</service>`, `<serviceType>AudioInputs</serviceType>` and `<inputTypeIndex>spdif-1</inputTypeIndex>`
- **THEN** the origin is an input origin

#### Scenario: Bluetooth input
- **WHEN** `/Status` reports `<serviceType>AudioInputs</serviceType>` and `<inputTypeIndex>bluetooth-1</inputTypeIndex>`
- **THEN** the origin is a service origin, as before

### Requirement: Native queue
For a service origin that is not a cast session, the device monitor SHALL
read the queue from `/Playlist`, reload it whenever the playlist id (`pid`)
changes, and report the entries after the current index (`song`) as the
upcoming entries. For cast sessions, input origins, stream origins and no
origin, it SHALL report the queue as not applicable, even when `/Playlist`
contains entries, and SHALL NOT read `/Playlist` for input origins.

#### Scenario: Native queue
- **WHEN** the origin is the native service "Qobuz", `song` is 0 and `/Playlist` holds 12 entries
- **THEN** the queue view is available with entries 2 to 12 in order

#### Scenario: Stale playlist during a cast session
- **WHEN** the origin is the cast service "Spotify" and `/Playlist` still holds 12 entries from an earlier session
- **THEN** the queue view is not applicable

#### Scenario: Stale playlist while an input plays
- **WHEN** the origin is the optical input and `/Playlist` still holds 12 entries from an earlier session
- **THEN** the queue view is not applicable and `/Playlist` is not requested

## ADDED Requirements

### Requirement: Audio inputs
For an input origin the device monitor SHALL report the input's kind from
`inputTypeIndex` (`spdif` optical, `analog` analog, `arc` or `hdmi` HDMI
ARC, anything else other), its name from `title1` (as set in the player's
app; "Input" when empty), and when it was selected (the observation time
minus `secs`). It SHALL report no track content: no title, artists, album,
artwork, position, quality or queue.

#### Scenario: Named optical input
- **WHEN** `/Status` reports an input origin with `<inputTypeIndex>spdif-1</inputTypeIndex>`, `<title1>Leak CDT</title1>` and `<secs>314</secs>`, observed at time T
- **THEN** the input is optical, named "Leak CDT", selected at T minus 314 s, and the device reports no content

#### Scenario: Input selected again
- **WHEN** `/Status` reports the input with `<state>connecting</state>` and `<secs>0</secs>` at time T
- **THEN** the input was selected at T and the playback state is buffering

#### Scenario: Unknown input kind
- **WHEN** `/Status` reports an input origin with `<inputTypeIndex>usb-1</inputTypeIndex>`
- **THEN** the input's kind is other
