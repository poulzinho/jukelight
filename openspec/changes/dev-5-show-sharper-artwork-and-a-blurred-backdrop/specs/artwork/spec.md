## ADDED Requirements

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
scaled down to at most 32 pixels on its longer side, as PNG. When the
image can't be decoded, the request SHALL fail like an unreachable source.

#### Scenario: Backdrop of a cover
- **WHEN** the backdrop of a 600 by 600 pixel cover is requested
- **THEN** the proxy serves a PNG of at most 32 by 32 pixels

#### Scenario: Not an image
- **WHEN** the backdrop is requested for a source that isn't a raster image
- **THEN** the request fails like an unreachable source
