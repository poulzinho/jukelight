## 1. A pulse on a change (one pull request, DEV-43: kiosk)

- [x] 1.1 Write failing component tests "Playing", "Nothing changes" (fake timers, a minute), "Quality changes" and "Playback resumes", then key the pulse by the track, source, route and quality instead of a timer, and remove `useBeat`; verify with the kiosk tests
- [x] 1.2 Write the failing e2e check "Between pulses" (once the first pulse has run, no animation in the signal path for 10 s) and keep the existing motion checks working on the first pulse; verify with Playwright
- [ ] 1.3 Verify: kiosk lint, typecheck and tests, end-to-end tests, `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; open the pull request and verify CI is green
- [ ] 1.4 Acceptance on the reference host: an hour of playback through Music Assistant against about 225 s per hour; a new track, a pause and resume, and the screen judged from across the room; record the results on DEV-43
- [x] 1.5 Review: a test that a new source or a route coming back pulses again; the pulse's key is built with `JSON.stringify`, so a `|` in a title or an artist can't make two tracks look the same (test)
