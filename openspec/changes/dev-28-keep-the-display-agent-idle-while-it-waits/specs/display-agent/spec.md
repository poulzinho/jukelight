## ADDED Requirements

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
