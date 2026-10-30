## MODIFIED Requirements

### Requirement: Claimed origin uses the provider's content
When a session provider claims the device's origin and has content, the
snapshot SHALL use that provider's content, and the session label SHALL be
the provider's label, followed by " · " and the music service when the
content names one. The playback state and connectivity SHALL always come
from the device.

#### Scenario: Provider drives the device
- **WHEN** a provider labelled "Example Provider" claims the device's origin and reports the title "Provider Song", while the device reports the title "Example Provider"
- **THEN** the snapshot shows the title "Provider Song" with the session label "Example Provider"

#### Scenario: State comes from the device
- **WHEN** a provider claims the origin and reports playing, while the device reports paused
- **THEN** the snapshot's playback state is paused

#### Scenario: Provider names the music service
- **WHEN** a provider labelled "Example Provider" claims the origin and its content names the music service "Example Service"
- **THEN** the session label is "Example Provider · Example Service"

## ADDED Requirements

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
