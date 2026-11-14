# session-music-assistant Specification

## Purpose
Recognises when Music Assistant drives the player and translates Music
Assistant's view of the playback (track, position, quality, queue, colours)
into session content, through Music Assistant's HTTP API.

## Requirements

### Requirement: Claiming the session
The Music Assistant provider SHALL claim the device's origin when the origin
is a stream whose host is the host of the configured Music Assistant URL and
whose path contains `/<player key>/`, the player key being the device's MAC
address in lowercase without separators (Music Assistant's `player_id` for
Bluesound players). That claim SHALL be decided without contacting Music
Assistant. The provider SHALL also claim an AirPlay origin when Music
Assistant's latest answer for this player (`players/get`, at most 5 seconds
old) has an `active_source` that is one of Music Assistant's queues, which
is how Music Assistant plays to a group of speakers.

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
- **WHEN** the device reports an AirPlay origin and Music Assistant reports this player's `active_source` as the queue `aabbccddeeff`
- **THEN** the provider claims the origin

#### Scenario: A member of another player's group
- **WHEN** the device reports an AirPlay origin and Music Assistant reports this player's `active_source` as the queue of the group leader `112233445566`
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

### Requirement: Track and artwork
The provider SHALL take the title, artists, album and duration from the
queue's current item (its media item), falling back to the player's current
media. Artwork SHALL be the image's own URL when Music Assistant marks it
remotely accessible and it is an `http(s)` URL, and otherwise Music
Assistant's image proxy at its original size
(`<MA URL>/imageproxy/<proxy id>?size=0&fmt=jpg`).

#### Scenario: Library track
- **WHEN** the current item is "Example Song" by "Example Artist" from "Example Album", 246 s long, with a library image that has the proxy id `abc123`
- **THEN** the content has that title, artist, album and duration, and the artwork `http://192.168.1.50:8095/imageproxy/abc123?size=0&fmt=jpg`

#### Scenario: Remote image
- **WHEN** the current item's image is remotely accessible at `https://static.qobuz.com/images/covers/00/00/0000000000000_600.jpg`
- **THEN** the content's artwork is that URL

### Requirement: Per-track position
The provider SHALL report the position from the queue's `elapsed_time` and
`elapsed_time_last_updated`. It SHALL NOT use the player's `elapsed_time`,
which counts across the whole stream. When `elapsed_time_last_updated` is
more than 10 seconds away from the local clock, the position SHALL be taken
as observed now.

#### Scenario: Queue position
- **WHEN** the queue reports `elapsed_time` 171.8 s, updated 1 s ago, while the player reports `elapsed_time` 437 s
- **THEN** the content's position has elapsed 171.8 s

#### Scenario: Clock skew
- **WHEN** the queue's `elapsed_time_last_updated` is 1 hour in the future
- **THEN** the position's observation time is the local time of the update

### Requirement: Source quality
The provider SHALL report the quality of the source, from the current item's
`streamdetails.audio_format`. The quality detail SHALL be built from:
"Lossless" for a lossless source that is not Hi-Res; "→ <bits>/<kHz> to
player" when the format sent to the player differs; and "Normalised" when
the loudness normalisation mode changes the audio (`dynamic`, `fixed_gain`,
`fallback_fixed_gain`, `fallback_dynamic`, `source`), joined with " · ".

#### Scenario: Padded CD-quality source
- **WHEN** the source is FLAC 16/44.1, the output is FLAC 24/44.1 and normalisation is `measurement_only`
- **THEN** the quality label is "FLAC 16/44.1" with the detail "Lossless · → 24/44.1 to player"

#### Scenario: Normalised Hi-Res source
- **WHEN** the source is FLAC 24/88.2, the output is FLAC 24/88.2 and normalisation is `dynamic`
- **THEN** the quality label is "FLAC 24/88.2", it is Hi-Res, and the detail is "Normalised"

### Requirement: Up next
The provider SHALL report the queue as available with the next entries after
the current index (at most 5), each with title, artists and a thumbnail from
the image proxy (`size=256`). It SHALL fetch the entries again only when the
current item, the current index or the number of items changes.

#### Scenario: Queue with following tracks
- **WHEN** the queue has 11 items and the current index is 2
- **THEN** the content's queue lists items 4 to 8

#### Scenario: Unchanged queue
- **WHEN** two consecutive polls report the same current item, index and item count
- **THEN** the queue entries are fetched only once

### Requirement: Music provider name
The provider SHALL name the music provider of the current item
(`streamdetails.provider`, an instance id) by the display name Music
Assistant reports for it (`providers`). Music Assistant's default name for
OpenSubsonic SHALL be shortened to "Subsonic"; the built-in provider has no
name.

#### Scenario: Qobuz through Music Assistant
- **WHEN** the current item's provider is `qobuz--instance1`, which Music Assistant names "Qobuz"
- **THEN** the content names the music service "Qobuz"

#### Scenario: OpenSubsonic
- **WHEN** the provider's name is "OpenSubsonic Media Server Library"
- **THEN** the content names the music service "Subsonic"

### Requirement: Album colours
The provider SHALL report the album's colours from the player's current
media palette: `background_dark` as the background colour and `on_dark` as
the accent colour, the pair Music Assistant keeps at a contrast ratio of at
least 4.5:1 with each other and `background_dark` with white text. When
either is missing, the content SHALL have no colours.

