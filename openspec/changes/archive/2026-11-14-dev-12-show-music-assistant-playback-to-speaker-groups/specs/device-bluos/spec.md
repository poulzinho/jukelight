## ADDED Requirements

### Requirement: Subwoofer
The device monitor SHALL report a subwoofer output while `/SyncStatus` has
`hasSubwoofer="true"`, and none when the attribute is missing or not
`true`; BluOS leaves it out while the subwoofer is switched off. When
`/Status` reports a changed `syncStat`, the monitor SHALL read
`/SyncStatus` again and report the change without waiting for another
long-poll.

#### Scenario: Subwoofer on
- **WHEN** `/SyncStatus` reports `hasSubwoofer="true"`
- **THEN** the device snapshot's outputs include a subwoofer

#### Scenario: Subwoofer off
- **WHEN** `/SyncStatus` has no `hasSubwoofer` attribute
- **THEN** the device snapshot has no subwoofer output

#### Scenario: Subwoofer switched off while playing
- **WHEN** the subwoofer is switched off, `/Status` returns with a changed `syncStat`, and the player's next long-poll would wait 100 seconds
- **THEN** the monitor reports no subwoofer within a few seconds
