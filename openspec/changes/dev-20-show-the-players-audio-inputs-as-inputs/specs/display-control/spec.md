## MODIFIED Requirements

### Requirement: Display on while music plays
The display decision SHALL be on, immediately, while the player is online
and playing or buffering, including when the display was off. While an
audio input plays, this holds only within the input time limit (see "Inputs
count as playing for a limited time").

#### Scenario: Playback starts
- **WHEN** the display is off and the player starts playing
- **THEN** the decision becomes on at once

#### Scenario: Buffering
- **WHEN** the display is off and the player is online and buffering
- **THEN** the decision becomes on at once

## ADDED Requirements

### Requirement: Inputs count as playing for a limited time
The player notices when a signal starts on an audio input, but not when it
stops. An input SHALL count as playing only once its signal started, and
only until the input time limit (default 90 minutes) has passed since then;
after that it SHALL count as not playing, and the grace period starts. An
input waiting for a signal SHALL count as not playing. A new start of the
signal (for example after selecting the input again) SHALL start the limit
again. Other sources are not affected.

#### Scenario: Within the input time limit
- **WHEN** the input time limit is 90 minutes and an input's signal started 80 minutes ago
- **THEN** the decision is on

#### Scenario: Past the limit and the grace period
- **WHEN** the input time limit is 90 minutes, the grace period is 10 minutes, and an input's signal started 100 minutes ago
- **THEN** the decision is off

#### Scenario: Waiting for a signal
- **WHEN** the grace period is 10 minutes and an input has been waiting for a signal for 10 minutes
- **THEN** the decision is off

#### Scenario: Signal starts again
- **WHEN** the decision is off after the input time limit and the input's signal starts again
- **THEN** the decision becomes on at once

#### Scenario: Another source
- **WHEN** a streaming service plays for 3 hours
- **THEN** the decision stays on
