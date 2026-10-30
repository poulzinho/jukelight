## ADDED Requirements

### Requirement: Display decisions
`GET /api/v1/display` SHALL be a Server-Sent Events stream of display
decisions. It SHALL send the current decision immediately after a client
connects, then every new decision, each as an event named `display` whose
data is `on` or `off`. It SHALL send a comment line at least every 30
seconds.

#### Scenario: Connect
- **WHEN** the decision is on and a client opens `GET /api/v1/display`
- **THEN** the first event is a `display` event with the data `on`

#### Scenario: Decision changes
- **WHEN** a client is connected and the decision changes to off
- **THEN** the client receives a `display` event with the data `off`

#### Scenario: Heartbeat
- **WHEN** a client is connected and the decision does not change for 30 seconds
- **THEN** the client has received at least one comment line

#### Scenario: Shutdown
- **WHEN** a client has the display stream open and shutdown is requested
- **THEN** the stream ends within 1 second