#### Scenario: Palette present
- **WHEN** the palette has `background_dark` [73, 51, 38], `on_dark` [209, 147, 108] and `primary` [73, 51, 38]
- **THEN** the content's colours have background (73, 51, 38) and accent (209, 147, 108)

#### Scenario: Incomplete palette
- **WHEN** the palette has `background_dark` but no `on_dark`
- **THEN** the content has no colours

#### Scenario: No palette
- **WHEN** the current media has no palette
- **THEN** the content has no colours

### Requirement: Unreachable Music Assistant
When a request fails, is rejected (for example 401 for a wrong token) or
returns unreadable data, the provider SHALL report no content, so the
session shows as unavailable, and SHALL keep retrying while it claims the
origin. It SHALL report content again after the next successful poll.

#### Scenario: Server down
- **WHEN** the provider claims the origin and Music Assistant does not answer
- **THEN** the provider reports no content

#### Scenario: Wrong token
- **WHEN** Music Assistant answers 401
- **THEN** the provider reports no content

#### Scenario: Back
- **WHEN** Music Assistant answers again after being down
- **THEN** the provider reports content after its next poll

### Requirement: Token handling
The provider SHALL send the token only as a bearer `Authorization` header.
The token SHALL NOT appear in logs, error messages or any API response.

#### Scenario: Failed request is logged
- **WHEN** a request to Music Assistant fails
- **THEN** the log message does not contain the token

### Requirement: Recorded responses as tests
The provider's behaviour SHALL be tested against recorded Music Assistant
responses (`players/get`, `player_queues/get`, `player_queues/items`,
`providers`) for at least a library track (Navidrome) and a Qobuz track.
Recorded responses SHALL contain example values only.

#### Scenario: No personal data in fixtures
- **WHEN** the test fixtures are checked
- **THEN** they contain no IP address outside 192.168.1.0/24, no MAC address other than aa:bb:cc:dd:ee:ff and no token

### Requirement: Music provider brand
The provider SHALL recognise the brand of the current item's music
provider from the `domain` Music Assistant reports for the provider
instance, not from its renameable name. A provider in the `opensubsonic`
domain SHALL be Navidrome when its name contains "Navidrome" (in any
case), and Subsonic otherwise. Providers of local files (domains
`filesystem_*`) SHALL be the brand Library. A domain without a known brand
SHALL have no brand; its name still names the service. As before, Music
Assistant's built-in provider names no service.

#### Scenario: Renamed Qobuz provider
- **WHEN** the current item's provider is in the domain `qobuz` and Music Assistant names it "My Qobuz"
- **THEN** the content's music service has the brand qobuz and the name "My Qobuz"

#### Scenario: Navidrome
- **WHEN** the current item's provider is in the domain `opensubsonic` and named "Navidrome"
- **THEN** the content's music service has the brand navidrome

#### Scenario: Another Subsonic server
- **WHEN** the current item's provider is in the domain `opensubsonic` and named "OpenSubsonic Media Server Library"
- **THEN** the content's music service has the brand subsonic and the name "Subsonic"

#### Scenario: Local files
- **WHEN** the current item's provider is in a `filesystem_local` domain
- **THEN** the content's music service has the brand library

#### Scenario: Unknown domain
- **WHEN** the current item's provider is in a domain without a known brand and named "Example Service"
- **THEN** the content's music service has no brand and the name "Example Service"

### Requirement: Delivered quality
Next to the source quality, the provider SHALL report the quality Music
Assistant sends to this player, from the `output_format` of the
`streamdetails.audio_processing.outputs` entry that names the player (or
the only entry), and the changes Music Assistant made on the way:
- **resampled**, when the output's sample rate differs from the source's;
- **bit depth changed**, when only the bit depth differs;
- **normalised**, when the normalisation mode changes the audio (see
  "Source quality"), with the gain from
  `audio_processing.queue_processing.normalization.applied_gain_db`, or
  else `volume_normalization_gain_correct`, when one is reported;
- **DSP**, when the output's `dsp.state` is `enabled`.

Without an output format, the delivered quality SHALL be unknown and only
normalisation and DSP SHALL be reported.

#### Scenario: Padded CD-quality source
- **WHEN** the source is FLAC 16/44.1, the output is FLAC 24/44.1, normalisation is `measurement_only` and DSP is `disabled`
- **THEN** the delivered quality is FLAC 24/44.1 and the only change is "bit depth changed"

#### Scenario: Upsampled and normalised
- **WHEN** the source is FLAC 16/44.1, the output is FLAC 24/96, normalisation is `dynamic` with an applied gain of −6.0 dB, and DSP is `enabled`
- **THEN** the delivered quality is FLAC 24/96 and the changes are "resampled", "normalised (−6.0 dB)" and "DSP"

#### Scenario: Untouched Hi-Res
- **WHEN** the source and the output are FLAC 24/96, normalisation is `disabled` and DSP is `disabled`
- **THEN** the delivered quality is FLAC 24/96 with no changes

#### Scenario: No output for the player
- **WHEN** `audio_processing` has no outputs
- **THEN** the delivered quality is unknown

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
