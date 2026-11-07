## 1. Larger covers and the backdrop image (stack layer 1, DEV-30: backend)

- [x] 1.1 Add `ArtworkSize` (`Original`, `Large`, `Backdrop`) to the `ArtworkSource` port and the HTTP route (`?size=`), with the cache keyed by id and size; existing artwork tests keep passing
- [x] 1.2 Write failing fixture tests "Qobuz cover", "Spotify cover", "Larger version missing" and "Unknown source", then add the URL rules on the final URL with the fallback
- [x] 1.3 Write failing fixture tests "Backdrop of a cover" and "Not an image", then add the backdrop version (the `image` crate with JPEG, PNG and WebP only; `cargo deny` passes)
- [x] 1.4 Write failing HTTP tests "Track with a cover" and "Track without a cover", then give the track's cover `?size=large` and add `backdropUrl`; queue entries keep the original
- [x] 1.5 Make the fake scenarios' covers PNG, so the backdrop works in screenshots
- [ ] 1.6 Verify the layer: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, kiosk lint, typecheck and tests; open the pull request and verify CI is green

## 2. Backdrop, steady updates and portrait layout (stack layer 2, DEV-31: kiosk and acceptance)

- [ ] 2.1 Write failing component tests "Track with a backdrop" and "No backdrop", then add the backdrop layer with the veil and the cross-fade (opacity only, no CSS blur)
- [ ] 2.2 Write failing component tests "Next track on the same album", "Track with a new cover" and "Quality changes", then key the cover by album and artists, decode a new cover before cross-fading it, key only the track text by the title, and key "Up next" entries by title and artists
- [ ] 2.3 Write failing screenshot checks for "No queue", "Long title" and "Missing album" in portrait (positions compared between scenarios), then replace the portrait centring with fixed slots; landscape unchanged
- [ ] 2.4 Review the screenshots in landscape and portrait
- [ ] 2.5 Acceptance on the reference host: a Qobuz album played through (sharp cover, no blink between tracks of the same album), a Spotify cast (large cover), a track change to another album (no blank cover), portrait without jumps; GPU load unchanged (`chrome://gpu` still hardware accelerated, animations smooth); record the results on DEV-5
- [ ] 2.6 Verify: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `shellcheck`, kiosk lint, typecheck and tests, `openspec validate --strict`; open the pull request and verify CI is green; after approval, sync and archive on this top layer before merging
