## 1. The README's header and badges (one pull request, DEV-60)

- [x] 1.1 Add the centred header to `README.md`: the icon from `web/kiosk/public/favicon.svg` (decision 1), the name, and the tagline "A now-playing kiosk for BluOS players and Music Assistant: artwork, the signal path and what's up next, on a wall-mounted screen."
- [x] 1.2 Add the badge row (decision 2): CI and CodeQL live from GitHub; "dependencies: audited by cargo-deny", licence MIT, Rust 1.93, kiosk React 19 and runs in Docker static, in Jukelight's colours with no third-party logos (decision 3), each linking to its source (the workflow runs, `deny.toml`, `LICENSE`, `rust-toolchain.toml`, `web/kiosk`, `Dockerfile`); add a comment saying which badges to update by hand and when
- [x] 1.3 Name the pinned toolchain in the development section: "Rust 1.93 (pinned in `rust-toolchain.toml`)" instead of "Rust (stable)"
- [x] 1.4 Render the README with the GitHub Markdown API in dark and light mode, check every badge and link URL answers, and show the owner the renders
- [ ] 1.5 Verify: CI green on the pull request; open the pull request
