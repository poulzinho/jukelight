# DEV-5: Show sharper artwork and a blurred backdrop

## Why

The kiosk runs on a 4K screen, but covers arrive at the size the player
reports (Qobuz covers at 600 px), so they look soft. The background is a
flat album colour and doesn't pick up the cover's mood. Two things also
make the screen restless:

- **Every track change rebuilds the whole playing screen.** The screen is
  keyed by the title, so the cover, the text, the badges and "Up next" are
  all thrown away and fade in again, and the cover goes blank until it has
  loaded, even for the next track of the same album (BluOS gives each song
  its own image URL).
- **The portrait layout jumps.** Its content is centred vertically, so a
  title on one or two lines, a missing album or a missing queue moves the
  cover and everything else up or down.

Part of roadmap step 4 (finished kiosk).

## What Changes

- **Larger covers:** the artwork proxy serves the largest version a known
  image CDN offers (Qobuz, Spotify), after following redirects, and falls
  back to the original image when that fails. "Up next" thumbnails keep
  the original size.
- **Blurred backdrop, cheaply:** the proxy also serves a tiny version of
  the cover (at most 32 px), which the kiosk stretches over the whole
  screen behind a dark veil. Stretching blurs it; there is no CSS blur,
  which is expensive at 4K on the reference host's integrated GPU.
- **Only what changes updates:** on a track change, the title and artists
  fade; the cover and the backdrop stay when the album is the same; a new
  cover is loaded first and then cross-fades over the old one; badges and
  "Up next" update in place.
- **A steady portrait layout:** in portrait, every part keeps a fixed place
  (the cover at the top, room for two title lines, one artist line, one
  album line, the progress, one row of badges, "Up next" at the bottom), so
  changing content never moves the other parts. Landscape stays as it is.
- **Fake scenarios** get raster covers (PNG), so screenshots show the
  backdrop.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `artwork`: larger versions of covers with a fallback, and a tiny backdrop
  version.
- `published-api`: the snapshot names a backdrop image; the track's cover
  URL asks for the large version.
- `kiosk-display`: the backdrop; updating only what changed on a track
  change; a steady portrait layout.

## Non-goals

- Music Assistant artwork sizes (MA serves its own images; its own ticket).
- Animations beyond the fades and the cross-fade (DEV-7).
- Changing the landscape layout.
- Recognising the same cover by its bytes; the kiosk compares album and
  artists instead (see design).

## Impact

- `crates/adapter-artwork` (larger versions, the backdrop version, a new
  image dependency for resizing), the `ArtworkSource` port in
  `crates/application`, `crates/adapter-http` (routes, DTO),
  `crates/adapter-fake` (PNG covers).
- `web/kiosk` (backdrop, playing screen, portrait layout) and the
  screenshot tests.
- API: a new `backdropUrl` field; existing fields unchanged.
- Two stacked pull requests: backend, then kiosk with the acceptance on the
  reference host.
