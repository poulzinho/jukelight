# display-control Specification

## Purpose
Decides when the display should be on or off from the player's playback
state and connectivity, so the screen wakes with the music and goes dark a
grace period after it stops.

## Requirements

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

### Requirement: Display off after the grace period
While the player is paused, stopped or offline, the display decision SHALL
stay on until the grace period has passed since the player stopped playing,
and then become off. The grace period SHALL start again when the player
plays again before it ends. Offline counts as not playing, whatever
playback state the player last reported.

#### Scenario: Paused within the grace period
- **WHEN** the grace period is 10 minutes and the player has been paused for 9 minutes
- **THEN** the decision is on

#### Scenario: Paused past the grace period
- **WHEN** the grace period is 10 minutes and the player has been paused for 10 minutes
- **THEN** the decision is off

#### Scenario: Stopped
- **WHEN** the grace period is 10 minutes and the player has been stopped for 10 minutes
- **THEN** the decision is off

#### Scenario: Player offline
- **WHEN** the grace period is 10 minutes and the player went offline 10 minutes ago, having last reported playing
- **THEN** the decision is off

#### Scenario: Playback resumes within the grace period
- **WHEN** the grace period is 10 minutes, the player pauses, plays again after 5 minutes and pauses again
- **THEN** the decision stays on until 10 minutes after the second pause

#### Scenario: No grace period
- **WHEN** the grace period is 0 minutes and the player pauses
- **THEN** the decision becomes off at once

### Requirement: Decision at start
When Jukelight starts, the display decision SHALL be on, and the grace
period SHALL count from the start until the player reports playing.

#### Scenario: Start with nothing playing
- **WHEN** Jukelight starts, the grace period is 10 minutes and the player stays stopped
- **THEN** the decision is on at the start and becomes off 10 minutes later

#### Scenario: Start while music plays
- **WHEN** Jukelight starts and the player's first report is playing
- **THEN** the decision is on and stays on

### Requirement: Only changes are published
A new display decision SHALL be published only when it differs from the
previous one.

#### Scenario: Track change while playing
- **WHEN** the decision is on and the track changes while the player keeps playing
- **THEN** no new decision is published

#### Scenario: Pause and resume within the grace period
- **WHEN** the decision is on and the player pauses and resumes within the grace period
- **THEN** no new decision is published

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
