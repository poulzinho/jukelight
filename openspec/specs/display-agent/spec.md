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
10 minutes, configurable), it SHALL switch the outputs off once and keep
retrying. When the backend answers again, its decision applies.

#### Scenario: Backend unreachable
- **WHEN** the fail-safe period is 2 seconds and the backend has been unreachable for 3 seconds
- **THEN** the agent has run `swaymsg "output * power off"` once

#### Scenario: Backend back after the fail-safe
- **WHEN** the agent switched the outputs off by fail-safe and the backend comes back with the decision on
- **THEN** the agent runs `swaymsg "output * power on"`
