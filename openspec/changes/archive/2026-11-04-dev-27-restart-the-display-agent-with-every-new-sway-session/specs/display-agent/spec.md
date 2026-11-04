## ADDED Requirements

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
