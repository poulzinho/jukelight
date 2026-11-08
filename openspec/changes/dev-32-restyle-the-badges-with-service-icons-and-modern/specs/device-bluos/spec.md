## ADDED Requirements

### Requirement: Service brand
The device monitor SHALL recognise the brand of the service a BluOS
player plays from its service id: `Qobuz` and `QobuzConnect` as qobuz,
`Spotify` as spotify, `Tidal` and `TidalConnect` as tidal, `Deezer` as
deezer, `AirPlay` as airplay, and `Bluetooth` as bluetooth. Other service
ids SHALL have no brand; the service name still names them.

#### Scenario: Qobuz Connect
- **WHEN** `/Status` reports the service `QobuzConnect`
- **THEN** the origin's service has the brand qobuz

#### Scenario: Unknown service
- **WHEN** `/Status` reports a service id without a known brand
- **THEN** the origin's service has no brand

### Requirement: Player maker
The device monitor SHALL report the player's maker from the `brand`
attribute of `/SyncStatus`, as a brand (`Bluesound` as bluesound); any
other maker SHALL be reported without a brand.

#### Scenario: Bluesound player
- **WHEN** `/SyncStatus` reports `brand="Bluesound"`
- **THEN** the device snapshot's maker has the brand bluesound

#### Scenario: Another BluOS maker
- **WHEN** `/SyncStatus` reports `brand="Example Maker"`
- **THEN** the device snapshot's maker has no brand
