# DEV-19: Tidy the port handling in the image and the reference setup

## Why

On the reference host Jukelight runs on port 5853, but `docker compose ps`
also lists `8080/tcp`: the image declares a fixed `EXPOSE 8080` whatever
`HTTP_PORT` says. That misleads anyone checking which port Jukelight uses.
A review of the DEV-4 pull request (after the change was archived) also
found awkward wording about the port in `docs/reference-setup.md`. A chore
outside the roadmap steps (1 walking skeleton, 2 Music Assistant session,
3 display power, 4 finished kiosk and admin): it tidies what step 3 left.

## What Changes

- The image no longer declares a fixed port: `EXPOSE 8080` is removed. The
  port is published by `ports:` in `docker-compose.yml` from `HTTP_PORT`,
  as before; the health check keeps using `HTTP_PORT`.
- CI checks that the built image declares no port.
- `docs/reference-setup.md`: "If you use another port than 8080" becomes
  "If you use a port other than 8080".

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None: no requirement changes. Jukelight listens on `HTTP_PORT` (default
8080) and the compose file publishes that port, as before. The change sets
`skip_specs: true`.

## Non-goals

- Changing the default port (8080).
- Other changes to the image or the compose file.

## Impact

- `Dockerfile`, `.github/workflows/ci.yml` (Docker job),
  `docs/reference-setup.md`.
- Deployments: after the next `docker compose up --build`, `docker compose
  ps` shows only the published port. Nothing else changes.
