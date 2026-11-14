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

## MODIFIED Requirements

### Requirement: A paused group
After the provider claimed a group session, when the device reports
stopped and Music Assistant's queue for the player still holds the same
current item, the provider SHALL hold the session: it SHALL keep claiming
the device and report that item with its position, "Up next" and the
speakers, marked as held. Music Assistant reports a pause about a second
after the device stops, so while its queue still reports that item as
playing, the hold SHALL last up to 5 seconds; it SHALL end then if the
queue still plays. The hold SHALL also end when the device reports
anything other than stopped, or when the queue's current item changes or
clears.

#### Scenario: Group paused
- **WHEN** a group session was claimed with "Example Song", then the device reports stopped with its own previous queue and Music Assistant's queue is idle with "Example Song" at 121 s
- **THEN** the provider claims the device and reports "Example Song" at 121 s, held

#### Scenario: Music Assistant reports the pause late
- **WHEN** a group session was claimed with "Example Song", the device reports stopped, Music Assistant's queue still reports "Example Song" playing, and a second later reports it idle
- **THEN** the provider holds the session throughout, and reports "Example Song", held

#### Scenario: The group plays on without the player
- **WHEN** the provider holds a paused group session and Music Assistant's queue keeps reporting the same item playing for more than 5 seconds
- **THEN** the provider no longer claims the device

#### Scenario: Group resumed
- **WHEN** the provider holds a paused group session and the device reports an AirPlay session that Music Assistant drives
- **THEN** the provider claims it as a group session, not held

#### Scenario: Something else plays
- **WHEN** the provider holds a paused group session and the device reports playing its own Qobuz queue
- **THEN** the provider no longer claims the device

#### Scenario: Queue cleared
- **WHEN** the provider holds a paused group session and Music Assistant's queue no longer has a current item
- **THEN** the provider no longer claims the device
