## MODIFIED Requirements

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
