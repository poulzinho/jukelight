# artwork Specification

## Purpose

Delivers album artwork to the kiosk without exposing where it comes from:
the backend issues opaque artwork ids and serves the images itself.

## Requirements

### Requirement: Opaque artwork ids
Snapshots SHALL reference artwork only through ids issued by the backend,
exposed as `/api/v1/art/<id>`. No API response SHALL contain the address of
the player, a CDN or any other artwork source.

#### Scenario: Artwork in a snapshot
- **WHEN** the device reports artwork from `http://192.168.1.40:11000/Artwork?songid=1`
- **THEN** the snapshot contains an artwork URL of the form `/api/v1/art/<id>` and no part of the source address

#### Scenario: Same source, same id
- **WHEN** the same artwork source is reported twice
- **THEN** both snapshots contain the same artwork id

### Requirement: Artwork proxy
The artwork proxy SHALL serve the image for an id it issued, following
redirects of the source, with the source's content type. It SHALL answer
404 for an id it did not issue and 502 when the source fails.

#### Scenario: Issued id
- **WHEN** the kiosk requests `/api/v1/art/<id>` for an issued id whose source returns a JPEG
- **THEN** the response is 200 with content type `image/jpeg` and the image data

#### Scenario: Unknown id
- **WHEN** the kiosk requests `/api/v1/art/0000000000000000`, an id the backend never issued
- **THEN** the response is 404 and no outgoing request is made

#### Scenario: Redirecting source
- **WHEN** an issued id's source answers with a redirect to another image URL
- **THEN** the proxy returns the image from the redirect target

#### Scenario: Source fails
- **WHEN** an issued id's source cannot be reached
- **THEN** the response is 502

### Requirement: Artwork cache
The proxy SHALL keep recently served images in a bounded cache, so that
repeated requests for the same id do not contact the source again.

#### Scenario: Second request
- **WHEN** the same issued id is requested twice
- **THEN** the source is contacted only once

### Requirement: Large covers
For the track's cover, the artwork proxy SHALL serve the largest version a
known image CDN offers: after following redirects, a Qobuz image ending in
`_600.jpg` (or another size) SHALL be fetched as `_max.jpg`, and a Spotify
image (`i.scdn.co/image/ab67616d0000b273…`) with the size code `82c1`. When
the larger version can't be fetched, the proxy SHALL serve the original
image. Other sources SHALL be served as they are.

#### Scenario: Qobuz cover
- **WHEN** the source redirects to `https://static.qobuz.com/images/covers/ab/cd/abcdef_600.jpg` and the large cover is requested
- **THEN** the proxy serves the image from `https://static.qobuz.com/images/covers/ab/cd/abcdef_max.jpg`

#### Scenario: Spotify cover
- **WHEN** the source is `https://i.scdn.co/image/ab67616d0000b2731234567890abcdef12345678` and the large cover is requested
- **THEN** the proxy serves the image from `https://i.scdn.co/image/ab67616d000082c11234567890abcdef12345678`

#### Scenario: Larger version missing
- **WHEN** the large cover is requested and the larger version answers 404
- **THEN** the proxy serves the original image

#### Scenario: Unknown source
- **WHEN** the large cover is requested for a source that isn't a known image CDN
- **THEN** the proxy serves the original image

### Requirement: Backdrop image
The artwork proxy SHALL serve a backdrop version of a cover: the image
scaled down to at most 128 pixels on its longer side, blurred so that it
shows no edges or blocks when stretched over a 4K screen, with a faint
grain against banding, as PNG. When the image can't be decoded, the
request SHALL fail like an unreachable source.

#### Scenario: Backdrop of a cover
- **WHEN** the backdrop of a 600 by 600 pixel cover is requested
- **THEN** the proxy serves a PNG of at most 128 by 128 pixels

#### Scenario: Blurred
- **WHEN** the backdrop of a cover that is half black and half white, split down the middle, is requested
- **THEN** the backdrop changes gradually from dark to light across the middle, with no sharp edge

#### Scenario: Grain against banding
- **WHEN** the backdrop of a cover of one flat colour is requested
- **THEN** neighbouring pixels of the backdrop differ slightly, by a few levels at most

#### Scenario: Not an image
- **WHEN** the backdrop is requested for a source that isn't a raster image
- **THEN** the request fails like an unreachable source
