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
a service origin, "Stream" for a stream origin, the input's kind for an
input origin ("Optical input", "Analogue input", "HDMI ARC", "Input"), and
empty when there is no origin. For an input origin the snapshot SHALL carry
the input.

#### Scenario: Native service
- **WHEN** no provider is registered and the device reports the service origin "Qobuz" with the title "Example Song"
- **THEN** the snapshot shows "Example Song" with the session label "Qobuz"

#### Scenario: Unclaimed stream
- **WHEN** no provider is registered and the device reports a stream origin with the title "Music Assistant"
- **THEN** the snapshot shows the title "Music Assistant" with the session label "Stream"

#### Scenario: Optical input
- **WHEN** the device reports an optical input named "Example Input"
- **THEN** the snapshot has the session label "Optical input", the input named "Example Input" and no track

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
