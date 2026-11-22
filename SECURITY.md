# Security

## Reporting a vulnerability

Please report vulnerabilities privately, through GitHub: open the
repository's **Security** tab and choose **Report a vulnerability**. Don't
open a public issue or pull request for it.

Say what you found, how to reproduce it, and what an attacker could do with
it. You'll get an answer within two weeks. Jukelight is a project run in
spare time, so a fix may take longer; you'll hear how it's going.

## What's in scope

- The backend: the REST and SSE API, the artwork proxy, how settings and
  the Music Assistant token are handled.
- The kiosk served by the backend.
- The display agent and the host setup described in
  [docs/reference-setup.md](docs/reference-setup.md).

Out of scope: the players and services Jukelight reads from (BluOS, Music
Assistant, the music services), and setups that put Jukelight on the
internet. Jukelight is meant for a home network: its API has no login and
only reads.

## Supported versions

Only the latest commit on `main`.
