## ADDED Requirements

### Requirement: Source of the session
The snapshot SHALL name its source: the music service's brand (if known)
and name, and the route it takes. When a provider claims the session, the
source SHALL be the provider's music service, with the provider as the
route; when the provider claims it but is unreachable, the route SHALL be
marked unavailable and the source SHALL be the device's own service. When
no provider claims it, the source SHALL be the device's service (or its
audio input) without a route.

#### Scenario: Claimed by Music Assistant
- **WHEN** Music Assistant claims the session and names the music service "Qobuz" with the brand qobuz
- **THEN** the snapshot's source has the brand qobuz, the name "Qobuz" and the route "Music Assistant", available

#### Scenario: Music Assistant unreachable
- **WHEN** Music Assistant claims the session but can't be reached, and the device reports the service `Qobuz`
- **THEN** the snapshot's source has the brand qobuz and the route "Music Assistant", unavailable

#### Scenario: Unclaimed service
- **WHEN** no provider claims the session and the device plays from the service `Spotify`
- **THEN** the snapshot's source has the brand spotify and no route
