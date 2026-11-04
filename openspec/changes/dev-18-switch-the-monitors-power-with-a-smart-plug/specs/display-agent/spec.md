## ADDED Requirements

### Requirement: Run the power commands
When a power-on command or a power-off command is configured, the agent
SHALL run it with every switch, after switching the outputs: the power-on
command after switching the outputs on, the power-off command after
switching them off. Without these commands, the agent SHALL only switch the
outputs.

#### Scenario: Switch on with a power-on command
- **WHEN** a power-on command is configured and the stream sends the decision on
- **THEN** the agent runs `swaymsg "output * power on"` and then the power-on command

#### Scenario: Switch off with a power-off command
- **WHEN** a power-off command is configured and the stream sends the decision off
- **THEN** the agent runs `swaymsg "output * power off"` and then the power-off command

#### Scenario: No power commands
- **WHEN** no power commands are configured and the stream sends the decision on
- **THEN** the agent runs only `swaymsg "output * power on"`

### Requirement: Retry a failing power command
When a power command fails, the agent SHALL log it and run it again at the
retry interval until it succeeds or the decision changes. A failing power
command SHALL NOT count towards the failed switches that stop the agent.

#### Scenario: Plug unreachable for a while
- **WHEN** the retry interval is 1 second and the power-on command fails twice and then succeeds
- **THEN** the agent has run the power-on command 3 times and keeps running

#### Scenario: Decision changes while retrying
- **WHEN** the power-on command keeps failing and the decision changes to off
- **THEN** the agent stops running the power-on command and runs the power-off command

## MODIFIED Requirements

### Requirement: Fail-safe without the backend
When the agent cannot reach the backend for the fail-safe period (default
10 minutes, configurable), it SHALL switch the outputs off once, with the
power-off command when one is configured, and keep retrying. When the
backend answers again, its decision applies.

#### Scenario: Backend unreachable
- **WHEN** the fail-safe period is 2 seconds and the backend has been unreachable for 3 seconds
- **THEN** the agent has run `swaymsg "output * power off"` once

#### Scenario: Backend unreachable with a power-off command
- **WHEN** a power-off command is configured, the fail-safe period is 2 seconds and the backend has been unreachable for 3 seconds
- **THEN** the agent has run `swaymsg "output * power off"` and then the power-off command, once each

#### Scenario: Backend back after the fail-safe
- **WHEN** the agent switched the outputs off by fail-safe and the backend comes back with the decision on
- **THEN** the agent runs `swaymsg "output * power on"`
