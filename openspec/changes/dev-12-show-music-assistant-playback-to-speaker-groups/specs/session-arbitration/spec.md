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
