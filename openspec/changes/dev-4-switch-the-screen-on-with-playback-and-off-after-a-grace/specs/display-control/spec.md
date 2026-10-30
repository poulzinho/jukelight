## Purpose

Decides when the display should be on or off from the player's playback
state and connectivity, so the screen wakes with the music and goes dark a
grace period after it stops.

## ADDED Requirements

### Requirement: Display on while music plays
The display decision SHALL be on, immediately, while the player is online
and playing or buffering, including when the display was off.

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
