## Context

The README is rendered by GitHub, in its dark or light theme. GitHub
allows a subset of HTML in Markdown (`<p align="center">`, `<img>`,
`<picture>`, `<a>`), which is how centred headers and badge rows are
usually built. The kiosk's icon (`web/kiosk/public/favicon.svg`) already
carries a dark and a light palette and switches with
`prefers-color-scheme`. The repository's rule is that no third-party logo
is committed or shown (DEV-56).

## Goals / Non-Goals

**Goals:**
- A header that looks right in GitHub's dark and light themes.
- Badges that tell the truth: live where GitHub reports the status,
  static only for facts that change rarely.

**Non-Goals:**
- Scorecard and release badges; repository settings; any product change.

## Decisions

### 1. The icon comes from the kiosk, in one file

The header shows `web/kiosk/public/favicon.svg` itself, so the README and
the kiosk can't drift apart. The SVG switches palettes with
`prefers-color-scheme`, which follows the viewer's system or browser
setting. Alternative: a `<picture>` with separate dark and light files and
GitHub's `#gh-dark-mode-only` / `media` rules, which follow GitHub's own
theme setting. Set aside: it means two more files to keep in step with the
icon, and most viewers run GitHub's theme in sync with their system.

### 2. Live badges for status, static badges for facts

CI and CodeQL use GitHub's own workflow badges, so they turn red when a
run fails. The others are static shields.io badges: "dependencies:
audited by cargo-deny" stays true while CI runs `cargo deny`; licence,
Rust version, React version and Docker change rarely and are updated by
hand with the toolchain or the kiosk (noted in a comment next to the
badges). Alternative: shields.io's dynamic badges reading `Cargo.toml` and
`package.json`. Set aside: they depend on file paths and ranges such as
`^19.3.1` and show them raw.

### 3. Jukelight's colours, no third-party logos

Static badges use the icon's dark palette: violet `#8b5cf6`, cyan
`#22d3ee`, pink `#f472b6`. shields.io can add the Rust, React or Docker
logos; they are left out under the repository's rule on brand logos.

## Tests

- No behaviour changes, so no unit, adapter or contract tests.
- Render the README as GitHub does (the GitHub Markdown API) in dark and
  light mode, and check every badge URL answers.
- The usual checks still pass in CI.

## Risks / Trade-offs

- [shields.io is unavailable] -> static badges show as broken images for
  a while; the README text is unaffected.
- [Static badges go stale] -> the comment next to the badges says when to
  update them; the Rust badge and `rust-toolchain.toml` change together.
