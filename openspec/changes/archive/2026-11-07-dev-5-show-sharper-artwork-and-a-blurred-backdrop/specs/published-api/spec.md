## ADDED Requirements

### Requirement: Cover and backdrop URLs
In the snapshot, `artUrl` SHALL point to the large version of the track's
cover, and `backdropUrl` to its backdrop version; both SHALL be `null`
without a cover. The artwork of "Up next" entries SHALL keep pointing to
the original version.

#### Scenario: Track with a cover
- **WHEN** the current track has a cover
- **THEN** the snapshot has an `artUrl` for the large version and a `backdropUrl` for the backdrop version of the same cover

#### Scenario: Track without a cover
- **WHEN** the current track has no cover
- **THEN** `artUrl` and `backdropUrl` are `null`
