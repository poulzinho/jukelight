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

The title tries five steps, largest first: 7.2 and 6.2 vmin on one line,
4.8 and 4.2 vmin on two, then 3.2 vmin on three; it takes the first at
which the text isn't clamped (`scrollHeight` within a fifth of a line of
`clientHeight`, since letters overhang their line box). In portrait the
title's box is one line at 7.2 vmin high (11.2 vmin): a short title leaves
no empty line under it, and two lines at 4.8 or 4.2 and three at 3.2 fit
in the same box, so nothing below moves. Each line is 1.15 times the
size: Figtree's letters reach from 0.88 above the baseline to 0.21 below,
and the title clips what falls outside its lines, so the first steps'
tighter 1.02 cut off the tails of "g" and "y" (found in the acceptance). The step is chosen in a layout
effect before paint, again when the title changes, when the viewport
changes and once the font has loaded.

Artists may wrap to a second line (then "…"): two credits such as a
soloist and an orchestra are more useful in full than a steady album line.
That is the one place the portrait layout moves, and only for that track.

- *Alternative:* CSS `clamp()` or container-query font sizes. Rejected:
  they size by the box, not by the text, so a short title would shrink too.
- *Alternative:* a measured continuous size. Rejected: fixed steps keep
  the typographic scale and make tests deterministic.
- *Alternative:* always reserve two artist lines. Rejected: it costs a line
  on every track for the few that need it.

### 2. Lid with mark and room; the mark no longer gives way

The lid grows from 8 to 19 diagram units and holds the mark (12 units,
icon only) and the room, set like a speaker's name (12.5 units, regular,
the main text colour), shortened with "…" to the lid: the streamer is as
important as the speakers. The body holds the format and the codec line.
The card grows from 56 to 58 units. The
mark leaves the body, so "the maker's mark gives way first" is dropped from
"Diagram fits": the format never competes with it any more.

### 3. Lighter lines, more air, and the page's proportions

Lines go from 1.6 to 1.0 units at 0.6 opacity; ports from radius 4.2 to
3.3 with a 1.1 stroke at 0.75 opacity; junction dots from 2.6 to 1.9 at
0.75. The diagram's icons (speakers, computers, outputs) go from a 1.9 to
a 1.4 stroke on lucide's 24-unit grid (about 1.05 as drawn); the card's
border goes to 1.05, as heavy as the icons, and stays stronger than the
lines.

Seen from across the room, the rows were too close: speaker rows go from
26 to 32 units apart, the gap above the streamer from 14 to 26 and the
output rows from 22 to 26. The source's line shrinks from 52 to 34, since
what the source streams now sits on the logo's line (decision 8). The gap
between the progress bar and the diagram and "Up next" grows from 1.5 to
5 vmin, so the progress bar reads as a divider. Removing the footer
(decision 9) and a bottom margin of 4.5vh pay for it; a two-speaker group
with two outputs still fits in portrait (measured: about 267 units of
room; an e2e scenario guards it).

In portrait, for gestalt, everything below the cover (the title, artists,
album, progress bar, the diagram and "Up next") lines up with the cover's
edges, as before this change; the page keeps its side margins. A wider
column below the cover was tried (47vh): it gave "Up next" more width but
broke the single pair of edges, and was undone after review. "Up next"
relies on grouping for its titles instead.

- *Alternative:* a smaller cover. Rejected: the cover is the screen's main
  element.
- *Alternative:* a wider column for the diagram and "Up next". Rejected
  after rendering: two different edges under one cover looked unsettled.

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
(`image::imageops::blur`, sigma 6 at that size; it repeats the edge
pixels, so the edges don't darken, as the flat-colour test confirms), then
adds a grain (±2 levels, xorshift) of a few levels from a
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

### 8. What the source streams: a wave from the logo

The source's format sits on the logo's line, right-aligned with and set
like the speakers' formats, and a gentle wave (a series of quadratic
curves, amplitude 1.8, half-wavelength 9 units) joins the logo's right
edge to it, in the source's class colour. A logo too wide to leave at
least 40 units of wave is drawn smaller. The wave carries no spark: it
describes the stream rather than leading anywhere, so a direction would
be misleading. Instead, when a pulse starts, a bright copy of the wave and
the format fades in and out (opacity only, in steps), while the spark
leaves the logo down the line as before.

