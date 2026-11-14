## MODIFIED Requirements

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
