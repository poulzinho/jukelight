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
(`label`, `detail`, `hiRes`) or null, `queue` (a list of entries with
`title`, `artists`, `artUrl`) or null when not applicable, `palette`
(`background` and `accent`, each `[red, green, blue]` from 0 to 255) or
null, and `input` (`kind`: `optical`, `analog`, `hdmi-arc` or `other`;
`name`; `startedAt`, when its signal started, in epoch milliseconds, or
null while waiting for a signal) or null.

#### Scenario: Playing track
- **WHEN** the player plays "Example Song" and the kiosk requests `GET /api/v1/player`
- **THEN** the response is 200 JSON with `state` "playing" and `track.title` "Example Song"

#### Scenario: Nothing playing
- **WHEN** the player is stopped with no content and the kiosk requests `GET /api/v1/player`
- **THEN** the response has `state` "stopped" and `track` null

#### Scenario: Album colours
- **WHEN** the snapshot has the background colour (73, 51, 38) and the accent colour (209, 147, 108)
- **THEN** the response has `palette.background` [73, 51, 38] and `palette.accent` [209, 147, 108]

#### Scenario: No album colours
- **WHEN** the snapshot has no colours
- **THEN** the response has `palette` null

#### Scenario: Input plays
- **WHEN** the player plays its optical input named "Example Input", whose signal started at 1,000,000 ms
- **THEN** the response has `input.kind` "optical", `input.name` "Example Input", `input.startedAt` 1000000, `session.label` "Optical input", `track` null and `queue` null

#### Scenario: Input waiting for a signal
- **WHEN** the player's optical input waits for a signal
- **THEN** the response has `input.startedAt` null

#### Scenario: No input
- **WHEN** the player plays a streaming service
- **THEN** the response has `input` null

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

### Requirement: Live streams end on shutdown
When the backend is asked to stop (SIGTERM or Ctrl-C), it SHALL end every
open live update stream before shutting down, so a graceful shutdown does
not wait for kiosks that never disconnect. Clients then reconnect to the
next start.

#### Scenario: Shutdown with a kiosk connected
- **WHEN** a client has `GET /api/v1/player/events` open and shutdown is requested
- **THEN** the event stream ends within 1 second

#### Scenario: No shutdown requested
- **WHEN** a client has the event stream open and no shutdown is requested
- **THEN** the stream stays open and keeps sending comment lines

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

### Requirement: Kiosk build
The backend SHALL identify the kiosk build it serves by a short identifier
derived from the built kiosk page, which changes whenever any kiosk file
changes. The kiosk page it serves SHALL carry the identifier as
`<meta name="jukelight-build" content="…">`. On every connection,
`GET /api/v1/player/events` SHALL send the identifier as a `build` event
right after the first snapshot.

#### Scenario: Page carries the build
- **WHEN** a browser requests `/`
- **THEN** the page contains `<meta name="jukelight-build">` with the backend's build identifier

#### Scenario: Stream announces the build
- **WHEN** a client opens `GET /api/v1/player/events`
- **THEN** the first event is the current snapshot and the second a `build` event with the same identifier as the page

### Requirement: Cover and backdrop URLs
In the snapshot, `artUrl` SHALL point to the large version of the track's
cover, and `backdropUrl` to its backdrop version; both SHALL be `null`
without a cover. The artwork of "Up next" entries SHALL keep pointing to
the original version.

#### Scenario: Track with a cover
- **WHEN** the current track has a cover
- **THEN** the snapshot has an `artUrl` for the large version and a `backdropUrl` for the backdrop version of the same cover

#### Scenario: Track without a cover
- **WHEN** the current track has no cover
- **THEN** `artUrl` and `backdropUrl` are `null`
