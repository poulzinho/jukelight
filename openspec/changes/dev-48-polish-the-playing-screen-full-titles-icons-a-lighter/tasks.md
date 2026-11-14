## 1. A softer background (stack layer 1, DEV-49: artwork adapter)

- [x] 1.1 Write failing artwork tests "Backdrop of a cover" (at most 128 px), "Blurred" (half black, half white: a gradual change, no sharp edge), "Grain against banding" (flat colour: neighbours differ by a few levels at most) and the same cover giving the same bytes; then scale to 128 px, blur and add a fixed-seed grain; verify with `cargo test -p jukelight-adapter-artwork`
- [x] 1.2 Verify the layer: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, kiosk lint, typecheck, tests and e2e; open the pull request and verify CI is green

## 2. Figtree (stack layer 2, DEV-50: kiosk)

- [x] 2.1 Write failing kiosk tests "Bundled font" (Figtree, served by Jukelight) and "Measured after loading" (the diagram is laid out again once the font has loaded); then replace `@fontsource-variable/inter` with `@fontsource-variable/figtree` and rebuild the diagram's measure when the font has loaded; verify with the kiosk tests
- [x] 2.2 Update the project context (`openspec/config.yaml`: the bundled font) and `README.md` where it names the font
- [x] 2.3 Verify the layer (as 1.2), including the e2e overflow checks in the new font and a screenshot review; open the pull request and verify CI is green

## 3. The track's text and the page's room (stack layer 3, DEV-51: kiosk)

