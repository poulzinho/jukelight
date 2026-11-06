# display-agent Specification

## Purpose
Runs on the host next to the kiosk browser, follows the published display
decisions and switches the monitor's output on or off, so the screen follows
the music without touching the kiosk page.

## Requirements

### Requirement: Follow the display decisions
The agent SHALL follow the display decision stream and switch all outputs
on when the decision is on and off when it is off.

#### Scenario: Decision on
- **WHEN** the stream sends the decision on
- **THEN** the agent runs `swaymsg "output * power on"`

#### Scenario: Decision off
- **WHEN** the stream sends the decision off
- **THEN** the agent runs `swaymsg "output * power off"`

### Requirement: Apply the decision on connect
After every connection to the backend, the agent SHALL apply the first
decision it receives, even when it equals the last decision it applied, so
the outputs match the backend after any interruption.

#### Scenario: Reconnect with the same decision
- **WHEN** the agent applied off, loses the connection, reconnects and receives off
- **THEN** the agent runs `swaymsg "output * power off"` again

### Requirement: Reconnect to the backend
When the connection to the backend ends, fails, or sends nothing (not even
a heartbeat) for the silence limit (default 60 seconds, configurable), the
agent SHALL connect again, retrying at the retry interval (default 5
seconds, configurable) until it succeeds.

#### Scenario: Backend restarts
- **WHEN** the retry interval is 1 second and the backend stops and starts again 3 seconds later, with the decision on
- **THEN** the agent reconnects within 2 seconds of the restart and runs `swaymsg "output * power on"`

#### Scenario: Silent connection
- **WHEN** the silence limit is 2 seconds and the connection stays open but sends nothing for 3 seconds
- **THEN** the agent has closed it and connected again

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

### Requirement: Stop after repeated failed switches
When switching the outputs fails 3 times in a row, the agent SHALL stop
with an error, so its service manager starts it again with the current
environment (for example a new sway session). A successful switch SHALL
reset the count.

#### Scenario: Stale sway session
- **WHEN** switching the outputs fails for 3 decisions in a row
- **THEN** the agent stops with a non-zero exit status

#### Scenario: Occasional failure
- **WHEN** one switch fails and the next one succeeds
- **THEN** the agent keeps running

### Requirement: Stay idle while waiting
While the connection to the backend is open and nothing arrives, the agent
SHALL wake up only when the silence limit could have been reached, not on
a short fixed interval, so it uses almost no CPU between messages.

#### Scenario: Quiet connection
- **WHEN** the silence limit is 60 seconds and the connection stays open without data for 5 seconds
- **THEN** the agent has checked the silence limit at most twice in that time

#### Scenario: Silence limit still applies
- **WHEN** the silence limit is 2 seconds and the connection stays open but sends nothing for 3 seconds
- **THEN** the agent has closed it and connected again

### Requirement: Run the power commands
When a power-on command or a power-off command is configured, the agent
SHALL run it with every switch, after switching the outputs: the power-on
command after switching the outputs on, the power-off command after
switching them off, also when switching the outputs failed (the monitor's
power doesn't depend on sway). Without these commands, the agent SHALL
only switch the outputs.

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
When a power command fails, the agent SHALL log it and run it again until
it succeeds or the decision changes. The first wait SHALL be the retry
interval; each further wait SHALL be twice the one before; no wait SHALL
be longer than 5 minutes,
so a plug that refuses the password is not kept locked. A failing power
command SHALL NOT count towards the failed switches that stop the agent.

#### Scenario: Plug unreachable for a while
- **WHEN** the retry interval is 1 second and the power-on command fails twice and then succeeds
- **THEN** the agent has run the power-on command 3 times and keeps running

#### Scenario: Plug keeps refusing
- **WHEN** the retry interval is 1 second and the power-on command keeps failing for 6 seconds
- **THEN** the agent has run it 3 times (after waits of 1 and 2 seconds), not 6 times

#### Scenario: Decision changes while retrying
- **WHEN** the power-on command keeps failing and the decision changes to off
- **THEN** the agent stops running the power-on command and runs the power-off command
