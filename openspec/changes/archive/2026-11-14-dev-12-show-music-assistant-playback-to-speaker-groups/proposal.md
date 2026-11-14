# DEV-12: Show Music Assistant playback to speaker groups

Together with DEV-13 (show which speakers are playing), in one stack:
DEV-12 is layer 1, DEV-44 and DEV-45 (sub-issues of DEV-13) are layers 2
and 3, and DEV-46 (a sub-issue of DEV-12, found in the acceptance) is
layer 4.

## Why

When Music Assistant plays to several speakers at once, it sends to every
speaker itself, the player included, over Sendspin or AirPlay instead of
its own HTTP stream. The player then reports only an AirPlay session, so
the kiosk shows "AirPlay" with no track, queue or colours, although Music
Assistant knows all of it. And nothing on the screen says that the music
also plays in other rooms, in which quality each room receives, or which
speaker hangs on the player's own output. Part of roadmap step 4 (finished
kiosk).

Recordings on the reference host (Music Assistant 2.10, a Bluesound Node
grouped with two other speakers) show what is available:

- **BluOS** reports the group session like any AirPlay session, and
  `/SyncStatus` shows no group. When the group pauses, the player reports
  stopped.
- **Music Assistant** reports, for this player, `active_source` = its own
  queue (or the group leader's) during a group session, and `AirPlay` when
  a phone plays by AirPlay; the group's members (`group_members`,
  `synced_to`); and one output format per set of speakers in the queue
  item's `audio_processing.outputs`.
- **BluOS** reports its subwoofer: `/SyncStatus` has `hasSubwoofer="true"`
  only while the subwoofer is switched on.

## What Changes

- **DEV-12, a group session is Music Assistant's:** during an AirPlay
  session the Music Assistant provider asks Music Assistant whether it
  drives the player; it claims the session when Music Assistant's
  `active_source` is one of its queues, and then shows that queue (the
  player's own, or the group leader's). A phone's AirPlay stays "AirPlay".
  This is the one claim that needs Music Assistant to answer.
- **DEV-13, the speakers:** the snapshot carries the other speakers of the
  group (name, kind, whether they play, the format each receives) and the
  player's own outputs: the speaker on its output (from configuration;
  the player can't know it) and its subwoofer (from BluOS, while on).
- **Your names:** `SPEAKER_NAMES` renames speakers (by the name Music
  Assistant shows), `PLAYER_OUTPUT_NAME` names the speaker on the player's
  output; both in `.env`, later edited in the admin app (DEV-6).
- **The signal path becomes a diagram of the group:** the source's logo
  and what it streams, Music Assistant's logo, one row per speaker with
  its format and colour, the player as a streamer card (Hi-Res logo,
  maker's mark) and its outputs. With a queue it runs top to bottom in a
  left column, "Up next" in the right one; without a queue it runs left to
  right across the width. Straight lines with right angles; one pulse runs
  along the whole tree on a change.
- **DEV-46, a paused group shows as paused:** when Music Assistant
  pauses a group, the player goes back to its own previous queue and
  reports stopped, while Music Assistant still holds the group's track.
  The provider keeps the session and shows that track as paused until
  the player plays again or Music Assistant lets go of the track; the
  screen goes off after the grace period, as for any pause.
- **The maker's mark:** a second user-supplied logo,
  `brand-assets/<maker>-mark.svg` (for example the Bluesound "B"), shown in
  the streamer's corner.

### Non-goals

- BluOS's own groups (one BluOS player streaming to others): the
  reference host has one BluOS player; the design leaves room for them.
- Volume per speaker, a speaker's own processing after it receives the
  stream.
- Editing the names in an admin app (DEV-6).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-music-assistant`: claiming AirPlay group sessions, the group's
  queue, the group's speakers and the format each receives.
- `device-bluos`: the player's subwoofer.
- `session-arbitration`: claims that need the provider's answer; the
  speakers, outputs and names on the snapshot.
- `published-api`: `speakers`, `outputs` and the maker's mark.
- `kiosk-display`: the signal path as a diagram of the group, its two
  layouts, its fitting and its pulse.

## Impact

- `crates/domain`: `Speaker`, `Output` on the content and the device
  snapshot.
- `crates/adapter-music-assistant`: the claim for AirPlay sessions, the
  group queue, speakers and per-speaker formats; new recorded fixtures
  (scrubbed: example names and addresses only).
- `crates/adapter-bluos`: `hasSubwoofer`.
- `crates/application`, `crates/app`: names from configuration;
  `.env.example`, `README.md`, `docs/reference-setup.md`.
- `crates/adapter-http`: `speakers`, `outputs`, `device.markUrl`.
- `web/kiosk`: a new `SignalPath` diagram and the two-column layout;
  e2e screenshots and overflow checks for every scenario.
- Reference host: an hour's CPU measurement, since the pulse now runs
  along more lines.
