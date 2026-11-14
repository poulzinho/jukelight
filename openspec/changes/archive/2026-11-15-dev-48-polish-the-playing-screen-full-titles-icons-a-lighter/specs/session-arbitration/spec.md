## ADDED Requirements

### Requirement: A new session settles before it shows
While a session provider is still settling the device's origin (it hasn't
answered yet whether it drives the device, or it claims the device but
hasn't fetched its first content), the published player snapshot (what
the kiosk and other live clients receive) SHALL stay as it was, for at
most 2 seconds. As soon as no provider is settling, or after those 2
seconds, the published snapshot SHALL catch up at once. The current
snapshot and the player's events SHALL follow the device as usual.

#### Scenario: A group resumes
- **WHEN** the snapshot shows the idle screen, the device reports an AirPlay origin, and the provider answers after half a second that it drives the device, with its content
- **THEN** the published snapshot changes once, from the idle screen straight to the provider's content, with no AirPlay snapshot in between

#### Scenario: The provider doesn't answer in time
- **WHEN** the device reports an AirPlay origin and the provider is still settling after 2 seconds
- **THEN** the published snapshot shows the device's AirPlay session

#### Scenario: Nothing settling
- **WHEN** the device reports a new origin and no provider is settling
- **THEN** the published snapshot changes at once

#### Scenario: The current snapshot and events aren't held back
- **WHEN** the device starts playing an AirPlay origin a provider is still settling
- **THEN** the current snapshot already shows it playing and the playback state change is sent as an event, while the published snapshot stays as it was
