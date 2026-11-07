## 1. Reload after an update (single pull request, no stack)

- [x] 1.1 Write failing HTTP tests "Page carries the build" and "Stream announces the build", then compute the build identifier from the embedded `index.html`, insert the meta tag when serving the page, and send the `build` event after the first snapshot
- [x] 1.2 Write failing kiosk tests "New build", "Same build", "Still different after reloading" and "No build in the page", then listen for the `build` event in `usePlayer` and reload at most once per announced build
- [x] 1.3 `docs/reference-setup.md`: drop the manual kiosk restart after an update; keep it in Troubleshooting as a fallback
- [x] 1.4 Acceptance on the reference host: update Jukelight with a kiosk change and see the page reload by itself and show the new version, without `systemctl restart jukelight-kiosk`; an update without a kiosk change doesn't reload; record the results on DEV-26
- [x] 1.5 Review: explain in the code why the build identifier's empty fallback can't occur and is harmless; the fake `EventSource` passes a real `MessageEvent` to named-event listeners
- [ ] 1.6 Verify: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `shellcheck`, kiosk lint, typecheck and tests, `openspec validate --strict`; open the pull request and verify CI is green; after approval, sync and archive before merging
