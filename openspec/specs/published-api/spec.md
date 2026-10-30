# published-api Specification

## Purpose

The versioned HTTP API the kiosk consumes: the current player snapshot as
JSON and live updates over Server-Sent Events, in Jukelight's own terms.

## Requirements

### Requirement: Current snapshot
`GET /api/v1/player` SHALL return the current snapshot as JSON, with:
`online`, `deviceName`, `state` (`playing`, `paused`, `stopped`,
`buffering`), `session` (`label`, `available`), `track` (`title`,
`artists`, `album`) or null, `artUrl` or null, `position` (`elapsed`,
`duration`, `observedAt` in epoch milliseconds) or null, `quality`
(`label`, `detail`, `hiRes`) or null, and `queue` (a list of entries with
`title`, `artists`, `artUrl`) or null when not applicable.

#### Scenario: Playing track
- **WHEN** the player plays "Example Song" and the kiosk requests `GET /api/v1/player`
- **THEN** the response is 200 JSON with `state` "playing" and `track.title` "Example Song"

#### Scenario: Nothing playing
- **WHEN** the player is stopped with no content and the kiosk requests `GET /api/v1/player`
- **THEN** the response has `state` "stopped" and `track` null

### Requirement: Live updates
`GET /api/v1/player/events` SHALL be a Server-Sent Events stream. It SHALL
send the current snapshot immediately after a client connects, then a new
snapshot whenever a change is detected, except progress that matches the
predicted elapsed time. It SHALL send a comment line at least every 30
seconds so clients can detect a dead connection.

#### Scenario: Connect
- **WHEN** a client opens `GET /api/v1/player/events`
- **THEN** the first event contains the current snapshot

#### Scenario: Track change
- **WHEN** a client is connected and the track changes
- **THEN** the client receives an event with the new snapshot

#### Scenario: Normal progress
- **WHEN** a client is connected and only the elapsed time advances as predicted
- **THEN** the client receives no snapshot event

#### Scenario: Heartbeat
- **WHEN** a client is connected and nothing changes for 30 seconds
- **THEN** the client has received at least one comment line

### Requirement: No vendor data in the API
API responses SHALL contain neither vendor field names nor the address of
the player or any other system.

#### Scenario: Response content
- **WHEN** the player is configured at 192.168.1.40 and the kiosk requests `GET /api/v1/player`
- **THEN** the response contains neither "192.168.1.40" nor any BluOS field name such as "title1" or "streamUrl"

### Requirement: Kiosk files
The backend SHALL serve the kiosk app at `/` from files built into the
binary, so the kiosk needs no other server.

#### Scenario: Open the kiosk
- **WHEN** a browser requests `/`
- **THEN** the response is the kiosk's HTML page
