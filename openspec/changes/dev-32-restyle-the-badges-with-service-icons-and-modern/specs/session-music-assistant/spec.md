## ADDED Requirements

### Requirement: Music provider brand
The provider SHALL recognise the brand of the current item's music
provider from the `domain` Music Assistant reports for the provider
instance, not from its renameable name. A provider in the `opensubsonic`
domain SHALL be Navidrome when its name contains "Navidrome" (in any
case), and Subsonic otherwise. Providers of local files (domains
`filesystem_*`) SHALL be the brand Library. A domain without a known brand
SHALL have no brand; its name still names the service. As before, Music
Assistant's built-in provider names no service.

#### Scenario: Renamed Qobuz provider
- **WHEN** the current item's provider is in the domain `qobuz` and Music Assistant names it "My Qobuz"
- **THEN** the content's music service has the brand qobuz and the name "My Qobuz"

#### Scenario: Navidrome
- **WHEN** the current item's provider is in the domain `opensubsonic` and named "Navidrome"
- **THEN** the content's music service has the brand navidrome

#### Scenario: Another Subsonic server
- **WHEN** the current item's provider is in the domain `opensubsonic` and named "OpenSubsonic Media Server Library"
- **THEN** the content's music service has the brand subsonic and the name "Subsonic"

#### Scenario: Local files
- **WHEN** the current item's provider is in a `filesystem_local` domain
- **THEN** the content's music service has the brand library

#### Scenario: Unknown domain
- **WHEN** the current item's provider is in a domain without a known brand and named "Example Service"
- **THEN** the content's music service has no brand and the name "Example Service"
