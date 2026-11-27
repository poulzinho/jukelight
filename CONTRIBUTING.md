# Contributing

Thanks for your interest in Jukelight. Bug reports, ideas and pull requests
are welcome. Please read this first: the project has a few rules that keep
it safe to publish.

## Ask before you build

For anything bigger than a small fix, open an issue first and describe what
you'd like to change and why. Jukelight is tested on one reference setup
(see [docs/reference-setup.md](docs/reference-setup.md)); say if yours is
different.

## How changes are made

Every change follows the [OpenSpec](https://github.com/Fission-AI/OpenSpec)
workflow, in `openspec/`:

1. **Propose:** a change folder in `openspec/changes/` with the proposal
   (why), the delta specs (what the system must do, as WHEN/THEN scenarios),
   the design (how, with the alternatives considered) and the tasks.
2. **Apply:** work through the tasks, test first: each scenario becomes a
   failing test, then the code that makes it pass.
3. **Verify:** check the code against the specs, design and tasks.
4. **Archive:** merge the delta specs into `openspec/specs/` and move the
   change to `openspec/changes/archive/`.

Feedback during review becomes a task in the change first, then a fix.

## Before you open a pull request

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd web/kiosk && npm ci && npm run lint && npm run typecheck && npm test
```

For kiosk changes, also run the end-to-end screenshots (see the README) and
look at them in landscape and portrait.

Commits and pull request titles use conventional commits:
`type(scope): what the change does`, for example
`fix(kiosk): keep the title's letters inside its room`.

## Keep it private and legal

- **Example values only:** in code, tests, fixtures, docs, issues and pull
  requests, use `192.168.1.x` addresses, `aa:bb:cc:dd:ee:ff` MACs and
  names such as "Example Speaker". Never your own addresses, device or room
  names, or tokens.
- **Recordings:** record responses from real devices into a `fixtures/raw/`
  folder (git-ignored), replace every personal value, and commit only the
  scrubbed copy. The `no_personal_data` tests check this.
- **Secrets** go in `.env`, which is git-ignored. Never commit it, and never
  paste a token into an issue or a log.
- **No brand logos** in the repository: they are trademarks, supplied in
  `brand-assets/` by whoever runs Jukelight.
- **Dependencies and fonts** must allow distribution under MIT;
  `cargo deny check licenses` checks the Rust side.

By contributing, you agree that your contribution is licensed under the
project's [MIT licence](LICENSE).
