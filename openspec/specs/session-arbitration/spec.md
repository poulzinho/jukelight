# session-arbitration Specification

## Purpose

Decides which content a player's snapshot shows: the content of a session
provider that claims the device's origin, or otherwise the content the
device itself reports.

## Requirements

### Requirement: Claimed origin uses the provider's content
When a session provider claims the device's origin and has content, the
snapshot SHALL use that provider's content, and the session label SHALL be
the provider's label, followed by " · " and the music service when the
content names one. The playback state SHALL come from the device, except
that held content (a paused group the provider holds) SHALL be paused.
Connectivity SHALL always come from the device.

#### Scenario: Provider drives the device
- **WHEN** a provider labelled "Example Provider" claims the device's origin and reports the title "Provider Song", while the device reports the title "Example Provider"
- **THEN** the snapshot shows the title "Provider Song" with the session label "Example Provider"

#### Scenario: State comes from the device
- **WHEN** a provider claims the origin and reports playing, while the device reports paused
- **THEN** the snapshot's playback state is paused

#### Scenario: Held content is paused
- **WHEN** a provider claims the origin with held content, while the device reports stopped
- **THEN** the snapshot's playback state is paused, with the provider's content

#### Scenario: Provider names the music service
- **WHEN** a provider labelled "Example Provider" claims the origin and its content names the music service "Example Service"
- **THEN** the session label is "Example Provider · Example Service"

### Requirement: Unreachable provider
When a session provider claims the device's origin but cannot supply
content, the snapshot SHALL show the title "Playing from <label>", with no
artists, album, artwork, position or queue, and SHALL mark the session as
unavailable.

#### Scenario: Provider down
- **WHEN** a provider labelled "Example Provider" claims the origin but cannot be reached
- **THEN** the snapshot shows "Playing from Example Provider", marked unavailable, without position or queue

### Requirement: Unclaimed origin uses the device's content
When no session provider claims the origin, the snapshot SHALL use the
content the device reports. When the device plays a service origin but
reports no content (an AirPlay sender that sends no title, as YouTube
does), the snapshot SHALL show the title "Playing from <service name>",
with no position or queue; while the device is stopped it SHALL show no
content. The session label SHALL be the service name for a service origin,
"Stream" for a stream origin, the input's kind for an input origin
("Optical input", "Analogue input", "HDMI ARC", "Input"), and empty when
there is no origin. For an input origin the snapshot SHALL carry the input.

#### Scenario: Native service
- **WHEN** no provider is registered and the device reports the service origin "Qobuz" with the title "Example Song"
- **THEN** the snapshot shows "Example Song" with the session label "Qobuz"

#### Scenario: Service without a title
- **WHEN** no provider claims the origin and the device plays the service origin "AirPlay" without content
- **THEN** the snapshot shows the title "Playing from AirPlay" with the session label "AirPlay", without position or queue

#### Scenario: Stopped service without a title
- **WHEN** the device reports the service origin "AirPlay" without content and is stopped
- **THEN** the snapshot has no content

#### Scenario: Unclaimed stream
- **WHEN** no provider is registered and the device reports a stream origin with the title "Music Assistant"
- **THEN** the snapshot shows the title "Music Assistant" with the session label "Stream"

#### Scenario: Optical input
- **WHEN** the device reports an optical input named "Example Input"
- **THEN** the snapshot has the session label "Optical input", the input named "Example Input" and no track

### Requirement: Claims are decided offline
Whether a provider claims an origin SHALL be decided from the provider's
configuration and the origin alone, without contacting the provider, so
that the label stays correct while the provider is unreachable. The one
exception SHALL be an origin the device reports the same way for the
provider and for others (an AirPlay session): its claim MAY depend on the
provider's latest answer, and without an answer the origin SHALL stay the
device's.

#### Scenario: Claim while the provider is down
- **WHEN** a provider's network connection is down and the device reports an origin that matches the provider's configuration
- **THEN** the provider still claims the origin

#### Scenario: AirPlay while the provider is down
- **WHEN** the device reports an AirPlay origin and the provider has had no answer for 5 seconds
- **THEN** the origin is not claimed and the snapshot shows the device's AirPlay session

### Requirement: Provider updates refresh the snapshot
When the provider that claims the device's origin publishes new content, the
snapshot SHALL be rebuilt from the latest device report and that content,
even when the device reports nothing new. Providers SHALL be told which
device report is current, so they can start or stop following the player.

