## MODIFIED Requirements

### Requirement: Current snapshot
`GET /api/v1/player` SHALL return the current snapshot as JSON, with:
`online`, `deviceName`, `state` (`playing`, `paused`, `stopped`,
`buffering`), `session` (`label`, `available`), `track` (`title`,
`artists`, `album`) or null, `artUrl` or null, `position` (`elapsed`,
`duration`, `observedAt` in epoch milliseconds) or null, `quality`
(`label`, `detail`, `hiRes`) or null, `queue` (a list of entries with
`title`, `artists`, `artUrl`) or null when not applicable, and `palette`
(`background` and `accent`, each `[red, green, blue]` from 0 to 255) or null.

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

## ADDED Requirements

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
