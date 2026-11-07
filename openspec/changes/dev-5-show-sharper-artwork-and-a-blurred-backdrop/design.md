## Context

- **Artwork proxy** (`crates/adapter-artwork`): issues an opaque id per
  source URL (`ArtworkSource::issue`), fetches the source on request
  (following redirects), caches up to 64 images and serves them at
  `/api/v1/art/{id}`. The DTO uses the same URL for the track's cover and
  for "Up next" thumbnails.
- **Sources:** BluOS reports `/Artwork?…` URLs per song that redirect to a
  CDN (for Qobuz `…_600.jpg`); Spotify covers are `i.scdn.co` URLs with a
  size code in the name. Fake scenarios use SVG data URLs.
- **Kiosk** (`web/kiosk/src/Kiosk.tsx`): the playing screen's `<main>` is
  keyed by `title|album`, so every track change remounts the whole screen,
  which fades in (600 ms) and reloads the cover. In portrait the grid is
  centred vertically (`content-center`), so every height change moves
  everything. Album colours set `--bg` and `--accent`.
- **Reference host:** Intel UHD 630 at 4K; the kiosk animates only opacity
  and transform.

## Goals / Non-Goals

**Goals:** sharp covers at 4K; a backdrop that picks up the cover; a track
change that only changes what changed and never shows a blank cover; a
portrait layout where nothing jumps; no extra GPU load worth measuring.

**Non-Goals:** Music Assistant image sizes; new animations; landscape
layout changes; content hashing of images.

## Decisions

### 1. Image sizes as a request parameter

`ArtworkSource::fetch(id, size)` with `ArtworkSize::{Original, Large,
Backdrop}`; the HTTP route reads `?size=large|backdrop` (no parameter:
original). The cache keys on id and size. The DTO gives the track's cover
as `…?size=large`, a new `backdropUrl` as `…?size=backdrop`, and queue
entries the original. Ids stay per source URL, so the existing id
requirements hold.

- Alternative: separate ids per size. Rejected: three issued entries per
  cover, and the URL no longer shows they are the same image.

### 2. Larger versions by URL rules, with a fallback

For `Large`, the proxy first tries the rule on the source URL itself (for
example a Spotify URL), so no original is fetched in vain; otherwise it
follows redirects to the final URL (for example from a BluOS `/Artwork`
URL), then applies the first matching rule: Qobuz `_<n>.jpg` → `_max.jpg`; Spotify
`ab67616d0000b273` → `ab67616d000082c1`. If the rewritten URL doesn't
answer with an image, it serves the original. The conventions are
undocumented, hence the fallback, and tests that pin them.

- Alternative: always fetch the original and upscale. Rejected: no sharper.

### 3. Backdrop: a 32 px PNG, stretched by the browser

For `Backdrop`, the proxy decodes the cover (the `image` crate, only JPEG,
PNG and WebP enabled), scales it to at most 32 px on the longer side and
encodes PNG. The kiosk shows it in a full-screen layer under a dark veil
(`bg-black/45`); the browser's smooth upscaling blurs it. Two stacked
layers cross-fade by opacity when the backdrop changes. Album colours stay
the base behind it and the accent of the progress bar.

- Alternative: CSS `filter: blur()` on the full cover. Rejected: expensive
  at 4K on the UHD 630 (DEV-5's original constraint).
- Alternative: the kiosk scales the image in a canvas. Rejected: more work
  in the browser for the same result.

### 4. The cover stays for the same album and artists

The kiosk keys the cover layer by album and artists (by the URL only when
there is no album), not by the title, so the next track on the same album
keeps the cover element and the backdrop. A new cover is decoded off-screen
(`HTMLImageElement.decode()`), then shown over the old one and faded in;
the old one goes once the fade has finished. Only `TrackInfo` is keyed by
the title, so only the text fades. "Up next" entries are keyed by title and
artists, not position.

- Alternative: name images by a hash of their bytes, so the same picture
  gets the same URL. Rejected for now: the id would only be known after
  fetching, which changes the proxy's contract; album and artists are
  enough for the case that matters (an album played through).

### 5. Portrait: fixed slots instead of centring

In portrait the screen becomes a column of fixed slots from top to bottom:
the cover at a fixed top margin, then slots with reserved heights for two
title lines, one artist line, one album line, the progress and one badge
row (no wrapping), and "Up next" pinned to the bottom for three entries.
A missing part leaves its slot empty. Sizes stay in `vmin`/`vh` as today,
so other portrait resolutions scale the same way.

- Alternative: keep centring but animate the movement. Rejected: still
  moves, and animating layout isn't allowed (opacity and transform only).
- Alternative: hide "Up next" space when there is no queue. Rejected: that
  is exactly the jump to avoid.

### 6. Fake covers become PNG

The fake scenarios' generated covers become small PNGs (drawn in Rust with
the same gradient and initials idea, without text if fonts are a problem),
so the backdrop works in screenshots. The data-URL path stays for other
callers.

## Risks / Trade-offs

- [A CDN changes its URL convention] → the fallback serves the original;
  the tests make the break visible.
- [Large covers are big (Qobuz `_max` up to 3000 px)] → the cache holds 64
  images; the browser scales them down. If memory becomes an issue, the
  cache capacity can drop.
- [Compilations with a different cover per track keep the first cover] →
  rare; the URL decides when there is no album. Accepted.
- [Empty space at the bottom in portrait without a queue] → intended: a
  still screen beats a jumping one.

## Tests

- **Domain/application:** the `ArtworkSize` in the port; the HTTP test
  double follows it.
- **Adapter fixtures** (`crates/adapter-artwork/tests/proxy.rs`, local test
  server): "Qobuz cover", "Spotify cover" (URL rules on the final URL),
  "Larger version missing", "Unknown source", "Backdrop of a cover", "Not
  an image".
- **HTTP** (`crates/adapter-http/tests/api.rs`): "Track with a cover",
  "Track without a cover"; `?size=` reaches the source.
- **Kiosk components:** "Track with a backdrop", "No backdrop", "Next track
  on the same album" (the cover element is the same node), "Track with a
  new cover" (the old cover stays until `decode()` resolves), "Quality
  changes".
- **Screenshots, landscape and portrait:** the backdrop in the existing
  scenarios; portrait positions compared between scenarios ("No queue":
  `playing-with-queue` against `cast-without-queue`; "Long title":
  `long-title` against `playing-with-queue`; "Missing album" with a new
  fake scenario if needed).
- **Reference host:** a Qobuz album played through (sharp cover, no blink
  between tracks), a Spotify cast, portrait without jumps.
