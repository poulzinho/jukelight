## MODIFIED Requirements

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
