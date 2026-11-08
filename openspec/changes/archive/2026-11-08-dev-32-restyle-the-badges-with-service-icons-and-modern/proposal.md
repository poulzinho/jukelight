# DEV-32: Restyle the badges with service icons and modern typography

Together with DEV-33 (recognise the music service behind Music Assistant),
the bottom layer of the same stack.

## Why

With real data the kiosk's badges wrap inside themselves in portrait
("Music Assistant · Qobuz", "HI-RES" and "FLAC 24/96 · Normalised" each
break onto two lines, found during the DEV-5 acceptance), and they look
dated: three similar pills with no hierarchy, and no way to recognise the
service from across the room. Brand logos would fix that, but the project
is open source (MIT) and must not distribute trademarked artwork, so logos
can only be supplied by whoever runs Jukelight. The badges also need to
know the music service's **brand**, not just its display name: today the
service behind Music Assistant (MA) is only a renameable name, and a
Subsonic server (such as Navidrome) isn't told apart. Part of roadmap step
4 (finished kiosk).

## What Changes

- **The source's brand (DEV-33):** the backend recognises the brand of the
  music service: from MA's provider `domain` (Navidrome when the
  `opensubsonic` provider's name contains "Navidrome", otherwise Subsonic;
  local files as Library), and from BluOS's service id. It also recognises
  the player's maker (BluOS `brand`, for example Bluesound).
- **The API** publishes a `source` (brand, name, and the route: "via Music
  Assistant", available or not) and a `device` (brand and room name), next
  to today's `session` label, which stays.
- **User-supplied logos:** a git-ignored `brand-assets/` folder next to
  `.env`, mounted read-only into the container. The backend detects which
  logos exist (also files added later, without a restart), serves them at
  `/api/v1/logos/{brand}` and publishes a logo URL only for brands that
  have one. The Hi-Res Audio logo is supplied the same way.
- **New badges:** one dark badge for the source: the brand's logo, or an
  icon and the name in Inter (the fallback, shipped in the repo); "via" and
  the MA logo inside the badge; a greyed, outlined badge when MA is
  unavailable. After it, on one line: the HI-RES tag (or the Hi-Res logo),
  the quality as text ("24 bit / 96 kHz FLAC", tabular figures) and the
  dimmer detail. When space runs out, the detail gives way first, then
  "via MA" as a whole; the source, Hi-Res and the quality never do.
- **Device signature:** the player's logo (or nothing) and the room name in
  a quiet footer on every screen, in the bottom-right corner in landscape.
  The idle screen's "<player name> · nothing playing" becomes "nothing
  playing" plus the signature.
- **Reference setup:** the `brand-assets/` folder, where to get each
  official logo and which variant to use; copying it to the host.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-music-assistant`: the music provider's brand from its domain,
  including Navidrome, Subsonic and Library.
- `device-bluos`: the service's brand from BluOS's service id, and the
  player's maker.
- `session-arbitration`: the snapshot's source (brand, name, route) for
  claimed and unclaimed sessions.
- `published-api`: `source` and `device` in the snapshot; logos served
  from the user's folder, with URLs only when present.
- `kiosk-display`: the source badge, the one-line quality, the Hi-Res logo
  when supplied, the device signature, and the idle screen's text.

## Non-goals

- Shipping any brand logo, the Hi-Res Audio logo, or brand fonts in the
  repo.
- Uploading logos in the browser (DEV-6, the admin app, will use the same
  folder and detection).
- Speaker groups and which speakers play (DEV-12, DEV-13).
- Asking a Subsonic server for its type (it would need its address and
  credentials).

## Impact

- `crates/domain` (the source's brand, the device's maker),
  `crates/adapter-music-assistant`, `crates/adapter-bluos`,
  `crates/application` (arbitration), `crates/adapter-http` (DTO, logo
  route), `crates/app` (the folder setting).
- `web/kiosk` (badges, footer, idle screen), with Lucide icons (ISC) as a
  new dependency, and the screenshot tests.
- `docker-compose.yml` (the read-only mount), `.gitignore`,
  `.env.example`, `README.md`, `docs/reference-setup.md`.
- API: new fields `source` and `device`; existing fields unchanged.
- A stack of four pull requests: DEV-33 (brands), then three sub-issues of
  DEV-32 (logos, badges, signature and acceptance).
