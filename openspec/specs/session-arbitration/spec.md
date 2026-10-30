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
content the device reports. The session label SHALL be the service name for
a service origin, "Stream" for a stream origin, and empty when there is no
origin.

#### Scenario: Native service
- **WHEN** no provider is registered and the device reports the service origin "Qobuz" with the title "Example Song"
- **THEN** the snapshot shows "Example Song" with the session label "Qobuz"

#### Scenario: Unclaimed stream
- **WHEN** no provider is registered and the device reports a stream origin with the title "Music Assistant"
- **THEN** the snapshot shows the title "Music Assistant" with the session label "Stream"

### Requirement: Claims are decided offline
Whether a provider claims an origin SHALL be decided from the provider's
configuration and the origin alone, without contacting the provider, so
that the label stays correct while the provider is unreachable.

#### Scenario: Claim while the provider is down
- **WHEN** a provider's network connection is down and the device reports an origin that matches the provider's configuration
- **THEN** the provider still claims the origin

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
