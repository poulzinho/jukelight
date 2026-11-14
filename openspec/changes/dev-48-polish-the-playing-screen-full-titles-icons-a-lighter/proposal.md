# DEV-48: Polish the playing screen: full titles, icons, a lighter diagram, Figtree and a softer background

In one stack of five layers, each a sub-issue of DEV-48: DEV-49 (a softer
background), DEV-50 (Figtree), DEV-51 (the track's text and the page's
room), DEV-52 (the diagram) and DEV-53 ("Up next" grouped).

## Why

On the reference host the playing screen cuts long titles after two lines
(a classical title loses its key, catalogue number and movement), the
diagram's lines compete with its content, the streamer card is crowded,
and the background, a 32-pixel thumbnail stretched over a 4K screen, shows
blocky colour patches. A rendered proposal of the fixes, in the real kiosk,
was reviewed and approved. Part of roadmap step 4 (finished kiosk).

## What Changes

- **Long titles in full:** the title steps down in size until it fits the
  room it already has (two lines, then three smaller lines); "…" only when
  even the smallest size doesn't fit. Short titles keep the large size, and
  in portrait nothing below the title moves.
- **Icons before the artists and the album:** a person (or a group of
  people, for several artists or a name like "… Orchestra") and a disc,
  dimmed.
- **A lighter diagram:** thinner, partly transparent lines, with smaller
  ports and junction dots, so the nodes and formats lead.
- **A roomier streamer card:** a taller lid holds the maker's mark (the
  icon, `device.markUrl`) and the room; the body keeps the format and the
  codec line. The mark always shows when supplied, instead of giving way
  to a long format in the card's corner.
- **More space between the speaker rows and the streamer,** taken back
  from the source's line, so a group of two speakers still shows both rows
  in portrait.
- **"Up next" grouped:** consecutive movements of one work share the work
  as a heading and show only the movement; consecutive tracks by one artist
  share the artist as a heading; single entries stay as today. Titles wrap
  to two lines before "…". A classical queue becomes readable ("II.
  Andante", "III. Allegro assai") and no entry hides its artist without a
  heading naming it.
- **Figtree instead of Inter:** an open-source variable font (SIL Open Font
  License 1.1), bundled with the kiosk like Inter today; nothing is loaded
  from the internet. Satoshi was considered and rejected: its licence (ITF
  Free Font License) forbids redistributing it in a repository or an image.
- **The track's block aligned with the cover:** in portrait the title,
  artists, album and progress bar share the cover's edges; the diagram and
  "Up next" take a wider column below, with more space under the progress
  bar. Long titles step down through one, two and three lines in a
  smaller title area; artists wrap to a second line when needed.
- **What the source streams as a wave:** a gentle wave joins the source's
  logo to its format; it lights up when a pulse starts.
- **No footer on the playing screen:** the streamer card already shows the
  maker's mark and the room; the space goes to the diagram and "Up next".
- **A softer background:** the artwork proxy makes the backdrop at a larger
  size, blurs it and adds a faint grain against banding; the kiosk still
  stretches one image without CSS filters, so its CPU use doesn't change.

### Non-goals

- Moving (scrolling) text for titles that still don't fit (a possible
  follow-up; it would need a CPU measurement).
- Changes to the landscape layout beyond the shared parts above.
- A font chosen by the user (any bundled font must allow redistribution).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `kiosk-display`: the playing screen fits long titles and shows icons by
  the artists and the album; the streamer card's lid holds the mark and
  the room; "Up next" groups entries by work or artist; diagram lines are
  lighter; the mark no longer gives way in
  "Diagram fits"; the source's wave and its flash in "Signal diagram" and
  "Pulse along the diagram"; no footer on the playing screen in "Device
  signature"; a new requirement names the bundled typeface.
- `artwork`: the backdrop image is larger and blurred with grain instead
  of a 32-pixel thumbnail.

## Impact

- `web/kiosk`: `TrackInfo` (title fitting, icons), `UpNext` (grouping,
  two-line titles, fitting whole groups), `SignalDiagram` (lines,
  ports, streamer card), `diagram/layout.ts` (gap, source line, room in the
  lid), `index.css` and `package.json` (Figtree replaces
  `@fontsource-variable/inter`), the diagram's text measuring after the
  font has loaded; unit and e2e tests.
- `crates/adapter-artwork`: the backdrop's size, blur and grain; its tests.
- `crates/adapter-fake`: a scenario with a classical queue, for the e2e checks.
- `openspec/config.yaml` (project context names the bundled font), `README.md`
  if it mentions the font; `brand-assets/` stays user-supplied (the mark).
- No API change.
