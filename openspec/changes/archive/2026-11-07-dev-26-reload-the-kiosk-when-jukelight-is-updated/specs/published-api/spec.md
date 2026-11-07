## ADDED Requirements

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
