## MODIFIED Requirements

### Requirement: Idle screen
When nothing is playing (stopped, or no track and no input), the kiosk
SHALL show the current time and "<player name> · nothing playing".

#### Scenario: Stopped
- **WHEN** the snapshot is stopped and the player name is "Example Player"
- **THEN** the screen shows the time and "Example Player · nothing playing"

## ADDED Requirements

### Requirement: Input screen
When an audio input plays (the snapshot has an input and is not stopped),
the kiosk SHALL show the current time, a disc that turns slowly (one turn
in 20 to 30 seconds), the input's name, the session label as a badge, and
"for N min" since the input was selected, in whole minutes. It SHALL show no
artwork, track time, progress bar or queue. With reduced motion the disc
SHALL stand still.

#### Scenario: Optical input
- **WHEN** the snapshot has the optical input named "Example Input", selected 23 minutes ago, with the session label "Optical input"
- **THEN** the screen shows the time, the turning disc, "Example Input", the badge "Optical input" and "for 23 min", and no progress bar or "Up next"

#### Scenario: Just selected
- **WHEN** the input was selected 20 seconds ago
- **THEN** the screen shows "for 0 min"

#### Scenario: Reduced motion
- **WHEN** the browser prefers reduced motion and an input plays
- **THEN** the disc does not turn
