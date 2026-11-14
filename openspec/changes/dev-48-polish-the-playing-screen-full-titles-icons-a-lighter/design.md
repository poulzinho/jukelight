## Context

The playing screen and its signal diagram were finished in DEV-12/DEV-13
(archived 2026-11-14). The kiosk bundles Inter through
`@fontsource-variable/inter` and loads nothing from the internet. The title
is one `<h1>` clamped to two lines; in portrait it keeps room for two lines
so the layout never moves. The diagram is laid out in diagram units by
`diagram/layout.ts`, measuring text with a canvas in the page's font; the
streamer card has a thin lid and the maker's mark in its body's top-right
corner, where it gives way to a long format. The artwork proxy makes the
backdrop as a 32-pixel PNG thumbnail; the kiosk stretches it over the
screen behind a veil, deliberately without CSS filters (too costly at 4K on
the reference host's integrated GPU).

The rendered proposal (the real kiosk, both orientations' parts, two
reference cases: a long classical title with one speaker, a short title
with two speakers) was reviewed and approved on 14 November 2026. Its
draft code is the starting point; it is not committed.

## Goals / Non-Goals

**Goals:**
- Show long titles in full without moving the steady portrait layout.
- Make a classical queue readable in "Up next" without moving text.
- Make the diagram calmer and the streamer card less crowded, with the
  maker's mark always visible when supplied.
- Bundle a font whose licence allows redistribution.
- Remove the backdrop's blocks without adding work to the kiosk.

**Non-Goals:**
- Moving titles; landscape-only layout changes; user-chosen fonts.

## Decisions

### 1. The title steps through fixed sizes, measured in the browser

The title tries four steps, largest first: 7.2, 6.1 and 5.2 vmin on two
lines, then 4.6 vmin on three; it takes the first at which the text isn't
clamped (`scrollHeight` within a fifth of a line of `clientHeight`, since
letters overhang their line box). In portrait the title's box keeps the
height of two lines at 7.2 vmin (14.4 vmin at line height 1.0), and three
lines at 4.6 vmin fit in it, so nothing below moves. The step is chosen in
a layout effect before paint, again when the title changes, when the
viewport changes and once the font has loaded.

- *Alternative:* CSS `clamp()` or container-query font sizes. Rejected:
  they size by the box, not by the text, so a short title would shrink too.
- *Alternative:* a measured continuous size. Rejected: fixed steps keep
  the typographic scale and make tests deterministic.

### 2. Lid with mark and room; the mark no longer gives way

The lid grows from 8 to 19 diagram units and holds the mark (12 units,
icon only) and the room, set like a speaker's name (12.5 units, regular,
the main text colour), shortened with "…" to the lid: the streamer is as
important as the speakers. The body holds the format and the codec line.
The card grows from 56 to 58 units. The
mark leaves the body, so "the maker's mark gives way first" is dropped from
"Diagram fits": the format never competes with it any more.

### 3. Lighter lines, and more air between the rows

Lines go from 1.6 to 1.1 units at 0.6 opacity; ports from radius 4.2 to
3.3 with a 1.1 stroke at 0.75 opacity; junction dots from 2.6 to 1.9 at
0.75. The card's border goes to 1.3, staying stronger than the lines. The diagram's icons (speakers, computers, outputs) go from a 1.9 to a 1.4 stroke, as light as the text beside them and the maker's mark. The
gap between the last speaker row and the streamer grows from 14 to 24
units; the source's line shrinks from 52 to 48 (its label still clears the
logo).

Seen from across the room, the rows were too close. Speaker rows go from
26 to 31 units apart, the gap above the streamer to 24 and the output rows
from 22 to 25. The space comes from the page's top margin in portrait
(5vh to 3vh), which moves everything up and gives the diagram more room
above the footer; a two-speaker group with two outputs still fits in
portrait (checked in the rendered proposal; an e2e scenario guards it).

- *Alternative:* a smaller cover. Rejected: the cover is the screen's
  main element; the top margin had room to spare.

