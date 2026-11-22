## Context

The repository is private on GitHub and already carries an MIT `LICENSE`.
Since the first change it has kept to open-source rules: example values
only, secrets in `.env`, raw recordings in the git-ignored `fixtures/raw/`,
brand logos user-supplied in the git-ignored `brand-assets/`, and Figtree
(SIL OFL 1.1) instead of a font that may not be redistributed. The audit
checked what turns public with the repository:

```
  files today, git history, commit messages  -> clean
  pull requests, comments, review threads    -> clean
  CI logs and artifacts                       -> clean (example data only)
  Rust and npm licences                       -> compatible with MIT
  commit author email                         -> personal address (decision 1)
```

## Goals / Non-Goals

**Goals:**
- A repository a stranger can read, build and contribute to, with nothing
  private in it.
- Clear notes on trademarks, security reports and the contribution workflow.

**Non-Goals:**
- Rewriting history; publishing an image; any behaviour change.

## Decisions

### 1. Keep the history as it is

The author's personal email address is on most commits. Alternatives:
- **Rewrite the history** to GitHub's noreply address and force-push.
  Rejected: GitHub keeps every pull request's original commits
  (`refs/pull/<n>/head`), so the address stays reachable, while every hash
  changes.
- **Publish a fresh repository** from today's tree. Rejected: it loses the
  pull requests, their reviews and the history, which document how the
  specs came about.

The address is one the owner uses for development. New commits use the
noreply address, and GitHub blocks pushes that expose the personal one.

### 2. A trademark note in the README, not a separate file

The README names the products Jukelight works with (BluOS, Bluesound,
Qobuz, Spotify, Music Assistant, AirPlay, Hi-Res Audio) and says they
belong to their owners and that Jukelight isn't affiliated with them. A
`TRADEMARKS.md` was considered; for a handful of names a short section is
easier to find. The note also says why brand logos aren't in the
repository: they are supplied in `brand-assets/` by whoever runs
Jukelight.

### 3. Jukelight's own icon

Vite's default favicon is Vite's logo. Jukelight gets a simple SVG of its
own, drawn for this change and chosen by the owner from rendered drafts: a
1940s jukebox in flat shapes, its arch lit in the kiosk's quality colours
(teal over the top, purple and gold down the sides), a record and a grille
in the window, on two short feet. Drafts tried and set aside: a record with
a lit arc (read as a camera lens), bubble tubes under an arch (read as
headphones over an equaliser), and versions with glows and gradients
(muddy at 16 px). The shapes sit on a 64-unit grid, so at 16 px the tubes
are whole pixels. No third-party mark, and no new dependency.

### 4. Tailwind as a dev dependency

`tailwindcss` and `@tailwindcss/vite` compile the CSS at build time; with
them under `dependencies`, the licence summary counted `lightningcss`
(MPL-2.0) as shipped. As dev dependencies they say what they are. The
built kiosk is the same byte for byte; the Docker build installs dev
dependencies already (it runs the build).

### 5. Third-party notices only with a published image

People build Jukelight from source, so each dependency's licence comes
with its package. A notices file (the OFL text for Figtree, the MIT and
Apache notices) becomes necessary once a prebuilt image is published, and
belongs to that change.

### 6. The audit as a final task, not a CI job

The audit scripts read the local `.env` and the git-ignored recordings to
look for their values; CI has neither. The existing
`no_personal_data` tests keep guarding the committed fixtures in CI.

## Tests

- No new behaviour, so no new unit, adapter or contract tests.
- Kiosk: the existing component tests and the e2e screenshots in landscape
  and portrait still pass; the screenshots are unchanged except the tab icon.
- `cargo deny check licenses` and an npm licence summary of what ships.
- The audit again on the final branch.

## Risks / Trade-offs

- [The email stays public in the history] -> accepted by the owner
  (decision 1); future commits use the noreply address.
- [Switching to public can't be undone: forks and clones keep a copy] ->
  the owner switches only after the merge and the final audit.
- [Strangers' pull requests run CI] -> approval required for first-time
  contributors, and the workflow has no secrets and no
  `pull_request_target`.
