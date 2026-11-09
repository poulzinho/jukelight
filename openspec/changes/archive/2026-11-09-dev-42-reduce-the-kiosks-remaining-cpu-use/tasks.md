## 1. A stepped pulse (one pull request, DEV-42: kiosk)

- [x] 1.1 Write the failing e2e check "Stepped pulse" (the running pulse and port animations in the signal path use `steps(33)`), then give both theme animations `steps(33)`; verify with Playwright, together with the existing motion checks
- [x] 1.2 Verify: kiosk lint, typecheck and tests, end-to-end tests, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; open the pull request and verify CI is green
- [x] 1.3 Acceptance on the reference host: an hour of playback through Music Assistant, Chrome's CPU time against about 296 s per hour; the pulse judged from across the room; record the results on DEV-42 (about 225 s per hour, against about 296 s with the smooth pulse and 500 s before DEV-37; the stepped light looks great from a distance)
