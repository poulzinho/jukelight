## 1. Port handling (single pull request, no stack)

- [x] 1.1 Add a CI step to the Docker job that loads the built image and fails when `docker image inspect` reports exposed ports; check it fails against today's image (locally with `docker build` and the same inspect command)
- [x] 1.2 Remove `EXPOSE 8080` from the Dockerfile until the check passes; check locally that `docker compose ps` with `HTTP_PORT=5853` lists only `5853` and that the container still becomes healthy
- [x] 1.3 Change "If you use another port than 8080" to "If you use a port other than 8080" in `docs/reference-setup.md`
- [x] 1.4 Verify: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, kiosk lint, typecheck and tests, `openspec validate --strict`; open the pull request and verify CI is green
