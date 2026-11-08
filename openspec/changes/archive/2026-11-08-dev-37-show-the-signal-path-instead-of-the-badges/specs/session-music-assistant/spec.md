## ADDED Requirements

### Requirement: Delivered quality
Next to the source quality, the provider SHALL report the quality Music
Assistant sends to this player, from the `output_format` of the
`streamdetails.audio_processing.outputs` entry that names the player (or
the only entry), and the changes Music Assistant made on the way:
- **resampled**, when the output's sample rate differs from the source's;
- **bit depth changed**, when only the bit depth differs;
- **normalised**, when the normalisation mode changes the audio (see
  "Source quality"), with the gain from
  `audio_processing.queue_processing.normalization.applied_gain_db`, or
  else `volume_normalization_gain_correct`, when one is reported;
- **DSP**, when the output's `dsp.state` is `enabled`.

Without an output format, the delivered quality SHALL be unknown and only
normalisation and DSP SHALL be reported.

#### Scenario: Padded CD-quality source
- **WHEN** the source is FLAC 16/44.1, the output is FLAC 24/44.1, normalisation is `measurement_only` and DSP is `disabled`
- **THEN** the delivered quality is FLAC 24/44.1 and the only change is "bit depth changed"

#### Scenario: Upsampled and normalised
- **WHEN** the source is FLAC 16/44.1, the output is FLAC 24/96, normalisation is `dynamic` with an applied gain of −6.0 dB, and DSP is `enabled`
- **THEN** the delivered quality is FLAC 24/96 and the changes are "resampled", "normalised (−6.0 dB)" and "DSP"

#### Scenario: Untouched Hi-Res
- **WHEN** the source and the output are FLAC 24/96, normalisation is `disabled` and DSP is `disabled`
- **THEN** the delivered quality is FLAC 24/96 with no changes

#### Scenario: No output for the player
- **WHEN** `audio_processing` has no outputs
- **THEN** the delivered quality is unknown
