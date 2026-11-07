## ADDED Requirements

### Requirement: Reload after an update
When the player event stream announces a build that differs from the one in
the kiosk's own page, the kiosk SHALL reload the page, at most once per
announced build. When the builds match, or the page carries no build (for
example during development), it SHALL NOT reload.

#### Scenario: New build
- **WHEN** the page carries build `a1b2c3` and the stream announces build `d4e5f6`
- **THEN** the kiosk reloads the page

#### Scenario: Same build
- **WHEN** the page carries build `a1b2c3` and the stream announces build `a1b2c3`
- **THEN** the kiosk doesn't reload

#### Scenario: Still different after reloading
- **WHEN** the kiosk has reloaded for build `d4e5f6` and the reloaded page still carries build `a1b2c3`
- **THEN** the kiosk doesn't reload again for build `d4e5f6`

#### Scenario: No build in the page
- **WHEN** the page carries no build and the stream announces build `d4e5f6`
- **THEN** the kiosk doesn't reload
