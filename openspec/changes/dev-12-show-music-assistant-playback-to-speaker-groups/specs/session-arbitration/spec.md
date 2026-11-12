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
