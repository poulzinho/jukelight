## 1. Agent restart (single pull request, no stack)

- [x] 1.1 Write failing agent tests for "Stale sway session" (a fake `swaymsg` that always fails; three decisions; the agent exits with status 1) and "Occasional failure" (one failure, then success; the agent keeps running), then count failed switches in the agent and stop after 3 in a row
- [x] 1.2 Change the sway configuration line in `docs/reference-setup.md` from `start` to `restart`, and add a troubleshooting entry for `swaymsg failed to switch the display`
- [x] 1.3 Check on the reference host: reinstall the agent, change the sway line, `sudo systemctl restart jukelight-kiosk`, and see in the agent's log that it switches again without a manual restart
- [x] 1.4 Verify: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `shellcheck`, kiosk lint, typecheck and tests, `openspec validate --strict`; open the pull request and verify CI is green; after approval, sync and archive before merging
