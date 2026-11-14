## 1. A softer background (stack layer 1, DEV-49: artwork adapter)

- [ ] 1.1 Write failing artwork tests "Backdrop of a cover" (at most 128 px), "Blurred" (half black, half white: a gradual change, no sharp edge), "Grain against banding" (flat colour: neighbours differ by a few levels at most) and the same cover giving the same bytes; then scale to 128 px, blur and add a fixed-seed grain; verify with `cargo test -p jukelight-adapter-artwork`
- [ ] 1.2 Verify the layer: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, kiosk lint, typecheck, tests and e2e; open the pull request and verify CI is green

## 2. Figtree (stack layer 2, DEV-50: kiosk)

- [ ] 2.1 Write failing kiosk tests "Bundled font" (Figtree, served by Jukelight) and "Measured after loading" (the diagram is laid out again once the font has loaded); then replace `@fontsource-variable/inter` with `@fontsource-variable/figtree` and rebuild the diagram's measure when the font has loaded; verify with the kiosk tests
- [ ] 2.2 Update the project context (`openspec/config.yaml`: the bundled font) and `README.md` where it names the font
- [ ] 2.3 Verify the layer (as 1.2), including the e2e overflow checks in the new font and a screenshot review; open the pull request and verify CI is green

## 3. The track's text and the page's room (stack layer 3, DEV-51: kiosk)

- [ ] 3.1 Write failing component tests "Short title", "Medium title", "Long classical title" and "Title too long for any size"; then the stepped title (7.2 and 6.2 vmin on one line, 5.4 and 4.6 on two, 3.6 on three, in an 11.2 vmin box); verify with the kiosk tests
- [ ] 3.2 Write failing component tests "Two artists on two lines", "Artists and album with icons", "Several artists", "A group's name" and "A name without a group word"; then artists wrapping to two lines and the icons (one person by default, a group for several artists or a group word); verify with the kiosk tests
- [ ] 3.3 Write failing tests for "Device signature" (no footer on the playing screen; the footer on the idle and input screens) and the steady portrait layout ("Title stepped down to two lines" and the artists' exception); then remove the footer from the playing screen, set the margins (5vh top, 4.5vh bottom) and 5vmin under the progress, and let the diagram and "Up next" run to the bottom margin; verify with the kiosk tests
- [ ] 3.4 Write failing e2e checks: a long classical title in full, "Aligned with the cover in portrait" (the edges of the title, progress, diagram and "Up next" against the cover), no footer on the playing screen, overflow checks against the bottom margin for every fake scenario in both orientations; review the screenshots
- [ ] 3.5 Verify the layer (as 1.2); open the pull request and verify CI is green

## 4. The diagram (stack layer 4, DEV-52: kiosk)

- [ ] 4.1 Write failing component and layout tests for the "Streamer card" scenarios (mark and room in the lid, the room set like a speaker, no mark), "Long room name" and "Lines stay in the background"; then the taller lid (19, card 58) and the lighter lines (1.0), ports, dots, icons (stroke 1.4) and border (1.05); verify with the kiosk tests
- [ ] 4.2 Write failing layout tests "What the source streams, as a wave", "A wide logo" and "Two speakers in portrait"; then the wave from the logo to the format (at least 40 units), the speaker rows (32), the gap above the streamer (26), the output rows (26) and the source's line (34); verify with the kiosk tests
- [ ] 4.3 Write failing tests for "Pulse along the diagram" (the wave and its format light up as the pulse starts; nothing lights with reduced motion), then the flash; verify with the kiosk tests and the e2e motion checks
- [ ] 4.4 Write the failing e2e check "two speakers without + N more in portrait", keep the overflow checks; review the screenshots against the approved renders
- [ ] 4.5 Verify the layer (as 1.2); open the pull request and verify CI is green

## 5. "Up next" grouped (stack layer 5, DEV-53: kiosk, fake device)

- [ ] 5.1 Write failing unit tests for the grouping function, one per "Up next" scenario (movements of one work, performer changes, no composer, dash separator, lone title, album, featured artist, artist interrupted, no artist), then the function; verify with the kiosk tests
- [ ] 5.2 Write failing component tests "Long title" (two lines, then "…") and "Heading at the bottom", then the grouped list; verify with the kiosk tests
- [ ] 5.3 Add a fake scenario with a classical queue; write the failing e2e check for a grouped classical queue; review the screenshots
- [ ] 5.4 Verify the layer (as 1.2); open the pull request and verify CI is green
- [ ] 5.5 Acceptance on the reference host for the whole stack: the long classical title, a medium and a short title in portrait from across the room, two artists on two lines, the streamer card with the mark, the wave and its flash, the background on a red and a dark cover, a two-speaker group, a classical queue and a pop queue in "Up next", no footer while playing; an hour's CPU against about 64 s; record the results on DEV-48
