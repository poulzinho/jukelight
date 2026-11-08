## ADDED Requirements

### Requirement: Source of the session
The snapshot SHALL name its source: the music service's brand (if known)
and name, and the route it takes. When a provider claims the session, the
source SHALL be the provider's music service, with the provider as the
route; when the provider names no music service, the source SHALL be the
provider itself, without a route. When the provider claims the session but
is unreachable, the source SHALL be the provider, with its route marked
unavailable. When no provider claims it, the source SHALL be the device's
service (or its audio input) without a route. The snapshot SHALL also carry
the player's maker.

#### Scenario: Claimed by Music Assistant
- **WHEN** Music Assistant claims the session and names the music service "Qobuz" with the brand qobuz
- **THEN** the snapshot's source has the brand qobuz, the name "Qobuz" and the route "Music Assistant", available

#### Scenario: Music Assistant alone
- **WHEN** Music Assistant claims the session and names no music service
- **THEN** the snapshot's source is "Music Assistant", without a route

#### Scenario: Music Assistant unreachable
- **WHEN** Music Assistant claims the session but can't be reached
- **THEN** the snapshot's source is "Music Assistant" and its route "Music Assistant" is unavailable

#### Scenario: Unclaimed service
- **WHEN** no provider claims the session and the device plays from the service `Spotify`
- **THEN** the snapshot's source has the brand spotify and no route
