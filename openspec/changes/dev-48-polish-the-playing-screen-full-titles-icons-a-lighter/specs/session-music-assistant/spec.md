## ADDED Requirements

### Requirement: Settling the session
The Music Assistant provider SHALL report that it is settling the device's
origin while it follows an AirPlay origin and Music Assistant hasn't
answered since the provider started following it, and while it claims the
origin but hasn't fetched content for it yet. It SHALL stop settling once
Music Assistant has answered, its content has been fetched, or Music
Assistant turned out to be unreachable.

#### Scenario: AirPlay, no answer yet
- **WHEN** the device has just reported an AirPlay origin and Music Assistant hasn't answered yet
- **THEN** the provider is settling

#### Scenario: AirPlay, answered
- **WHEN** Music Assistant has answered for the AirPlay origin, whether it drives the device or not
- **THEN** the provider isn't settling

#### Scenario: Claimed, no content yet
- **WHEN** the provider has just claimed the origin and hasn't fetched content yet
- **THEN** the provider is settling, until the content has been fetched

#### Scenario: Unreachable
- **WHEN** Music Assistant doesn't answer the provider's request
- **THEN** the provider isn't settling, and the snapshot shows it unreachable as before