#### Scenario: Track change seen only by the provider
- **WHEN** a provider claims the origin, the device keeps reporting the same stream, and the provider's content changes from "Song A" to "Song B"
- **THEN** the snapshot's title changes to "Song B"

#### Scenario: Provider told about the device
- **WHEN** the device reports a new snapshot
- **THEN** every provider receives that snapshot before the player snapshot is rebuilt

### Requirement: Source of the session
The snapshot SHALL name its source: the music service's brand (if known)
and name, and the route it takes. When a provider claims the session, the
source SHALL be the provider's music service, with the provider as the
route; when the provider names no music service, the source SHALL be the
provider itself, without a route. When the provider claims the session but
is unreachable, the source SHALL be the provider, with its route marked
unavailable. When no provider claims it, the source SHALL be the device's
service (or its audio input) without a route. The snapshot SHALL also carry
the player's maker.

#### Scenario: Claimed by Music Assistant
- **WHEN** Music Assistant claims the session and names the music service "Qobuz" with the brand qobuz
- **THEN** the snapshot's source has the brand qobuz, the name "Qobuz" and the route "Music Assistant", available

#### Scenario: Music Assistant alone
- **WHEN** Music Assistant claims the session and names no music service
- **THEN** the snapshot's source is "Music Assistant", without a route

#### Scenario: Music Assistant unreachable
- **WHEN** Music Assistant claims the session but can't be reached
- **THEN** the snapshot's source is "Music Assistant" and its route "Music Assistant" is unavailable

#### Scenario: Unclaimed service
- **WHEN** no provider claims the session and the device plays from the service `Spotify`
- **THEN** the snapshot's source has the brand spotify and no route

### Requirement: Speakers and outputs on the snapshot
The snapshot SHALL carry the claimed content's other speakers and the
device's outputs. The device's outputs SHALL be the configured speaker on
the player's output (`PLAYER_OUTPUT_NAME`), when set, followed by the
subwoofer when the device reports one.

#### Scenario: Group with an output speaker and a subwoofer
- **WHEN** Music Assistant reports two other speakers, the output speaker is configured as "Example Speaker" and the device reports a subwoofer
- **THEN** the snapshot has two speakers and the outputs "Example Speaker" (a speaker) and a subwoofer

#### Scenario: Nothing configured
- **WHEN** no output speaker is configured and the device reports no subwoofer
- **THEN** the snapshot has no outputs

### Requirement: Your names for speakers
The snapshot SHALL show a speaker under the name given for it in
`SPEAKER_NAMES` (a JSON object from the name Music Assistant shows to your
name), and under its reported name otherwise. An invalid `SPEAKER_NAMES`
SHALL be logged at start-up and ignored.

#### Scenario: Renamed speaker
- **WHEN** `SPEAKER_NAMES` is `{"Example Portable Speaker": "Kitchen"}` and Music Assistant reports a speaker named "Example Portable Speaker"
- **THEN** the snapshot lists it as "Kitchen"

#### Scenario: Not renamed
- **WHEN** `SPEAKER_NAMES` has no entry for "Example Web Player"
- **THEN** the snapshot lists it as "Example Web Player"

#### Scenario: Invalid names
- **WHEN** `SPEAKER_NAMES` is not a JSON object of strings
- **THEN** start-up logs a warning and every speaker keeps its reported name

### Requirement: A new session settles before it shows
While a session provider is still settling the device's origin (it hasn't
answered yet whether it drives the device, or it claims the device but
hasn't fetched its first content), the published player snapshot (what
the kiosk and other live clients receive) SHALL stay as it was, for at
most 2 seconds. As soon as no provider is settling, or after those 2
seconds, the published snapshot SHALL catch up at once. The current
snapshot and the player's events SHALL follow the device as usual.

#### Scenario: A group resumes
- **WHEN** the snapshot shows the idle screen, the device reports an AirPlay origin, and the provider answers after half a second that it drives the device, with its content
- **THEN** the published snapshot changes once, from the idle screen straight to the provider's content, with no AirPlay snapshot in between

#### Scenario: The provider doesn't answer in time
- **WHEN** the device reports an AirPlay origin and the provider is still settling after 2 seconds
- **THEN** the published snapshot shows the device's AirPlay session

#### Scenario: Nothing settling
- **WHEN** the device reports a new origin and no provider is settling
- **THEN** the published snapshot changes at once

#### Scenario: The current snapshot and events aren't held back
- **WHEN** the device starts playing an AirPlay origin a provider is still settling
- **THEN** the current snapshot already shows it playing and the playback state change is sent as an event, while the published snapshot stays as it was