- *Alternative:* a "streams" row with an icon under the logo. Rejected
  after rendering: it repeated the speakers' rows without adding meaning.
- *Alternative:* a spark along the wave. Rejected: the stream has no
  destination there, and a curved path would need motion paths, which the
  compositor doesn't run.

### 9. No footer on the playing screen

The streamer card's lid now shows the maker's mark and the room, so the
device signature only repeated it. It stays on the idle and input
screens (an offline player keeps its playing screen, with its indicator); on the playing screen its space goes to the diagram and
"Up next", which now run to the page's bottom margin instead of the
footer's top.

### 10. A new session settles before it shows (DEV-54)

The session provider port gains `settling(device)`, false by default. The
Music Assistant provider is settling while it follows an AirPlay origin
without an answer since it started following it, and while it claims the
origin without content fetched for it; a failed poll ends settling (the
provider is unreachable, shown as before). The player monitor rebuilds as
usual, updating the current snapshot and sending events (the display
turns on at once), but while a provider is settling and less than 2
seconds have passed, it holds back the published snapshot, which the
kiosk follows, and sets a timer for the deadline; a provider's update or
the timer publishes the snapshot as soon as nothing is settling. (Found in
review: holding back the whole rebuild also delayed the events and the
current snapshot.) Track changes, pauses and positions of a settled session are not
delayed: settling only happens around a new origin.

- *Alternative:* the kiosk waits for a settled snapshot. Rejected: the
  kiosk can't tell a provider still answering from a session that really
  is AirPlay; the backend knows.
- *Alternative:* keep the last content until the provider has content.
  Rejected: it would show a stale track under a new origin.

### 11. Every paused group is held (DEV-55)

Recorded on the reference host (45 steps, a third of a second apart):
the player stops and returns to its own Qobuz queue first, while Music
Assistant's queue still reports the same item *playing* for about a
second, then idle. The hold of DEV-46 asked for a queue that isn't
playing, so on its first poll it let go, and when Music Assistant caught
up there was no group left to hold.

The hold now accepts the queue still playing the same item, for up to 5
seconds from the start of the hold (`Timing::pause_lag`); past that the
group is taken to play on without the player, and the hold ends as
before. A changed or cleared item still ends it at once.

- *Alternative:* wait for Music Assistant before deciding the hold.
  Rejected: holding at once keeps the paused screen from the first frame;
  letting go later costs nothing.
- The recording's states are the existing scrubbed fixtures' (a group
  playing, the same group paused), so the tests use those; the raw
  recording stays out of the repository.

### Stack layers

Five pull requests, bottom-up, each building and passing CI on its own:
1. **DEV-49, the background** (artwork adapter only).
2. **DEV-50, Figtree:** every later layer is measured and screenshotted in
   the final font.
3. **DEV-51, the track's text and the page's room:** the title, artists,
   icons, no footer, margins and the gap under the progress; it settles
   the vertical room the diagram depends on.
4. **DEV-52, the diagram:** lighter drawing, the streamer's lid, the rows'
   spacing, the wave and its flash; two speakers fit only with layer 3's
   room.
5. **DEV-53, "Up next" grouped:** fits to layer 3's bottom margin.
6. **DEV-54, settling:** application and Music Assistant adapter.
7. **DEV-55, holding every paused group:** Music Assistant adapter; the
   top layer carries the acceptance, the sync and the archive.

Each layer runs the e2e overflow checks for every fake scenario in both
orientations, so a layer that breaks the fit fails on its own.

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

- [A long title at 4.2 or 3.2 vmin is about the size of the artists
  line] → only titles that can't fit otherwise get there; they stay bold.
- [Artists on two lines move the rest of the screen] → only for that
  track; the diagram gives way by its usual rules if it no longer fits.
- [Without a footer, the player's name shows only in the streamer card] →
  the card's lid carries the mark and the room; the footer returns on
  every other screen.
- [Figtree is narrower than Inter] → texts fit more, not less; the e2e
  overflow checks run in the new font.
- [The font loads after the first layout] → the diagram and the title are
  measured again once it has loaded; the first frame may use the fallback.
- [Bigger backdrop file] → about 30 KB per album, cached; negligible.

## Migration Plan

No API or configuration change. A rebuilt image brings the new font and
backdrop; cached backdrops are recreated after a restart. Rollback is a
revert.