In portrait the column below the cover (title, progress, diagram, "Up
next") widens from 42vh to 47vh (at most 92vw), so the side margins shrink
from about 13% to 8% of the width and the page breathes; the cover keeps
its 42vh, so the vertical budget is unchanged. A long title fits on two
lines at a larger size, and "Up next" gains most of the width.

- *Alternative:* widen the cover with the column. Rejected: a taller cover
  takes back the room the diagram needs (two speakers would fold).

For gestalt, the title, artists and album stay with the cover: in portrait
they are inset by half the difference between the column and the cover
(`calc((min(92vw,47vh) - min(86vw,42vh)) / 2)` on each side), so their
edges line up with the cover's. The progress bar starts the wider band and
separates the track from what plays it (the diagram and "Up next"). The
title's size steps fit the inset width.

### 4. Figtree, bundled like Inter

`@fontsource-variable/figtree` replaces `@fontsource-variable/inter`: one
variable file (weights 300–900), so the diagram's 650 weight is exact.
The diagram's canvas measure reads the page's font; it is rebuilt once
`document.fonts` reports the font loaded, which lays the diagram out
again (today a remeasure happens only on resize).

- *Alternative:* Satoshi (preferred look). Rejected: the ITF Free Font
  License forbids distributing the files through a repository or image,
  and converting them. Plus Jakarta Sans, DM Sans, Manrope and Outfit were
  rendered too; Figtree was chosen.
- *Alternative:* Satoshi as a user-supplied font. Rejected for now: one
  bundled font keeps the layout tested and identical for everyone.

### 5. The backdrop is blurred once, in the artwork proxy

The proxy scales the cover to at most 128 pixels, applies a Gaussian blur
(`image::imageops::blur`, sigma about 6 at that size) on a slightly larger
canvas so the edges don't darken, then adds a grain of a few levels from a
fixed-seed pseudo-random sequence, so the same cover always yields the same
bytes (the artwork cache and tests rely on it). The kiosk is unchanged: one
stretched image behind the veil, no CSS filter, so its CPU use is the same;
the PNG grows from about 2 KB to about 30 KB, fetched once per album.

- *Alternative:* CSS `filter: blur()`. Rejected (CPU on the reference host).
- *Alternative:* a larger thumbnail without blur. Rejected: still blocky
  when stretched by 30×, and dark gradients band.

### 6. "Up next" groups entries in the kiosk, from the titles

Grouping is a pure function in the kiosk over the queue's entries and the
current track: split each title at its last ": " or " - "; a beginning is a
work only when a neighbour (or, for the first entry, the current track)
shares it, so a lone "Interlude: …" or "Song - Remastered 2011" stays
whole; then consecutive entries of one work, or else with the same main
(first) artist, form a group. The list renders headings and entries;
fitting in portrait counts a heading together with its first entry, so a
heading is never left alone above the footer. The API is unchanged: the
queue already carries titles and artists.

- *Alternative:* moving (scrolling) titles. Rejected for now: continuous
  movement costs CPU on the reference host and is harder to read from
  across the room; grouping shows the differing part at rest.
- *Alternative:* omit an entry's artist when it repeats the current
  track's. Rejected: the entry looked anonymous; a heading names it.
- *Alternative:* group in the backend. Rejected: grouping is presentation;
  the kiosk has the titles, and the rules are easier to test there.

### 7. The artist's icon: one person or a group, from the names

The API gives only the artists' names, so the icon follows two light
rules in the kiosk: two or more artists show the group icon (lucide
`Users`); one artist shows it too when the name contains, as a whole word
and in any case, a word for a group from a short list (orchestra,
philharmonic, ensemble, quartet, trio, choir, band, and their Spanish,
French, German and Italian forms); otherwise the one-person icon
(`UserRound`) is the default. The vocal microphone was replaced: at this
size it read as a pen.

- *Alternative:* the artist's type from MusicBrainz. Rejected: it would
  send the listening history to an outside service and add a lookup for
  one small icon.
- *Alternative:* the count only. Kept as the main rule; the word list only
  catches ensembles that are named as such, and a band without such a
  word keeps the person icon.

### Tests per layer

- **Artwork adapter (fixtures):** backdrop size, blur (the half-black,
  half-white scenario), grain (flat colour), determinism, not an image.
- **Kiosk unit (grouping function):** one test per "Up next" scenario.
- **Kiosk components:** title steps (short, long classical, too long),
  icons, the lid (mark and room, no mark, long room), lighter lines; the
  layout's two-speaker fit and the room's fitting in the layout tests; the
  remeasure after the font loads.
- **e2e (landscape and portrait, every fake scenario):** the existing
  overflow checks, a long classical title in full, two speakers without
  "+ N more", the steady portrait layout with a three-line title, Figtree
  served by Jukelight; screenshots reviewed against the approved proposal.
- **Reference host:** the two reference cases from across the room, the
  background on a red and a dark cover, and an hour's CPU against 64 s.

## Risks / Trade-offs

- [A title at 4.6 vmin is smaller than the artists line] → only titles
  that can't fit otherwise get there; the artists stay on one line below.
- [Figtree is narrower than Inter] → texts fit more, not less; the e2e
  overflow checks run in the new font.
- [The font loads after the first layout] → the diagram and the title are
  measured again once it has loaded; the first frame may use the fallback.
- [Bigger backdrop file] → about 30 KB per album, cached; negligible.

## Migration Plan

No API or configuration change. A rebuilt image brings the new font and
backdrop; cached backdrops are recreated after a restart. Rollback is a
revert.
