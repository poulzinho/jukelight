## Context

- **Domain:** `Content.service: Option<String>` names the music service
  behind a provider session; `Origin::Service { id, name, cast }` is the
  device's service; `Session { label, available }` is the published
  label ("Music Assistant · Qobuz"). `AudioQuality` has a format (codec,
  bit depth, sample rate) or a bitrate, a label and a detail.
- **Music Assistant adapter:** reads `providers` (`instance_id`, `name`,
  `domain`, …) and names the current item's provider by `name`, shortening
  "OpenSubsonic Media Server Library" to "Subsonic". The recorded fixtures
  show domains such as `qobuz`, `opensubsonic`, `builtin`.
- **BluOS:** `/Status` has the service id (`Qobuz`, `QobuzConnect`,
  `Spotify`, `Capture`, `http`, …) and `serviceName`; `/SyncStatus` has
  `brand="Bluesound"`, `model` and `modelName`.
- **Kiosk:** `Badges.tsx` renders three pills (session label, HI-RES,
  quality label · detail), wrapping inside each pill in portrait (DEV-5
  acceptance). The idle screen shows "<player name> · nothing playing".
- **Open source:** brand logos and the Hi-Res Audio logo must never be in
  the repository; they are user-supplied (decided for the Hi-Res logo
  earlier; extended to all brand logos on 2026-11-08).
- **Mocks (2026-11-08, on DEV-32):** dark badges with wordmarks or icon and
  name; MA inside the badge; the HI-RES tag; the quality as text; the
  Bluesound footer signature; reviewed on dark and light backdrops.

## Goals / Non-Goals

**Goals:** recognisable sources from across the room; one line that never
wraps, with a clear order of what gives way; logos only when the user
supplies them, with a good-looking fallback otherwise; the source's brand
robust against renamed providers.

**Non-Goals:** logos in the repo; uploading logos in the browser (DEV-6);
speaker groups (DEV-12, DEV-13); asking Subsonic servers for their type.

## Decisions

### 1. Brands as short identifiers

