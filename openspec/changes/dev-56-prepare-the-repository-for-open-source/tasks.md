## 1. The repository's public face (one pull request, DEV-56)

- [x] 1.1 Bring the README up to date: what Jukelight shows today (Music Assistant sessions, the signal diagram, "Up next"), switching the screen with the display agent, the configuration table checked against `.env.example`, and a link to the reference setup; example values only
- [x] 1.2 Add the trademark note to the README (decision 2): the products named belong to their owners, Jukelight isn't affiliated with them, and brand logos are supplied in `brand-assets/` by whoever runs Jukelight
- [x] 1.3 Draw Jukelight's own icon as SVG and show the owner a rendered draft in the browser tab and at full size; once approved, replace `web/kiosk/public/favicon.svg`; verify with the kiosk tests and the e2e screenshots in landscape and portrait
- [x] 1.4 Move `tailwindcss` and `@tailwindcss/vite` to `devDependencies` (decision 4); verify the built kiosk is unchanged (same files in `dist/`), the Docker build passes, and the licence summary of shipped packages lists only MIT, ISC, Apache-2.0, BSD and OFL-1.1
- [x] 1.5 Add `SECURITY.md` (report privately through GitHub's private vulnerability reporting; what's in scope: the backend, the kiosk, the display agent) and `CONTRIBUTING.md` (propose -> apply -> verify -> archive with OpenSpec, tests first, example values only, no personal data, recordings in `fixtures/raw/`, brand logos never committed)
- [x] 1.6 Run the audit again on this branch: tracked files and the whole history against the local `.env` and the patterns (private addresses, MACs, host names, tokens), commit messages, pull requests and their comments, CI artifacts, `cargo deny check licenses`, the npm licence summary; record the outcome on DEV-56 without any private value
- [x] 1.7 Verify: cargo fmt, clippy -D warnings, tests; kiosk lint, typecheck, unit and e2e tests; open the pull request and verify CI is green
- [x] 1.8 Found in CI: `cargo deny` reports RUSTSEC-2026-0592 (a stack overflow in `url` 2.5.8's `Url::origin()` on nested `blob:` URLs; Jukelight gets `url` through `reqwest` and `tower-http` and never calls `origin()` itself). Update `url` to 2.5.9 in `Cargo.lock`; verify `cargo deny check` passes and the Rust checks stay green

## 2. GitHub settings before the merge (the owner)

- [ ] 2.1 Set the commit email to GitHub's noreply address (`git config user.email`, on every machine that commits) and turn on "Block command line pushes that expose my email"
- [ ] 2.2 Turn on Dependabot alerts and private vulnerability reporting, and set Actions to require approval for workflows from first-time contributors

Switching the repository to public, and turning on secret scanning with push
protection (available once it is public), follow the merge; they are
recorded on DEV-56, not here.
