## MODIFIED Requirements

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

## ADDED Requirements

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
