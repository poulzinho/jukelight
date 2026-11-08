## 1. What Music Assistant does to the audio (stack layer 1, DEV-38: domain, Music Assistant adapter)

- [x] 1.1 Write failing domain unit tests for `AudioQuality::class()` (Hi-Res, lossless, lossy) and `signal_class` (upsampled CD quality stays lossless, downsampled Hi-Res becomes lossless, no quality is unknown), then add `QualityClass`, `Delivery`, `Change` and `Content.delivery`; verify with `cargo test -p jukelight-domain`
- [x] 1.2 Write failing MA fixture tests "Padded CD-quality source", "Upsampled and normalised", "Untouched Hi-Res" and "No output for the player" (scrubbed edited copies of the recorded `streamdetails`), then map the delivery from `audio_processing` (output format, `applied_gain_db` with the `volume_normalization_gain_correct` fallback, `dsp.state`); verify with `cargo test -p jukelight-adapter-music-assistant`
- [x] 1.3 Verify the layer: `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, kiosk lint, typecheck and tests; open the pull request and verify CI is green

## 2. The signal in the API (stack layer 2, DEV-39: API, fake adapter)

- [x] 2.1 Write failing HTTP tests "Hi-Res through Music Assistant", "Upsampling stays lossless", "Downsampled Hi-Res", "Native service", "Quality not reported" and "Input", then publish `signal` (`SignalDto`, tagged changes, `hiResLogoUrl` only for Hi-Res); verify with `cargo test -p jukelight-adapter-http`
- [x] 2.2 Kiosk API types for `signal`; fake scenarios `upsampled` and `unknown-quality` (the existing `cast-without-queue` is the lossy one), with the delivery on the `music-assistant` and `long-badges` scenarios; verify with the kiosk typecheck and by reading `/api/v1/player` for each scenario
- [ ] 2.3 Verify the layer (as 1.3); open the pull request and verify CI is green

## 3. The signal path on the kiosk (stack layer 3, DEV-40: kiosk, reference host)

- [ ] 3.1 Move `Logo` and `SourceIcon` to `Brand.tsx`; write failing component tests "Through Music Assistant", "Native service", "No logos" and "Route unavailable", then `SignalPath` with its nodes and waves; verify with the kiosk tests
- [ ] 3.2 Write failing component tests for "Signal colours" and every "Sound box" scenario, then the colour tokens (with their soft shades) and `SoundBox`; verify with the kiosk tests
- [ ] 3.3 Write failing component tests "Reduced motion" and "Paused" (no pulse rendered) and an e2e check that the running animations change only `transform` and `opacity`, then the masked wave with its moving band and the arrival port; verify with the kiosk tests and Playwright
- [ ] 3.4 Write failing tests for the playing screen ("Track with all data", "Quality changes") and the input screen ("Optical input"), then replace `Badges` with `SignalPath`, give the input screen its icon and label, remove `Badges.tsx` and the accent tint of the quality; verify with the kiosk tests
- [ ] 3.5 Write failing screenshot checks "Landscape" and "Narrow portrait" (no part wraps or overflows; labels, then the third line, give way) and keep the steady portrait layout checks passing, with the fake scenarios from 2.2 (`long-badges` renamed `long-signal`), then the fitting; review the screenshots in landscape and portrait
- [ ] 3.6 Verify the layer (as 1.3); open the pull request and verify CI is green
- [ ] 3.7 Acceptance on the reference host: Qobuz via MA (Hi-Res, normalised), a Subsonic or Navidrome album via MA (upsampled), Spotify (lossy), MA unavailable, pause, the optical input; portrait; the kiosk's CPU use over an hour against the badges' build, as for DEV-28; record the results on DEV-37
