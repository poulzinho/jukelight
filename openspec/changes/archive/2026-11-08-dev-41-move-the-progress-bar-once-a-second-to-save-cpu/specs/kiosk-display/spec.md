## MODIFIED Requirements

### Requirement: Progress between updates
The kiosk SHALL advance the progress bar and elapsed time by itself from the
snapshot's position while the state is playing, and SHALL keep them still
while paused. The bar and the elapsed time SHALL move together, once a
second, when the elapsed time reaches the next whole second, and the bar
SHALL move without a transition, so nothing in the progress animates
between those moments.

#### Scenario: Progress without updates
- **WHEN** the snapshot reports elapsed 60 s of 200 s while playing and no update arrives for 10 seconds
- **THEN** the screen shows about 1:10 elapsed

#### Scenario: Once a second
- **WHEN** the snapshot reports elapsed 60.4 s of 200 s while playing
- **THEN** the bar and "1:00" stay as they are for 0.6 s, then the bar moves to 61 s and the time shows "1:01" at the same moment

#### Scenario: No transition
- **WHEN** a track plays
- **THEN** the bar's fill has no transition and no animation runs in the progress
