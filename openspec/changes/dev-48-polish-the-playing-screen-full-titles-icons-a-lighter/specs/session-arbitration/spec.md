## ADDED Requirements

### Requirement: A new session settles before it shows
While a session provider is still settling the device's origin (it hasn't
answered yet whether it drives the device, or it claims the device but
hasn't fetched its first content), the player snapshot SHALL stay as it
was, for at most 2 seconds. As soon as no provider is settling, or after
those 2 seconds, the snapshot SHALL be rebuilt at once from the device and
the providers, as usual.

#### Scenario: A group resumes
- **WHEN** the snapshot shows the idle screen, the device reports an AirPlay origin, and the provider answers after half a second that it drives the device, with its content
- **THEN** the snapshot changes once, from the idle screen straight to the provider's content, with no AirPlay snapshot in between

#### Scenario: The provider doesn't answer in time
- **WHEN** the device reports an AirPlay origin and the provider is still settling after 2 seconds
- **THEN** the snapshot shows the device's AirPlay session

#### Scenario: Nothing settling
- **WHEN** the device reports a new origin and no provider is settling
- **THEN** the snapshot changes at once