- [x] 3.1 Write failing component tests "Short title", "Medium title", "Long classical title" and "Title too long for any size"; then the stepped title (7.2 and 6.2 vmin on one line, 5.4 and 4.6 on two, 3.6 on three, in an 11.2 vmin box); verify with the kiosk tests
- [x] 3.2 Write failing component tests "Two artists on two lines", "Artists and album with icons", "Several artists", "A group's name" and "A name without a group word"; then artists wrapping to two lines and the icons (one person by default, a group for several artists or a group word); verify with the kiosk tests
- [x] 3.3 Write failing tests for "Device signature" (no footer on the playing screen; the footer on the idle and input screens) and the steady portrait layout ("Title stepped down to two lines" and the artists' exception); then remove the footer from the playing screen, set the margins (5vh top, 4.5vh bottom) and 5vmin under the progress, and let the diagram and "Up next" run to the bottom margin; verify with the kiosk tests
- [x] 3.4 Write failing e2e checks: a long classical title in full, "Aligned with the cover in portrait" (the edges of the title, progress, diagram and "Up next" against the cover), no footer on the playing screen, overflow checks against the bottom margin for every fake scenario in both orientations; review the screenshots
- [x] 3.5 Verify the layer (as 1.2); open the pull request and verify CI is green
- [x] 3.6 Found in the acceptance (7.4): the title clipped the tails of its letters ("g"). Write a failing e2e check "Letters with tails and accents" (each line holds the letters' ink, from the font's own metrics, for a short and a long title in both orientations); then lines 1.15 times the size and the steps 7.2 and 6.2 on one line, 4.8 and 4.2 on two, 3.2 on three, still in the 11.2 vmin box; verify with the kiosk tests and the e2e checks
- [x] 3.7 Found in the review: the title didn't fit itself again when the viewport changed (design decision 1), and the component tests left their stand-in heights in place. Write the failing component test "a new viewport: the title fits again from the largest size"; then refit on a resize and remove the stand-ins after each test; verify with the kiosk tests

## 4. The diagram (stack layer 4, DEV-52: kiosk)

- [x] 4.1 Write failing component and layout tests for the "Streamer card" scenarios (mark and room in the lid, the room set like a speaker, no mark), "Long room name" and "Lines stay in the background"; then the taller lid (19, card 58) and the lighter lines (1.0), ports, dots, icons (stroke 1.4) and border (1.05); verify with the kiosk tests
- [x] 4.2 Write failing layout tests "What the source streams, as a wave", "A wide logo" and "Two speakers in portrait"; then the wave from the logo to the format (at least 40 units), the speaker rows (32), the gap above the streamer (26), the output rows (26) and the source's line (34); verify with the kiosk tests
- [x] 4.3 Write failing tests for "Pulse along the diagram" (the wave and its format light up as the pulse starts; nothing lights with reduced motion), then the flash; verify with the kiosk tests and the e2e motion checks
- [x] 4.4 Write the failing e2e check "two speakers without + N more in portrait", keep the overflow checks; review the screenshots against the approved renders
- [x] 4.5 Verify the layer (as 1.2); open the pull request and verify CI is green

## 5. "Up next" grouped (stack layer 5, DEV-53: kiosk, fake device)

- [x] 5.1 Write failing unit tests for the grouping function, one per "Up next" scenario (movements of one work, performer changes, no composer, dash separator, lone title, album, featured artist, artist interrupted, no artist), then the function; verify with the kiosk tests
- [x] 5.2 Write failing component tests "Long title" (two lines, then "…") and "Heading at the bottom", then the grouped list; verify with the kiosk tests
- [x] 5.3 Add a fake scenario with a classical queue; write the failing e2e check for a grouped classical queue; review the screenshots
- [x] 5.4 Verify the layer (as 1.2); open the pull request and verify CI is green (found: on CI the e2e check that the cover has loaded read it the instant the page opened; it now waits for it)
- [x] 5.5 Found in the acceptance (7.4): the wave's flash smeared the format ("16/44.1"): its bright copy lacked tabular numbers, so the digits didn't lie over the format's. Test "the flash text lies exactly over the format"; fixed in layer 4
- [x] 5.6 Found in the acceptance (7.4): back from the idle screen, the diagram showed for two frames at a guessed size (236 by 227 pixels) before its measured one, so the diagram jumped. Write the failing e2e check "Back from the idle screen" (the diagram's and "Up next"'s boxes in the first frame equal the settled ones), then measure the box before the first paint and remember logo proportions across screens
- [x] 5.7 Found in the acceptance (7.4), with the title's taller lines (3.6): in landscape "Up next" always showed four entries, so a two-line title pushed the classical queue past the bottom margin. Keep the e2e check "up next groups a classical queue by work" failing in landscape; then bound the information column to the page in landscape as in portrait, so entries that don't fit leave whole; verify with the kiosk tests and the e2e checks
- [x] 5.8 Found in the review: a stale doc comment above the logos' proportions; remove it

## 6. A new session settles before it shows (stack layer 6, DEV-54: application, Music Assistant adapter)

- [ ] 6.1 Write failing application tests "A group resumes", "The provider doesn't answer in time" and "Nothing settling" (a fake provider that settles, with a paused clock); then `SessionProvider::settling`, and the monitor holding the snapshot for at most 2 seconds with a timer; verify with `cargo test -p jukelight-application`
- [ ] 6.2 Write failing provider tests "AirPlay, no answer yet", "AirPlay, answered", "Claimed, no content yet" and "Unreachable"; then the Music Assistant provider's `settling`; verify with `cargo test -p jukelight-adapter-music-assistant`
- [ ] 6.3 Verify the layer (as 1.2); open the pull request and verify CI is green

## 7. Every paused group is held (stack layer 7, DEV-55: Music Assistant adapter)

- [ ] 7.1 Record the failing pause on the reference host (Music Assistant's players and queues, the player's status, and the live feed) at each step; copy into the git-ignored `fixtures/raw/`, commit only scrubbed fixtures
- [ ] 7.2 From the recording: the spec change ("A paused group" in session-music-assistant) and the design (decision 11), then a failing fixture test for the recorded sequence, then the fix; verify with `cargo test -p jukelight-adapter-music-assistant`
- [ ] 7.3 Verify the layer (as 1.2); open the pull request and verify CI is green
- [ ] 7.4 Acceptance on the reference host for the whole stack: the long classical title, a medium and a short title in portrait from across the room, two artists on two lines, the streamer card with the mark, the wave and its flash, the background on a red and a dark cover, a two-speaker group, a classical queue and a pop queue in "Up next", no footer while playing, resuming a group without a jump, pausing a group (held); an hour's CPU against about 64 s; record the results on DEV-48
