## ADDED Requirements

### Requirement: Subwoofer
The device monitor SHALL report a subwoofer output while `/SyncStatus` has
`hasSubwoofer="true"`, and none when the attribute is missing or not
`true`; BluOS leaves it out while the subwoofer is switched off.

#### Scenario: Subwoofer on
- **WHEN** `/SyncStatus` reports `hasSubwoofer="true"`
- **THEN** the device snapshot's outputs include a subwoofer

#### Scenario: Subwoofer off
- **WHEN** `/SyncStatus` has no `hasSubwoofer` attribute
- **THEN** the device snapshot has no subwoofer output
