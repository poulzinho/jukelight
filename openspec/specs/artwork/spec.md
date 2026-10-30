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
