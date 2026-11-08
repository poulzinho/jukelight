## 1. The source's brand (stack layer 1, DEV-33: domain, adapters, API)

- [x] 1.1 Domain: `Brand` (identifier alphabet), `Service { brand, name }` on the content, `Source { brand, name, via: Option<Route> }` on the snapshot, the device's maker; unit tests
- [x] 1.2 Write failing MA fixture tests "Renamed Qobuz provider", "Navidrome", "Another Subsonic server", "Local files" and "Unknown domain" (scrubbed fixture variants), then recognise the brand from the provider's domain
- [x] 1.3 Write failing BluOS fixture tests "Qobuz Connect", "Unknown service", "Bluesound player" and "Another BluOS maker", then map service ids to brands and read the maker from `/SyncStatus`
- [x] 1.4 Write failing application tests "Claimed by Music Assistant", "Music Assistant unreachable" and "Unclaimed service", then build the snapshot's source in arbitration
- [x] 1.5 Write failing HTTP tests "Qobuz through Music Assistant", "Lossless" and "Lossy", then publish `source`, `device` and the quality details (logo URLs still `null`); kiosk API types
- [x] 1.6 Verify the layer: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, kiosk lint, typecheck and tests; open the pull request and verify CI is green
- [x] 1.7 Review of layer 1: document that `brand` may be `null` in the kiosk's types; MA's `domain` comment (the brand is derived from it); ASCII lowercasing for "Navidrome"; `FakeProvider::with_brand` fails in debug builds on an invalid brand

## 2. User-supplied logos (stack layer 2, DEV-34: backend, compose, docs)

- [x] 2.1 Write failing HTTP tests "Logo supplied", "No logo", "Logo added later", "Unknown brand requested" and path safety (a temporary folder), then serve logos from `BRAND_ASSETS_DIR` with a short cache and publish logo URLs (with `?v=`) only when present, including `quality.hiResLogoUrl`
- [x] 2.2 `BRAND_ASSETS_DIR` in the configuration; the read-only mount in `docker-compose.yml`; `brand-assets/` in `.gitignore`; `.env.example` and `README.md`
- [x] 2.3 `docs/reference-setup.md`: the folder, file names and variants, the official source per brand, the Hi-Res logo, copying the folder to the host
- [ ] 2.4 Verify the layer (as 1.6); open the pull request and verify CI is green
- [x] 2.5 Review of layer 2: a logo is cacheable for good only when the request names its current version (`?v=`), otherwise `no-cache`; the artwork route's doc comment back in its place

## 3. The new badges (stack layer 3, DEV-35: kiosk)

- [x] 3.1 Write failing component tests "Logo supplied", "No logos", "Route unavailable", then the source badge with Lucide fallback icons (`lucide-react`) and per-brand logo heights
- [x] 3.2 Write failing component tests "Hi-Res track" and "Hi-Res track with the logo supplied", then the Hi-Res mark (tag or logo) and the quality as text from the new fields, with tabular figures
- [x] 3.3 Write failing screenshot checks "Everything fits" and "Too long for portrait" (no part wraps; detail first, then "via"), with fake scenarios (long badges, Navidrome, MA unavailable) and a generated test logos folder in the e2e setup; then the one-line row and its priorities
- [ ] 3.4 Review the screenshots in landscape and portrait; verify the layer (as 1.6); open the pull request and verify CI is green
- [x] 3.5 Review of layer 3: an explicit key for the badge line instead of `JSON.stringify`; the detail and "via" come back when the line gets wider; the e2e setup removes its test-logo folder and creates the empty one

## 4. Device signature and acceptance (stack layer 4, DEV-36: kiosk, reference host)

- [ ] 4.1 Write failing component tests for the device signature (with and without a logo) and the idle screen's "nothing playing", then the footer on every screen and its slot in the steady portrait layout; screenshot checks that the portrait slots still hold
- [ ] 4.2 Acceptance on the reference host: copy the user's `brand-assets/` (official logos where available); Qobuz via MA, Spotify, Navidrome via MA (provider renamed), the optical input, MA unavailable, idle; portrait and landscape; tune the per-brand logo heights; record the results on DEV-32
- [ ] 4.3 Verify: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `shellcheck`, kiosk lint, typecheck and tests, `openspec validate --strict`; open the pull request and verify CI is green; after approval, sync and archive on this top layer before merging
