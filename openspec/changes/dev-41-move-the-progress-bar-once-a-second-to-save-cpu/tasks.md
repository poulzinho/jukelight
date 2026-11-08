## 1. The progress bar once a second (one pull request, DEV-41: kiosk)

- [x] 1.1 Write failing component tests "Once a second" (fake timers: nothing changes before the next whole second, then the bar and the time together) and paused (no tick scheduled), then add `useSecondTicks` and use it in `Progress`; verify with the kiosk tests
- [x] 1.2 Write failing tests "No transition" (no transition class on the fill; e2e: the motion check expects `0s` in both projects and no animation runs in the progress), then remove the fill's transition; verify with the kiosk tests and Playwright
- [ ] 1.3 Verify: kiosk lint, typecheck and tests, end-to-end tests, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; open the pull request and verify CI is green
- [ ] 1.4 Acceptance on the reference host: an hour of playback, Chrome's CPU time against the 500 s per hour baseline; the bar's movement judged from across the room; record the results on DEV-41
