# DEV-60: Brand the README: header, tagline and status badges

## Why

Jukelight is public now (DEV-56), and its README starts with a plain
title. A header with the project's icon, a one-line tagline and status
badges makes the project recognisable and shows at a glance that it is
built, tested and checked for security.

## What Changes

- A centred header at the top of `README.md`: the neon jukebox icon, the
  name and a tagline that matches the repository description (DEV-57).
- One row of status badges under the tagline:
  - live: CI (`.github/workflows/ci.yml`) and CodeQL (GitHub code
    scanning);
  - static: "dependencies: audited by cargo-deny", licence MIT, Rust 1.93,
    kiosk React 19, runs in Docker.
- The development section names the pinned toolchain (Rust 1.93, from
  `rust-toolchain.toml`) instead of "Rust (stable)", so it agrees with the
  badge.

Non-goals:
- An OpenSSF Scorecard badge (it needs a workflow) and a release badge
  (no release yet).
- The repository's description and topics (DEV-57, a setting) and its
  social preview image (DEV-58).
- Any change to the kiosk, the backend or the icon itself.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None: the change touches the README only. The change sets
`skip_specs: true`.

## Impact

- `README.md` only. The icon is referenced from
  `web/kiosk/public/favicon.svg`; no file is added.
- Static badges are images from shields.io, loaded by whoever views the
  README on GitHub; nothing is downloaded at build or run time.
- Roadmap: outside the four steps; it follows the publishing (DEV-56).
