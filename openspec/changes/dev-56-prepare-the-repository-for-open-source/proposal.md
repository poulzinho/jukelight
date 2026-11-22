# DEV-56: Prepare the repository for open source

## Why

Jukelight is going public under the MIT licence. An audit on 22 November 2026
found the repository safe to publish: no personal data or secrets in the
files, the git history, the commit messages, or the pull requests and their
comments; licences compatible with MIT; no brand logos committed. A few
things still don't suit a public project: an outdated README, no trademark
note, Vite's logo as the kiosk's icon, and no guidance for reporting
vulnerabilities or contributing.

## What Changes

- Bring the README up to date: Music Assistant sessions, switching the
  screen with the display agent, and the reference setup are built, not
  planned.
- Add a trademark note: the products Jukelight works with belong to their
  owners, and Jukelight is not affiliated with them.
- Replace Vite's default favicon with Jukelight's own icon.
- Move Tailwind from the kiosk's dependencies to its dev dependencies: it
  only runs at build time.
- Add `SECURITY.md` (how to report a vulnerability) and `CONTRIBUTING.md`
  (the OpenSpec workflow, example values only, no personal data).
- Run the audit again on the final branch.
- After the merge, the owner switches the repository to public and turns on
  GitHub's protections (listed in the tasks). The commit history is kept as
  it is.

Non-goals:
- Rewriting the git history or starting a fresh repository (see design).
- Publishing a Docker image, and with it a third-party notices file.
- Changing any behaviour of the backend, the kiosk or the display agent.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
None: the change touches documentation, repository files, the kiosk's icon
and a dependency's category, not any requirement. The change sets
`skip_specs: true`.

## Impact

- `README.md`, new `SECURITY.md` and `CONTRIBUTING.md`.
- `web/kiosk/public/favicon.svg` (and `web/kiosk/index.html` if the icon's
  name changes).
- `web/kiosk/package.json` and `package-lock.json`: Tailwind under
  `devDependencies`; the built kiosk is unchanged.
- GitHub repository settings, changed by the owner after the merge.
- Roadmap: outside the four steps; it prepares the project for publishing
  and applies to all of them.