A brand is a short lowercase identifier (`qobuz`, `spotify`, `tidal`,
`apple-music`, `deezer`, `navidrome`, `subsonic`, `library`, `plex`,
`jellyfin`, `music-assistant`, `airplay`, `bluetooth`, `bluesound`,
`hi-res`), the same as the logo's file name. In the domain it is a
`Brand` newtype; the adapters map their vendor values to it. A
`Service { brand: Option<Brand>, name }` replaces `Content.service`'s plain
name; the snapshot's `Source { service, via: Option<Route> }` has a
`Route { service, available }`; `Origin::Service` gains its `brand`; the
device and player snapshots gain the player's `maker` (`Option<Brand>`).
Session providers may name their own brand (`SessionProvider::brand`,
`None` by default; Music Assistant's is `music-assistant`).

- Alternative: brand names as display text. Rejected: renamed providers
  and languages would break the mapping to logos.

### 2. Brand recognition per adapter

- **MA:** by `domain` (the recorded fixtures confirm `qobuz` and
  `opensubsonic`; the other domain names follow Music Assistant's provider
  names and are unverified, where a wrong one only means the fallback
  badge); `opensubsonic` is navidrome when the provider name
  contains "navidrome" (case-insensitive), else subsonic; `filesystem_*`
  is library. Unknown domains: no brand. MA's built-in provider still names
  no service (the existing requirement), as it stands for MA itself.
- **BluOS:** by service id (`Qobuz`, `QobuzConnect` → qobuz, …); the maker
  from `/SyncStatus` `brand` (`Bluesound` → bluesound; others no brand).
- **Arbitration** builds the snapshot's source: the provider's service
  with the provider as route (available or not), or the device's service
  without a route; for an input, the input (no brand, generic icon).

- Alternative for Navidrome: a `.env` setting, or asking the server
  (OpenSubsonic `ping`). Rejected: one more setting to keep in sync, or
  the server's address and credentials in Jukelight.

### 3. Logos from a user folder, detected by the backend

`BRAND_ASSETS_DIR` (default `/brand-assets` in the container), mounted
read-only from `./brand-assets` next to `.env` in `docker-compose.yml`
(the folder may be missing). The backend looks up `<brand>.svg`, then
`<brand>.png`, when it builds a snapshot's DTO and when serving
`/api/v1/logos/{brand}`, with a short cache (a few seconds) so added or
removed files show up without a restart. Logo URLs carry the file's
modification time (`?v=…`), so the browser fetches a replaced logo.
Responses: the file with its content type, `Cache-Control` long-lived;
404 for an unknown brand or a missing file; brands are restricted to the
identifier alphabet (`[a-z0-9-]`), so no path tricks.

`.gitignore` gets `brand-assets/`. The reference setup describes the
folder, the official sources per brand, and copying it to the host. It
creates the folder before the first `docker compose up`: a missing bind
mount source is created by Docker, owned by root, and copying logos into
it then fails (found during the acceptance).

- Alternative: logos in the image or the repo. Rejected: trademarks.
- Alternative: the kiosk probes for logo files itself. Rejected: one
  request per brand per screen, and 404s in the browser's log.

### 4. Source badge and fallback

A dark badge (`bg-black/38`, a fine inner border, fully rounded), height
about 1.85 times the text size. Inside: the logo (wordmarks at a
per-brand height, so they look equally heavy: a small table in the kiosk,
with a default), or a Lucide icon for the brand (`audio-lines` for
services without one, `server` for Subsonic, `folder` for Library, `cable`
for inputs, and the like) and the name in Inter 600. Then "via" and the
route's logo (or its name). An unavailable route is its own outlined,
greyed badge (logo in grey, or the name) before the source's badge.

Lucide (`lucide-react`, ISC) is the only new kiosk dependency; brand
icons that are trademarks are not taken from it.

### 5. The quality line and what gives way

One flex row with `white-space: nowrap`:
`[source badge] [Hi-Res] quality · detail`. The quality reads "24 bit /
96 kHz FLAC" (lossless, from the new fields) or "160 kbps" (lossy), with
`font-variant-numeric: tabular-nums`; HI-RES uses Inter's `case`
feature and +0.08 em tracking. The detail is the only shrinkable item
(`min-width: 0`, ellipsis); when its visible width falls below about four
characters, it is hidden. If the row still doesn't fit, the "via" part is
hidden as a whole (measured with a `ResizeObserver` on the row, no layout
animation).

### 6. Device signature

A footer component on every screen: the device's logo (about 2 vmin high,
80 % opacity), or without one a streamer icon (Lucide `radio-receiver`)
and the maker's name for a known maker (from the acceptance: the maker
shouldn't simply vanish without its logo), then the player's name at 60 %
opacity, centred in the bottom margin
in portrait, in the bottom-right corner in landscape. In portrait its
space is part of the fixed slots (the bottom padding grows by its height),
so "Up next" keeps its place. The idle screen drops the player name from
its line ("nothing playing"). The connection indicator moves from the
bottom-right to the top-right corner, so it never covers the signature in
landscape (the spec only asks for "a corner").

## Risks / Trade-offs

- [Wordmarks vary in shape] → per-brand heights, tuned on screenshots with
  the user's logos; unknown brands use a default height.
- [A user supplies a dark-on-transparent logo] → it would be invisible on
  the dark badge; the reference setup asks for the dark-background
  variant, and the admin app (DEV-6) can preview it later.
- [Navidrome detection depends on the provider's name] → documented; the
  fallback (Subsonic) is still correct, only less specific.
- [The reference host needs the folder copied] → a step in the reference
  setup; without it, the fallback badges show.

## Tests

- **Domain:** `Brand` identifiers (alphabet), `Source` and `Service`.
- **Adapter fixtures:** MA "Renamed Qobuz provider", "Navidrome", "Another
  Subsonic server", "Local files", "Unknown domain" (scrubbed fixtures,
  adding a renamed-provider and Navidrome variant); BluOS "Qobuz Connect",
  "Unknown service", "Bluesound player", "Another BluOS maker".
- **Application:** "Claimed by Music Assistant", "Music Assistant
  unreachable", "Unclaimed service" with fake ports.
- **HTTP:** "Qobuz through Music Assistant" (fields), "Lossless", "Lossy",
  "Logo supplied", "No logo", "Logo added later" (a temporary folder),
  "Unknown brand requested", plus path safety.
- **Kiosk components:** "Logo supplied", "No logos", "Route unavailable",
  "Hi-Res track with the logo supplied", the device signature with and
  without a logo, the idle screen's text.
- **Screenshots (landscape and portrait):** fake scenarios with long
  badges, Navidrome, MA unavailable, and a test logos folder generated by
  the e2e setup (plain shapes, no real logos); "Too long for portrait"
  checks that no part wraps (each part's height equals one line) and that
  the portrait slots still hold with the footer.
- **Reference host:** the user's logos copied; Qobuz via MA, Spotify,
  Navidrome via MA, the optical input, MA unavailable, idle; portrait and
  landscape.
