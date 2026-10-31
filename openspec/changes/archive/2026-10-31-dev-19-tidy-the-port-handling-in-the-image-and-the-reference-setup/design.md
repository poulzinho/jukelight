## Context

`EXPOSE` in a Dockerfile documents a port; it does not publish it. The
compose file publishes `${HTTP_PORT:-8080}:${HTTP_PORT:-8080}`, and the
container listens on `HTTP_PORT` (image default `ENV HTTP_PORT=8080`). With
`HTTP_PORT=5853`, `docker compose ps` shows `0.0.0.0:5853->5853/tcp,
8080/tcp`: the second entry comes only from the fixed `EXPOSE`.

## Goals / Non-Goals

**Goals:** the image and `docker compose ps` show only the port Jukelight
really uses.

**Non-Goals:** a different default port; other image changes.

## Decisions

### 1. Remove `EXPOSE`

The image declares no port. `docker compose ps` then lists only the
published port.

- Alternative: a build argument (`ARG HTTP_PORT`, `EXPOSE ${HTTP_PORT}`)
  passed from the compose file. Rejected: the declared port is fixed at
  build time, so changing `HTTP_PORT` and running `docker compose up -d`
  without `--build` would show a stale port again.
- Alternative: keep `EXPOSE 8080` as documentation of the default.
  Rejected: it is wrong for every other port, and the default is already
  documented in the README and `.env.example`.

### 2. Checked in CI

The Docker job loads the built image and fails if
`docker image inspect` reports exposed ports. The check fails on today's
image before the Dockerfile changes (test-first).

## Risks / Trade-offs

- [Someone runs the image with `docker run -P`, which publishes declared
  ports] → that no longer publishes anything; the README and the compose
  file publish the port explicitly, which is the documented way.
