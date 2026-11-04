## 1. Idle agent (single pull request, no stack)

- [x] 1.1 Write the failing agent test "Quiet connection" (a counting `sleep` wrapper on `PATH`; `SILENCE_SECONDS=60`; at most two `sleep` calls in 5 s) and confirm it fails against the current agent, which shows the per-second wake-ups
- [x] 1.2 Make the watchdog sleep until the next possible deadline (stopping its sleep when it is stopped), and stamp once per non-empty line; the new test and all existing agent tests pass under bash and dash, including "Silent connection"
- [x] 1.3 On the reference host, note the agent's CPU time (`systemctl --user show jukelight-display-agent -p CPUUsageNSec`) over an hour, reinstall the agent, and measure again: under 10 s per hour, with the screen still following the music
- [x] 1.4 Review: once the watchdog's sleep has ended, forget its process id, so stopping the watchdog never kills a process that reused it; read the sleep count once in the "Quiet connection" test
- [x] 1.5 Verify: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `shellcheck`, kiosk lint, typecheck and tests, `openspec validate --strict`; open the pull request and verify CI is green; after approval, sync and archive before merging
